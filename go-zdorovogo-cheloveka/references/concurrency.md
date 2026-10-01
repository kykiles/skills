# Горутины, каналы и context

## Нужна ли конкурентность

Начни с последовательного кода. Горутины оправданы, когда есть реальный выигрыш: параллельные независимые операции ввода-вывода, фоновая работа с явным жизненным циклом, использование нескольких ядер на измеренно тяжёлой задаче. Конкурентность «на всякий случай» добавляет гонки, утечки и нестабильные тесты.

## Жизненный цикл горутины

**Принцип:** до написания `go` ответь на три вопроса:

1. Что её останавливает: закрытие входного канала, отмена `ctx`, конец работы?
2. Кто ждёт её завершения: `sync.WaitGroup`, канал результата, `errgroup`?
3. Куда уходит её ошибка или паника?

Функция, которая обещает «не возвращает управление, пока не завершатся начатые вызовы», обязана дождаться всех запущенных горутин на каждом пути выхода — успехе, ошибке и отмене. Ранний `return` из цикла чтения результатов, пока горутины ещё отправляют в канал, — утечка: они навсегда заблокируются на отправке.

Ограниченный параллелизм с остановкой при первой ошибке:

```go
// processAll вызывает work для каждого элемента, не более limit одновременно.
// После первой ошибки или отмены ctx новые элементы не запускаются; функция
// возвращает управление только после завершения всех начатых вызовов.
func processAll(ctx context.Context, items []string, limit int, work func(context.Context, string) error) error {
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	var (
		wg       sync.WaitGroup
		mu       sync.Mutex
		firstErr error
	)
	sem := make(chan struct{}, limit)
	for _, item := range items {
		select {
		case sem <- struct{}{}:
		case <-ctx.Done():
		}
		if ctx.Err() != nil {
			break
		}
		wg.Add(1)
		go func() {
			defer wg.Done()
			defer func() { <-sem }()
			if err := work(ctx, item); err != nil {
				mu.Lock()
				if firstErr == nil {
					firstErr = err
					cancel()
				}
				mu.Unlock()
			}
		}()
	}
	wg.Wait()
	if firstErr != nil {
		return firstErr
	}
	return ctx.Err()
}
```

**Проверка выбора:** для каждого `return` функции найди, где ждут горутины (`wg.Wait()`), и что разблокирует каждую их отправку или получение. `wg.Add` вызывается до `go`, а не внутри горутины. Если `golang.org/x/sync` уже есть в `go.mod`, то же самое короче даёт `errgroup.WithContext` с `SetLimit`; ради одной функции зависимость не добавляй. С Go 1.25 вместо `wg.Add(1); go func() { defer wg.Done(); ... }()` можно писать `wg.Go(func() { ... })`. Пример полагается на Go 1.22+, где `item` своя в каждой итерации; при `go` ниже 1.22 передавай `item` аргументом горутины.

## context

- `ctx` — первый параметр функции, которая блокируется или делает ввод-вывод. Не храни его в структуре и не подменяй: передавай дальше полученный.
- Внутреннюю отмену выводи из полученного контекста: `ctx, cancel := context.WithCancel(ctx)`. Контекст от `context.Background()` внутри функции теряет отмену и дедлайн вызывающего. `Background()` — для `main`, инициализации и тестов.
- `cancel` от `WithCancel`, `WithTimeout`, `WithDeadline` вызывай всегда, обычно `defer cancel()`: иначе ресурсы контекста живут до отмены родителя (`go vet` проверяет это анализатором lostcancel).
- Блокирующая операция в горутине, которая может пережить потребителя, ждёт и `<-ctx.Done()`. Длинный цикл проверяет `ctx.Err()` между итерациями.
- При отмене возвращай ошибку, для которой `errors.Is(err, context.Canceled)` или `errors.Is(err, context.DeadlineExceeded)` истинно: `ctx.Err()` или обёртку `%w`. Ошибку отмены, пришедшую от вызванной функции, не заменяй своей.
- `context.WithoutCancel` (Go 1.21) — для работы, которая должна пережить запрос (аудит, сброс кэша); такая работа всё равно нуждается в своём таймауте и владельце.

## Каналы

- Закрывает канал отправитель, и только когда значений больше не будет. Получатель не закрывает чужой канал; повторное закрытие и отправка в закрытый канал паникуют. Если отправителей несколько, закрывай после `wg.Wait()` всех отправителей.
- Получатель читает `for v := range ch` или `v, ok := <-ch`.
- Буфер — 0, 1 или точно известное число значений (например, `len(items)`, чтобы отправители никогда не блокировались). Большой буфер «на всякий случай» прячет взаимоблокировку, а не устраняет её.
- Отправка или получение из nil-канала блокируется навсегда: так можно отключить ветку `select`.

```go
// Плохо: если ctx отменят раньше, горутина навсегда заблокируется на отправке.
func firstResultBad(ctx context.Context, query func() string) (string, error) {
	ch := make(chan string)
	go func() { ch <- query() }()
	select {
	case r := <-ch:
		return r, nil
	case <-ctx.Done():
		return "", ctx.Err()
	}
}
```

```go
// Буфер на одно значение: горутина всегда может отправить результат и завершиться.
// Сам query после отмены продолжит работу, поэтому долгой операции передавай ctx.
func firstResult(ctx context.Context, query func() string) (string, error) {
	ch := make(chan string, 1)
	go func() { ch <- query() }()
	select {
	case r := <-ch:
		return r, nil
	case <-ctx.Done():
		return "", ctx.Err()
	}
}

// generate отдаёт 0..n-1, пока потребитель читает; останавливается при отмене ctx.
func generate(ctx context.Context, n int) <-chan int {
	out := make(chan int)
	go func() {
		defer close(out)
		for i := 0; i < n; i++ {
			select {
			case out <- i:
			case <-ctx.Done():
				return
			}
		}
	}()
	return out
}
```

## Мьютексы и атомики

**Принцип:** мьютекс защищает данные, а не код. Каждое обращение к защищаемому полю — чтение и запись — идёт под тем же мьютексом. Канал передаёт владение или сигнал; общее состояние проще защитить мьютексом.

```go
// Counter безопасен для одновременного использования.
type Counter struct {
	mu     sync.Mutex
	counts map[string]int
}

func NewCounter() *Counter { return &Counter{counts: make(map[string]int)} }

func (c *Counter) Inc(key string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.counts[key]++
}

func (c *Counter) Get(key string) int {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.counts[key]
}

// Snapshot возвращает копию: вызывающий не получает доступ к map в обход мьютекса.
func (c *Counter) Snapshot() map[string]int {
	c.mu.Lock()
	defer c.mu.Unlock()
	return maps.Clone(c.counts)
}
```

**Проверка выбора:**
- Все методы типа с мьютексом — с указательным получателем: получатель-значение копирует мьютекс (`go vet`: copylocks).
- Критическая секция короткая. Под блокировкой не вызывай чужой код (колбэки, сетевые запросы): это ведёт к взаимоблокировкам и долгим ожиданиям.
- `sync.RWMutex` — только при измеренной конкуренции читателей; иначе он сложнее и обычно не быстрее `Mutex`.
- `sync.Map` — для узких случаев (ключи пишутся один раз, или горутины работают с непересекающимися ключами), а не как замена map с мьютексом.
- Для счётчиков и флагов — `atomic.Int64`, `atomic.Bool` (Go 1.19).
- Для однократной инициализации — `sync.Once` или `sync.OnceValue` (Go 1.21).

## Таймеры и ожидание

- `time.Sleep` в рабочем цикле игнорирует отмену. Жди через `select` с `ctx.Done()`.
- `time.NewTicker` останавливай (`defer t.Stop()`).
- При строке `go` ниже 1.23 неостановленный таймер живёт до срабатывания, поэтому `time.After` в частом цикле копит таймеры; используй `time.NewTimer` со `Stop`. С 1.23 такие таймеры собирает GC.

```go
// poll вызывает check сразу и затем каждые every, пока ctx не отменён.
func poll(ctx context.Context, every time.Duration, check func(context.Context) error) error {
	t := time.NewTicker(every)
	defer t.Stop()
	for {
		if err := check(ctx); err != nil {
			return err
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-t.C:
		}
	}
}
```

## Гонки данных

- Одновременные чтение и запись одной map без синхронизации — гонка, которую рантайм часто превращает в `fatal error: concurrent map read and map write`. Её не перехватить `recover`.
- `go test -race` находит гонки только на исполненных путях и требует cgo. Отсутствие отчёта не доказывает отсутствие гонок, взаимоблокировок и утечек: тест должен реально запускать конкурентные операции.
- Тесты конкурентного кода — в [testing.md](testing.md#конкурентный-код).

## Источники

- [Go Code Review Comments: Goroutine Lifetimes, Contexts](https://go.dev/wiki/CodeReviewComments), [Go Concurrency Patterns: Pipelines and cancellation](https://go.dev/blog/pipelines), [Go Concurrency Patterns: Context](https://go.dev/blog/context).
- [context](https://pkg.go.dev/context), [sync](https://pkg.go.dev/sync), [sync/atomic](https://pkg.go.dev/sync/atomic), [time](https://pkg.go.dev/time) — `Timer`, `Ticker` и изменения Go 1.23.
- [The Go Memory Model](https://go.dev/ref/mem), [Data Race Detector](https://go.dev/doc/articles/race_detector).
- [errgroup](https://pkg.go.dev/golang.org/x/sync/errgroup) — если модуль уже используется в проекте.
