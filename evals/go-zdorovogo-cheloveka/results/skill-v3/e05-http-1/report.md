Исправлены все три проблемы в statusapi:

**Изменения в client.go:**
1. GetStatus теперь учитывает отмену контекста: использует http.NewRequestWithContext вместо Get
2. На несуществующую задачу (404) возвращает ErrNotFound; на другие коды != 200 - StatusError
3. resp.Body закрывается через defer на всех путях выхода, тело дочитывается для возврата соединения в пул

**Добавлены тесты в client_test.go:**
- TestGetStatusNotFound: проверяет ErrNotFound на 404
- TestGetStatusError: проверяет StatusError на других кодах
- TestGetStatusContextCancel: проверяет реакцию на отмену контекста
- TestGetStatusConnectionReuse: проверяет переиспользование соединений

**Проверки:**
- go test -v: все 5 тестов пройдены
- go test -race: гонок нет
- go vet: без ошибок
- gofmt: форматирование верно

Код теперь соответствует контракту в doc-комментарии GetStatus.
