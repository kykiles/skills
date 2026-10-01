Исправлена проблема в пакете cmdargs, когда аргументы нескольких команд с общим профилем перезаписывали друг друга.

**Что изменено:**

Файл `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-1/cmdargs/cmdargs.go`:
- Функция `WithoutEmpty`: заменено `out := args[:0]` на `out := make([]string, 0, len(args))`
- Это предотвращает использование одного и того же underlying array разными срезами

Файл `/cmdargs_test.go`:
- Добавлен новый тест `TestProfileSharedUnderlying` для воспроизведения и проверки исправления

**Проблема:**
Функция `WithoutEmpty` использовала `args[:0]` для создания среза, который указывал на тот же underlying array. Когда разные команды вызывали `Command()` на одном `Profile`, они все работали с одним и тем же underlying array. При append операциях во второй команде перезаписывались аргументы первой.

**Решение:**
Создание нового массива через `make()` гарантирует независимость срезов, возвращаемых `WithoutEmpty`. Это соответствует контракту функции ("args не изменяется").

**Проверки:**
- `go test -v`: 3 теста пройдены (TestBuild, TestWithoutEmpty, TestProfileSharedUnderlying)
- `go vet ./...`: нет предупреждений
- `go test -race`: нет data races

Все проверки успешны.
