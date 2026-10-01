## Исправлены race conditions в ttlcache

### Изменения в коде:

**cache.go:**
1. Добавлена защита мьютексом в методе `Get()` (строки 36-37)
   - Была гонка данных между чтением в Get и записью в Set
   - Теперь Get блокирует мьютекс перед доступом к map

2. Исправлен получатель метода `Len()` с `(c Cache)` на `(c *Cache)` (строка 46)
   - Было: значение получателя не защищал оригинальный мьютекс
   - Теперь: указатель получателя корректно защищает данные

**cache_test.go:**
- Добавлен `TestConcurrentAccess()` для проверки гонок данных
- Тест проверяет одновременный доступ из трех горутин: запись, чтение и cleanup

### Результаты проверок:

✓ `go test -race -count=5` - PASS (все итерации успешны)
✓ `go test -v -count=10` - PASS (стабильность подтверждена)
✓ `go vet ./...` - no errors
✓ `gofmt` - форматирование OK

### Файлы:
- `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-1/ttlcache/cache.go`
- `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-1/ttlcache/cache_test.go`

Кэш теперь безопасен для одновременного использования из нескольких горутин без race conditions.
