**Исправлены две проблемы в функции CountMatches пакета logscan:**

**Проблема 1: "too many open files"**
- Причина: `defer f.Close()` внутри цикла накапливал дескрипторы файлов
- Решение: Удалена отложенная очистка; добавлены явные `f.Close()` сразу после обработки каждого файла

**Проблема 2: Неполные результаты на больших логах**
- Причина: Ошибки scanner не проверялись после цикла
- Решение: Добавлена проверка `sc.Err()` после сканирования; теперь возвращается ошибка при проблемах с I/O

**Файлы изменены:**
- `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-1/logscan/scan.go` - 6 строк изменено (удалена defer, добавлены close и error check)
- `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-1/logscan/scan_test.go` - Добавлен тест TestCountMatchesManyFiles для 100 файлов

**Проверки:**
- `go test -v` → PASS (оба теста пройдены)
- `go vet` → нет проблем
- `go test -race` → нет data race проблем
- `gofmt` → корректное форматирование

Код готов к использованию.
