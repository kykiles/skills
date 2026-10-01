Исправлены все три проблемы в statusapi:

1. **GetStatus теперь реагирует на отмену контекста** — используется http.NewRequestWithContext вместо client.Get, контекст передаётся в HTTP-запрос

2. **На несуществующую задачу (404) возвращается ErrNotFound** — добавлена проверка resp.StatusCode перед разбором JSON, при 404 возвращается нулевой Status с ErrNotFound вместо пустого статуса без ошибки

3. **Соединения больше не копятся** — добавлен defer resp.Body.Close() и дочитывание части тела при ошибках для возврата соединений в пул

Изменённые файлы:
- /tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v3-2/statusapi/client.go (+22 строк)
- /tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v3-2/statusapi/client_test.go (+67 строк)

Прочитанные справочники: concurrency.md, types-errors.md, io-os.md

Проверки:
- go test -v: PASS (все 4 теста)
- go test -race: PASS (нет гонок)
- go vet: PASS (нет ошибок)
- gofmt: OK (код отформатирован)
