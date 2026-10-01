Зафиксированы две гонки данных в ttlcache:

**Изменения в `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v3-3/ttlcache/cache.go`:**

1. **Метод `Get`** — добавлена защита мьютексом при доступе к map `c.items`. Метод читал из map без блокировки, что вызывало гонку при одновременных операциях Set, Get и Cleanup.

2. **Метод `Len`** — изменён получатель с value `(c Cache)` на pointer `(c *Cache)`. При value-получателе копировался мьютекс, и его блокировка не защищала исходные данные — классическая ошибка из справочника concurrency.md.

**Дополнение в `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v3-3/ttlcache/cache_test.go`:**

Добавлены тесты для верификации:
- `TestConcurrentAccess` — одновременные Set, Get, Len из 30 горутин
- `TestConcurrentCleanup` — одновременные операции с запущённой горутиной cleanup

**Проверки запущены:**

1. `go test -race -v` — все 3 теста PASS, race detector не обнаружил гонок
2. `go vet ./...` — нет нарушений  
3. `gofmt -l` — код отформатирован правильно

Справочник: concurrency.md (мьютексы, гонки данных, чтение/запись map)

Все изменения соответствуют контракту "Cache безопасен для одновременного использования из нескольких горутин".
