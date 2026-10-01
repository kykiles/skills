Завершены исправления в клиенте statusapi согласно контракту.

## Изменения в client.go:

1. **Поддержка отмены контекста**:
   - Заменён `http.Client.Get()` на `http.NewRequestWithContext()` + `http.Client.Do()`
   - Теперь запрос корректно реагирует на отмену и дедлайны контекста

2. **Обработка ошибок на несуществующие задачи**:
   - Добавлена проверка `resp.StatusCode == http.StatusNotFound` → возвращает `ErrNotFound`
   - Добавлена проверка других кодов ошибок (не 200) → возвращает `*StatusError`

3. **Закрытие соединений**:
   - Добавлен `defer resp.Body.Close()` для предотвращения утечек соединений

## Изменения в client_test.go:

Добавлены четыре новых теста:
- `TestGetStatusNotFound` — проверка 404 возвращает `ErrNotFound`
- `TestGetStatusOtherError` — проверка других кодов ошибок возвращают `StatusError`
- `TestGetStatusContextCancellation` — проверка реакции на отмену контекста с дедлайном
- `TestGetStatusConnectionClosed` — проверка корректного закрытия соединений при множественных запросах

## Результаты тестирования:

✓ `go test -v`: 5 тестов пройдено (TestGetStatus, TestGetStatusNotFound, TestGetStatusOtherError, TestGetStatusContextCancellation, TestGetStatusConnectionClosed)
✓ `go vet ./...`: ошибок не найдено
✓ `go fmt ./...`: код соответствует стандартам форматирования
✓ `go test -v -race`: все тесты прошли без race conditions

Код теперь полностью соответствует контракту в комментарии к GetStatus.
