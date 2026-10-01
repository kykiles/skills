# Ресурсы, сеть, процессы и платформы

## Файлы и Close

**Принцип:** ресурс закрывается там, где открыт: `defer x.Close()` сразу после проверки ошибки открытия. Если есть готовая функция (`os.ReadFile`, `os.WriteFile`, `io.ReadAll`), используй её вместо ручного открытия.

При записи ошибка `Close` (и `Flush` у `bufio.Writer`) — тоже ошибка записи: данные могут не дойти до диска. При чтении её обычно можно не проверять.

```go
// saveLines пишет строки в файл; ошибки Flush и Close возвращаются вызывающему.
func saveLines(path string, lines []string) (err error) {
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer func() {
		if cerr := f.Close(); err == nil {
			err = cerr
		}
	}()
	w := bufio.NewWriter(f)
	for _, l := range lines {
		if _, err := w.WriteString(l + "\n"); err != nil {
			return err
		}
	}
	return w.Flush()
}
```

`defer` выполняется при выходе из функции, а не из итерации. В цикле по файлам вынеси тело в функцию — это тот случай, когда функция с одним вызовом оправдана:

```go
func countLines(paths []string) (int, error) {
	total := 0
	for _, p := range paths {
		n, err := countFileLines(p)
		if err != nil {
			return 0, err
		}
		total += n
	}
	return total, nil
}

// countFileLines закрывает файл до перехода к следующему.
func countFileLines(path string) (int, error) {
	f, err := os.Open(path)
	if err != nil {
		return 0, err
	}
	defer f.Close()
	n := 0
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		n++
	}
	return n, sc.Err()
}
```

Чтобы атомарно заменить файл, пиши во временный файл в том же каталоге (`os.CreateTemp(dir, ...)`), затем `Sync`, `Close` и `os.Rename`. На Windows замена может не пройти, если целевой файл кем-то открыт.

## HTTP-клиент

**Принцип:**
- запрос создаётся с контекстом: `http.NewRequestWithContext`, а не `client.Get`;
- `resp.Body` закрывается на всех путях;
- статус проверяется до разбора тела;
- клиент переиспользуется.

```go
// getJSON запрашивает rawURL и разбирает ответ 200 в v.
func getJSON(ctx context.Context, client *http.Client, rawURL string, v any) error {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, rawURL, nil)
	if err != nil {
		return err
	}
	resp, err := client.Do(req)
	if err != nil {
		return err // при отмене errors.Is(err, context.Canceled) истинно
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		// Дочитываем немного тела, чтобы соединение могло вернуться в пул.
		_, _ = io.Copy(io.Discard, io.LimitReader(resp.Body, 4<<10))
		return fmt.Errorf("GET %s: статус %d", rawURL, resp.StatusCode)
	}
	return json.NewDecoder(io.LimitReader(resp.Body, 10<<20)).Decode(v)
}
```

**Проверка выбора:**
- Соединение возвращается в пул, только если тело дочитано до конца и закрыто.
- `http.Client` безопасен для одновременного использования: создавай один на приложение или компонент, а не на каждый запрос.
- `http.DefaultClient` не имеет таймаута; ограничивай время через `ctx` или `Client.Timeout`. `Client.Timeout` не подходит для долгих потоковых ответов.
- Тело недоверенного ответа читай с ограничением (`io.LimitReader`).
- Части пути экранируй `url.PathEscape`, параметры собирай через `url.Values`.
- Не отключай проверку TLS (`InsecureSkipVerify`); в тестах используй клиент от `httptest.NewTLSServer`.

## HTTP-сервер

**Принцип:** у сервера заданы таймауты (как минимум `ReadHeaderTimeout`), обработчики учитывают `r.Context()`, остановка идёт через `Shutdown`. `log.Fatal` и `os.Exit` — только в `main`.

```go
package main

import (
	"context"
	"errors"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
)

func main() {
	if err := run(); err != nil {
		log.Fatal(err)
	}
}

func run() error {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	srv := &http.Server{
		Addr: ":8080",
		Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			fmt.Fprintln(w, "ok")
		}),
		ReadHeaderTimeout: 5 * time.Second,
	}
	errc := make(chan error, 1)
	go func() { errc <- srv.ListenAndServe() }()

	select {
	case err := <-errc:
		return err
	case <-ctx.Done():
	}
	shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	if err := srv.Shutdown(shutdownCtx); err != nil {
		return err
	}
	if err := <-errc; !errors.Is(err, http.ErrServerClosed) {
		return err
	}
	return nil
}
```

Размер тела запроса ограничивай `http.MaxBytesReader`. После `http.Error` делай `return`: запись в ответ после отправки заголовков ничего не исправит.

## Внешние программы: os/exec

**Принцип:** `exec.Command(name, args...)` запускает программу напрямую, без shell. Каждый аргумент доходит до программы как есть: пробелы, кавычки, `;`, `$`, `*` не интерпретируются. Строка вида `sh -c "prog " + arg` или `cmd /c` с пользовательскими данными — инъекция команд и непереносимость. Если shell нужен ради конвейера или перенаправления, собери это в Go (`cmd.Stdin`, `cmd.Stdout`, `io.Pipe`).

```go
// ToolResult — итог работы программы.
type ToolResult struct {
	Stdout, Stderr string
	ExitCode       int
}

// runTool запускает программу без shell. Ненулевой код выхода — не ошибка;
// ошибка — если программа не запустилась или прервана отменой ctx.
func runTool(ctx context.Context, name string, args ...string) (ToolResult, error) {
	cmd := exec.CommandContext(ctx, name, args...)
	var stdout, stderr bytes.Buffer
	cmd.Stdout, cmd.Stderr = &stdout, &stderr
	cmd.WaitDelay = 5 * time.Second // не ждать вечно, если потомки держат stdout

	err := cmd.Run()
	res := ToolResult{Stdout: stdout.String(), Stderr: stderr.String()}
	if err != nil && ctx.Err() != nil {
		// Процесс, убитый при отмене, выглядит как ExitError: сообщаем об отмене.
		return res, ctx.Err()
	}
	var exitErr *exec.ExitError
	if errors.As(err, &exitErr) {
		res.ExitCode = exitErr.ExitCode()
		return res, nil
	}
	return res, err
}
```

**Проверка выбора:**
- `exec.CommandContext` при отмене убивает только сам процесс. Его потомки могут продолжить работу и держать открытыми `stdout`/`stderr`, поэтому задавай `WaitDelay` (Go 1.20). Остановка всего дерева процессов зависит от ОС: группы процессов на Unix, job objects на Windows.
- `ExitError.ExitCode()` возвращает −1, если процесс завершён сигналом.
- С Go 1.19 `exec.LookPath` и `exec.Command` не находят программу в текущем каталоге по короткому имени (`exec.ErrDot`).
- На Windows `os/exec` экранирует аргументы по правилам обычных программ. `.bat` и `.cmd` выполняются через `cmd.exe` с другими правилами разбора, поэтому недоверенные аргументы туда не передавай.

## Пути и файловая система

- `path/filepath` — для путей ОС, `path` — для путей со слэшами (URL, `embed`, `io/fs`). Собирай пути `filepath.Join`, а не конкатенацией с `/`.
- Имя файла от пользователя проверяй: `filepath.IsLocal` (Go 1.20) отвергает абсолютные пути и выход через `..`. С Go 1.24 `os.OpenRoot` ограничивает все операции каталогом, включая символические ссылки.
- Права вида `0o644`/`0o600` на Windows почти не действуют. Файловые системы Windows и macOS по умолчанию не различают регистр имён.
- `bufio.Scanner` в режиме строк отрезает и `\n`, и `\r\n`. Строка длиннее 64 КиБ даёт ошибку `bufio.ErrTooLong`, если не увеличить буфер через `Scanner.Buffer`.

```go
// readUserFile читает name только внутри dir: "../" и абсолютные пути отвергаются.
func readUserFile(dir, name string) ([]byte, error) {
	root, err := os.OpenRoot(dir)
	if err != nil {
		return nil, err
	}
	defer root.Close()
	f, err := root.Open(name)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	return io.ReadAll(f)
}
```

## Linux и Windows

- Платформенный код размещай в файлах с суффиксом (`proc_windows.go`, `proc_unix.go`) или с ограничением сборки в первой строке (`//go:build windows`, `//go:build !windows`). Общий API у всех вариантов должен совпадать.
- Сигналы: `os.Interrupt` переносим. `syscall.SIGTERM` компилируется под Windows, но доставляется там иначе, чем на Unix. Для остановки по Ctrl+C используй `signal.NotifyContext`.
- `GOOS=windows go vet ./...` и `GOOS=windows go build ./...` проверяют только компиляцию. При кросс-компиляции cgo по умолчанию выключен (`CGO_ENABLED=0`), и файлы с `import "C"` не собираются вовсе. Поведение процессов, путей и сигналов на другой ОС доказывает только запуск на ней (CI или машина).

## JSON

- Поля без тега кодируются по имени поля; неэкспортируемые поля игнорируются.
- `omitempty` опускает нулевые числа, строки, nil и пустые срезы и map, но не структуры. Для структур есть `omitzero` (Go 1.24).
- Nil-срез кодируется как `null`, пустой — как `[]`.
- Неизвестные поля при разборе по умолчанию игнорируются; строгий режим — `Decoder.DisallowUnknownFields`.
- Число в `any` становится `float64`; для точных целых используй `Decoder.UseNumber` или конкретный тип.
- Смена тега или типа поля — изменение формата данных: проверь совместимость с сохранёнными данными и другими сервисами.

## Источники

- [os](https://pkg.go.dev/os) — `Root`, `CreateTemp`; [bufio](https://pkg.go.dev/bufio); [path/filepath](https://pkg.go.dev/path/filepath) — `IsLocal`.
- [net/http](https://pkg.go.dev/net/http) — `Client`, `Response.Body`, `Server`, `Shutdown`; [net/http/httptest](https://pkg.go.dev/net/http/httptest).
- [os/exec](https://pkg.go.dev/os/exec) — `CommandContext`, `Cancel`, `WaitDelay`, `ErrDot`, запуск на Windows.
- [os/signal](https://pkg.go.dev/os/signal); [Build constraints](https://pkg.go.dev/cmd/go#hdr-Build_constraints); [cgo](https://pkg.go.dev/cmd/cgo).
- [encoding/json](https://pkg.go.dev/encoding/json) — `omitempty`, `omitzero`, `Decoder`.
