## Исправление race condition в ttlcache

### Найденные проблемы:
1. **Метод `Get`** (cache.go, строки 35-41): доступ к map `c.items` без блокировки мьютекса — race condition между чтением и записью
2. **Метод `Len`** (cache.go, строки 44-48): получатель по значению `(c Cache)`, что копирует мьютекс и не защищает данные от конкурентного доступа

### Внесённые изменения:

**Файл: /tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v3-2/ttlcache/cache.go**
- Добавлена блокировка мьютекса в методе `Get`: `c.mu.Lock()` и `defer c.mu.Unlock()` (строки 36-37)
- Изменён получатель метода `Len` со значения на указатель: `func (c *Cache) Len()` вместо `func (c Cache) Len()` (строка 46)

**Файл: /tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v3-2/ttlcache/cache_test.go**
- Добавлен импорт `"sync"`
- Добавлен новый тест `TestConcurrentAccess`, который реально проверяет конкурентность: одновременно запускает 16 горутин, вызывающих Set, Get, Len и StartCleanup в цикле

### Проведённые проверки:

1. **`go test -race -v`** — 2 теста (TestSetGet и TestConcurrentAccess), все PASS, race detector не обнаружил гонок ✓
2. **`go test -race -v -count=5`** — 5-кратный запуск всех тестов, все PASS, race detector не обнаружил гонок ✓
3. **`go vet ./...`** — успешно, предупреждений не найдено ✓
4. **Форматирование (`gofmt`)** — файлы отформатированы корректно ✓

Справочники: concurrency.md

Все гонки данных (concurrent map read and map write) исправлены. Кэш теперь безопасен для одновременного использования из нескольких горутин согласно его контракту.
