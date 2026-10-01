Завершено приведение runner.Run в порядок. 

**Изменено:**

1. `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v4-3/runner/runner.go`
   - Заменён `exec.Command("sh", "-c", ...)` на `exec.CommandContext(ctx, name, args...)` — аргументы теперь доходят до программы как есть без интерпретации shell (пробелы, кавычки, `;`, `$`, `*` сохраняются)
   - stdout и stderr теперь собираются раздельно в разные буферы
   - Ненулевой код выхода кладётся в `Result.ExitCode`, ошибка возвращается только при отмене ctx или сбое запуска
   - Добавлена обработка `exec.ExitError` с проверкой `errors.As` 
   - При отмене ctx программа останавливается, возвращается ошибка для которой `errors.Is(err, ctx.Err())` истинно
   - Добавлен `WaitDelay = 5s` (Go 1.20) чтобы не ждать вечно, если потомки держат stdout/stderr
   - Обновлены импорты: добавлены `errors` и `time`, удалена `strings`
   - Обновлен doc-комментарий функции Run с полным описанием контракта

2. `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v4-3/runner/runner_test.go`
   - Добавлены 4 новых теста для проверки функционала:
     - TestArgsWithSpecialChars: проверяет что аргументы со спецсимволами доходят как есть
     - TestStdoutStderr: проверяет раздельность stdout и stderr
     - TestExitCode: проверяет что ненулевой код выхода в Result.ExitCode, а не ошибка
     - TestContextCancel: проверяет что отмена ctx останавливает программу и возвращает контекстную ошибку

**Справочники:**
- io-os.md (Внешние программы: os/exec) — полный пример runTool с контрактом
- concurrency.md (context) — правила работы с контекстом

**Проверки:**
- `go test -v` (5 тестов): все прошли ✓
- `gofmt -l`: форматирование OK ✓
- `go vet ./...`: ошибок нет ✓
- `go test -race -v`: гонок не найдено ✓
- `GOOS=windows go vet ./...`: кросс-платформенная компиляция OK ✓

Версия Go: 1.22 (WaitDelay доступен).
