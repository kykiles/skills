Привел в порядок runner.Run согласно контракту:

**Что изменено:**

1. **runner.go** (33 строки изменены, удален import "strings", добавлены import "errors" и "time"):
   - Заменил `exec.Command("sh", "-c", ...)` на `exec.CommandContext(ctx, name, args...)` — аргументы теперь доходят до программы как есть, без интерпретации shell
   - Раздельно собираю stdout и stderr в отдельных `bytes.Buffer` вместо общего буфера
   - Добавил `cmd.WaitDelay = 5 * time.Second` для правильного завершения потомков процесса
   - Ненулевой код выхода больше не ошибка: код кладется в `Result.ExitCode`
   - Обработка отмены ctx: если `ctx.Err() != nil` и произошла ошибка, возвращаю `ctx.Err()` (для которого истинно `errors.Is(err, ctx.Err())`)
   - Обновил doc-комментарий с полным описанием контракта

2. **runner_test.go** (87 строк добавлено):
   - TestRunEcho — базовый тест (был)
   - TestRunArgumentsWithSpaces — проверка аргументов с пробелами
   - TestRunArgumentsWithSpecialChars — проверка спецсимволов ($, *, ;, ") не интерпретируются
   - TestRunSeparateStderr — проверка раздельности stdout и stderr
   - TestRunNonZeroExitCode — ненулевой код выхода не является ошибкой
   - TestRunContextCancellation — отмена ctx возвращает context.Canceled
   - TestRunContextTimeout — истечение таймаута возвращает context.DeadlineExceeded

**Проверки запущены:**

- ✓ `go test -v` (все 7 тестов пройдены на Linux)
- ✓ `go test -v -count=2` (все тесты пройдены дважды для стабильности)
- ✓ `go vet ./...` (на Linux)
- ✓ `GOOS=windows go vet ./...` (кросс-компиляция для Windows)
- ✓ `GOOS=windows go build ./...` (сборка для Windows)
- ✓ `gofmt -l runner.go runner_test.go` (форматирование OK)
- ✓ Построен бинарник tool и протестирован:
  - echo "hello world" работает
  - printf с аргументами, содержащими пробелы, работает
  - ненулевой код выхода (exit 42) правильно передается
  - stdout и stderr раздельно

**Справочники прочитаны:**
- [concurrency.md](references/concurrency.md) — context, горутины, жизненный цикл
- [io-os.md](references/io-os.md) — os/exec, процессы, context, Linux и Windows

**Результат:** контракт выполнен полностью, код готов к использованию на Linux и Windows.
