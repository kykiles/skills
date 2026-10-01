## Исправлены все три проблемы в statusapi/client.go

### Что изменено:

**Файл: `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/base-3/statusapi/client.go`**

Переписана функция `GetStatus()` (строки 47-71) с исправлением трёх критических проблем:

1. **Поддержка отмены контекста** — заменён вызов `http.Client.Get()` на `http.NewRequestWithContext()` + `http.Client.Do()`, что позволяет уважать deadlines и cancellation signals контекста.

2. **Проверка HTTP-статусов** — добавлена обработка ответов:
   - HTTP 404 → возвращает `ErrNotFound`
   - Другие статусы ≠ 200 → возвращает `&StatusError{Code: статус_код}`
   - HTTP 200 → парсит JSON нормально

3. **Предотвращение утечек соединений** — добавлен `defer resp.Body.Close()` для гарантированного закрытия тела ответа даже при ошибках парсинга.

### Запущены тесты:

**Файл: `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/base-3/statusapi/client_test.go`**

Добавлены четыре новых теста, покрывающих все три исправления:

- `TestGetStatusContextCancellation` — проверяет, что функция правильно прерывается при истечении deadline контекста
- `TestGetStatusNotFound` — проверяет, что 404 возвращает `ErrNotFound`
- `TestGetStatusError` — проверяет, что другие HTTP-ошибки (400, 403, 500, 503) возвращают `*StatusError`
- `TestGetStatusBodyClosed` — проверяет, что тела ответов закрываются (100 последовательных запросов без утечек соединений)

**Результаты тестирования:**
- ✅ Все 5 тестов PASS (старый + 4 новых)
- ✅ Race detection PASS (проверка на race conditions)
- ✅ Тесты запущены 5 раз подряд (-count=5) — все PASS
- ✅Timeout 30s — нет зависаний

Код теперь полностью соответствует контракту, описанному в комментариях к GetStatus.
