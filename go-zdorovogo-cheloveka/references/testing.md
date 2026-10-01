# Тесты

## Что проверять

**Принцип:** тест проверяет наблюдаемое поведение из контракта и падает с понятным сообщением. Перед сдачей мысленно удали исправление: упадёт ли тест? Если нет, он проверяет не то.

- Для ошибки сначала напиши тест, который воспроизводит её на исходном коде, затем исправляй.
- Проверяй все обещания контракта, которых касается правка, а не только симптом из задачи. Если в doc-комментарии написано «вход не изменяется» или «при ошибке возвращает nil», тест проверяет и это.
- Проверяй тот путь, о котором заявляешь. Тест с заранее отменённым контекстом проверяет только «отмена до начала»; «отмену во время работы» проверяет тест, где операция уже идёт: сервер ждёт `r.Context().Done()`, процесс спит, `fn` блокируется.
- По умолчанию сравнивай результат целиком (`==` для сравнимых структур, `slices.Equal`, `reflect.DeepEqual` или `cmp.Diff`, если `go-cmp` уже есть в проекте). Если часть полей несущественна или нестабильна (время, сгенерированный ID, порядок без гарантии), проверяй нужные свойства и явно назови, что не сравнивается.
- Ошибки проверяй через `errors.Is`/`errors.As`, а не по тексту.
- Не тестируй константы и стандартную библиотеку. Не меняй существующий тест, чтобы он прошёл: если тест устарел из-за изменения контракта, скажи об этом.

## Как писать

```go
// parseKV разбирает строку вида "ключ=значение" по первому '='.
func parseKV(s string) (key, value string, err error) {
	key, value, ok := strings.Cut(s, "=")
	if !ok || key == "" {
		return "", "", fmt.Errorf("parseKV: нет ключа в %q", s)
	}
	return key, value, nil
}

func TestParseKV(t *testing.T) {
	tests := []struct {
		in, key, value string
		wantErr        bool
	}{
		{in: "a=b", key: "a", value: "b"},
		{in: "a=b=c", key: "a", value: "b=c"},
		{in: "a=", key: "a", value: ""},
		{in: "=b", wantErr: true},
		{in: "ab", wantErr: true},
	}
	for _, tt := range tests {
		t.Run(tt.in, func(t *testing.T) {
			key, value, err := parseKV(tt.in)
			if (err != nil) != tt.wantErr {
				t.Fatalf("parseKV(%q) err = %v; wantErr %v", tt.in, err, tt.wantErr)
			}
			if key != tt.key || value != tt.value {
				t.Errorf("parseKV(%q) = %q, %q; want %q, %q", tt.in, key, value, tt.key, tt.value)
			}
		})
	}
}
```

- Табличный тест — когда случаев несколько и они различаются только данными. Для одного-двух случаев проще отдельные проверки.
- В помощниках вызывай `t.Helper()`, чтобы ошибка указывала на строку теста.
- Освобождай ресурсы через `t.Cleanup`. Временные файлы создавай в `t.TempDir()`, переменные окружения меняй через `t.Setenv` (несовместим с `t.Parallel`). Контекст бери из `t.Context()` (Go 1.24+), он отменяется в конце теста.
- Тестовые данные храни в `testdata/`. Тестовые помощники держи в `_test.go`, а не в основном коде.
- Внешняя сеть и реальные сервисы в тестах недоступны. Используй `httptest.NewServer`, `net.Listen("tcp", "127.0.0.1:0")`, временные каталоги.
- Тест не оставляет в репозитории артефактов: `coverage.out`, бинарников, логов. Профиль покрытия пиши во временный каталог (`-coverprofile=$(mktemp)`).

## HTTP

```go
func TestRequestHonoursDeadline(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		<-r.Context().Done() // отвечает только после отмены клиентом
	}))
	defer srv.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 50*time.Millisecond)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, srv.URL, nil)
	if err != nil {
		t.Fatal(err)
	}
	resp, err := srv.Client().Do(req)
	if err == nil {
		resp.Body.Close()
	}
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("err = %v; want context.DeadlineExceeded", err)
	}
}
```

Закрытие `resp.Body` проверяется обёрткой `http.RoundTripper`, которая подменяет `Body` и считает вызовы `Close`.

## Конкурентный код

- Синхронизируй тест каналами, `sync.WaitGroup` и явными сигналами из тестовой функции, а не `time.Sleep`. Sleep делает тест медленным и нестабильным под нагрузкой CI.
- Зависание должно ронять тест, а не вешать его: оборачивай вызов в ожидание с таймаутом.
- Запускай `go test -race`, а для поиска редких сбоев — с `-count=20`.
- Утечку горутин проверяй по поведению: функция дождалась начатой работы, после возврата нет новых вызовов. Если в проекте есть `go.uber.org/goleak`, используй его. Сравнение `runtime.NumGoroutine()` — слабый дополнительный сигнал.
- Код со временем (таймауты, тикеры) с Go 1.25 тестируй в `synctest.Test`: в «пузыре» время виртуальное и сдвигается, когда все горутины заблокированы.

```go
// within проверяет, что f завершилась за d; иначе тест падает, а не висит.
func within(t *testing.T, d time.Duration, f func()) {
	t.Helper()
	done := make(chan struct{})
	go func() {
		defer close(done)
		f()
	}()
	select {
	case <-done:
	case <-time.After(d):
		t.Fatalf("не завершилось за %v", d)
	}
}

func TestDeadlineVirtualTime(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		start := time.Now()
		within(t, time.Minute, func() { <-ctx.Done() })
		if got := time.Since(start); got != 5*time.Second {
			t.Fatalf("прошло %v; want 5s виртуального времени", got)
		}
	})
}
```

## Внешние процессы

Тест, который запускает `sh`, `echo` или `printf`, не работает на Windows. Переносимый способ — сам тестовый бинарник в роли дочерней программы:

```go
func TestMain(m *testing.M) {
	if os.Getenv("TEST_HELPER_MODE") == "echo-args" {
		// Дочерний режим: печатаем аргументы и выходим, не запуская тесты.
		fmt.Print(strings.Join(os.Args[1:], "\n"))
		os.Exit(0)
	}
	os.Exit(m.Run())
}

func TestArgsReachChildVerbatim(t *testing.T) {
	args := []string{"a b", `"q"`, "$HOME", "x;y", "*"}
	cmd := exec.Command(os.Args[0], args...)
	cmd.Env = append(os.Environ(), "TEST_HELPER_MODE=echo-args")
	out, err := cmd.Output()
	if err != nil {
		t.Fatal(err)
	}
	if got := strings.Split(string(out), "\n"); !slices.Equal(got, args) {
		t.Fatalf("дочерний процесс получил %q; want %q", got, args)
	}
}
```

В пакете может быть только один `TestMain`. Если он уже есть, добавь дочерний режим в него.

## Рефакторинг

Если поведение нужно сохранить байт в байт, а существующие тесты покрывают мало, скопируй старую реализацию в `_test.go` под другим именем и сравни с новой на множестве входов: граничные случаи и случайные данные с фиксированным зерном. После рефакторинга такой тест можно оставить как регрессионный или удалить по договорённости с пользователем.

## Fuzzing и бенчмарки

```go
func FuzzParseKV(f *testing.F) {
	f.Add("a=b")
	f.Add("=")
	f.Fuzz(func(t *testing.T, s string) {
		key, value, err := parseKV(s)
		if err != nil {
			return
		}
		if got := key + "=" + value; got != s {
			t.Fatalf("parseKV(%q) = %q, %q; обратная сборка даёт %q", s, key, value, got)
		}
	})
}

func BenchmarkParseKV(b *testing.B) {
	for b.Loop() {
		parseKV("key=value")
	}
}
```

- Fuzzing оправдан для разборщиков и кода, принимающего недоверенный ввод. Обычный `go test` прогоняет только начальные значения; поиск запускается `go test -fuzz=FuzzParseKV -fuzztime=30s`.
- Оптимизацию начинай с измерения: бенчмарк до и после при одинаковых условиях, сравнение через `benchstat`, профиль (`-cpuprofile`, `-memprofile`) для поиска узкого места. `b.Loop()` появился в Go 1.24; при более старой строке `go` пиши `for i := 0; i < b.N; i++`.

## Источники

- [testing](https://pkg.go.dev/testing) — `T.Helper`, `Cleanup`, `TempDir`, `Setenv`, `Context`, `B.Loop`, `F.Fuzz`; [testing/synctest](https://pkg.go.dev/testing/synctest).
- [Go Wiki: TableDrivenTests](https://go.dev/wiki/TableDrivenTests), [Go Test Comments](https://go.dev/wiki/TestComments).
- [Go Fuzzing](https://go.dev/doc/security/fuzz/), [Testing concurrent code with testing/synctest](https://go.dev/blog/synctest).
- [net/http/httptest](https://pkg.go.dev/net/http/httptest); дочерний процесс-тестовый бинарник — приём из тестов [os/exec](https://cs.opensource.google/go/go/+/refs/tags/go1.25.0:src/os/exec/exec_test.go).
