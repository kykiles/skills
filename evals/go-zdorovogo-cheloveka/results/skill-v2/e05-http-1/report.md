## Исправления statusapi

### Что изменено

Исправлены три проблемы в `GetStatus`, которые нарушали контракт, описанный в doc-комментарии:

**1. Отмена контекста не обрабатывалась**
   - Было: `client.Get()` не использует контекст
   - Стало: `http.NewRequestWithContext(ctx, ...)` передает контекст в запрос
   - Результат: GetStatus теперь прерывается при отмене контекста или дедлайне

**2. На несуществующую задачу возвращался пустой статус без ошибки**
   - Было: код не проверял HTTP-статус перед разбором тела, декодировал любой ответ
   - Стало: проверка statusCode перед декодированием
   - 404: возвращает `ErrNotFound`
   - Остальные статусы ≠200: возвращает `*StatusError`
   - Результат: контракт соблюдается, клиент может различать ошибки

**3. Под нагрузкой копились открытые соединения**
   - Было: `resp.Body` никогда не закрывался
   - Стало: `defer resp.Body.Close()` сразу после получения ответа
   - Дополнительно: при ошибках дочитываем немного тела (`io.LimitReader`), чтобы соединение вернулось в пул
   - Результат: соединения правильно освобождаются

### Файлы

- `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v2-1/statusapi/client.go` — основное исправление
- `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v2-1/statusapi/client_test.go` — новые тесты

### Проверки

**Tests:**
```
go test -v ./...
=== RUN   TestGetStatus
--- PASS: TestGetStatus (0.00s)
=== RUN   TestGetStatusNotFound
--- PASS: TestGetStatusNotFound (0.00s)
=== RUN   TestGetStatusError
--- PASS: TestGetStatusError (0.00s)
=== RUN   TestGetStatusContextCancellation
--- PASS: TestGetStatusContextCancellation (0.10s)
PASS
ok  	example.com/statusapi	0.109s
```

**Race detector:**
```
go test -race
PASS
ok  	example.com/statusapi	1.124s
```

**go vet:** OK (no warnings)

**Build:** successful

**Форматирование:** соответствует стилю проекта, `gofmt` не требуется

### Новые тесты покрывают

- ✓ Успешный запрос 200 OK
- ✓ Ошибка 404 → ErrNotFound, zero Status
- ✓ Ошибка 500 → *StatusError, zero Status
- ✓ Отмена контекста → context.DeadlineExceeded, zero Status

### Что не проверено и почему

- Реальные соединения под нагрузкой требуют интеграционного теста, не unit-теста
- Проверка, что соединение возвращается в пул, косвенно подтверждается отсутствием утечек в тестах и работой connection pooling в `http.Client`

### Совместимость

Изменение совместимо с кодом, который уже использует контракт (проверяет ErrNotFound и *StatusError). Код, игнорировавший ошибки, теперь будет их получать — это исправление, а не регрессия.
