## Изменения в runner.Run

### Что изменено

Полностью переписана функция `Run` в файле `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v4-2/runner/runner.go` для соответствия контракту использования в CLI на Linux и Windows:

1. **Запуск программы без shell**: Заменено `exec.Command("sh", "-c", ...)` на `exec.CommandContext(ctx, name, args...)`. Это обеспечивает:
   - Передачу аргументов с пробелами, кавычками и символами (`;`, `$`, `*`) как есть, без интерпретации shell
   - Корректное перенесение аргументов на Windows

2. **Разделение stdout и stderr**: Вместо смешивания в один буфер (`cmd.Stdout = &out; cmd.Stderr = &out`) используются два отдельных буфера. Result содержит оба потока отдельно.

3. **Обработка кодов выхода**: 
   - Ненулевой код выхода теперь помещается в `Result.ExitCode`, а не возвращается как ошибка
   - Ошибка возвращается только если программу не удалось запустить

4. **Обработка отмены контекста**:
   - Используется `exec.CommandContext` для корректного прерывания процесса при отмене ctx
   - Установлен `WaitDelay = 5 * time.Second` для обработки дочерних процессов (Go 1.20+)
   - При отмене ctx функция возвращает `ctx.Err()`, так что `errors.Is(err, ctx.Err())` истинно

5. **Обновлен doc-комментарий** с описанием контракта функции, включая обещания относительно передачи аргументов, возврата stdout/stderr, обработки кодов выхода и отмены контекста.

### Добавлены тесты

В файл `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v4-2/runner/runner_test.go` добавлены пять новых тестов:
- `TestRunArgumentsWithSpaces` — проверка передачи аргументов с пробелами
- `TestRunArgumentsWithSpecialChars` — проверка передачи спецсимволов (`;`)
- `TestRunSeparateStdoutStderr` — проверка разделения stdout и stderr
- `TestRunNonZeroExitCode` — проверка возврата ненулевого кода выхода в Result.ExitCode
- `TestRunContextCancellation` — проверка остановки процесса и возврата ctx.Err() при отмене

### Проведённые проверки

```
go test -v          — ✓ Все 6 тестов пройдены (0.020s)
go vet ./...        — ✓ Ошибок не обнаружено
gofmt -l            — ✓ Форматирование корректно
go test -race       — ✓ Гонки данных не обнаружены
GOOS=windows go vet — ✓ Компиляция под Windows без ошибок
go build ./cmd/tool — ✓ Инструмент собирается успешно
```

Ручное тестирование с собранным инструментом:
- `./test-runner printf "test;value"` — ✓ Спецсимволы передаются как есть
- `./test-runner printf "hello world"` — ✓ Пробелы в аргументах сохраняются
- Разделение stdout/stderr — ✓ Работает корректно
- Exit code 42 — ✓ Возвращается правильно в коде выхода процесса

### Справочники, прочитанные для задачи

- `references/io-os.md` — раздел "Внешние программы: os/exec" содержит эталонную реализацию
- `references/types-errors.md` — правила работы с ошибками и `errors.Is/As`

### Файлы изменены

- `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v4-2/runner/runner.go` (35 строк добавлено, 7 удалено)
- `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v4-2/runner/runner_test.go` (82 строки добавлено)
