Исправлены две проблемы в svcconfig:

1. **Ошибка `<nil>` при стартовании:** Функция validate() возвращала *ValidationError вместо interface error, создавая typed nil проблему. Изменил сигнатуру на error и логику Load() для явной проверки валидации.

2. **errors.Is не срабатывает:** Использовалось %v вместо %w при оборачивании ошибок, что разрывало цепочку ошибок. Исправлено на %w на обеих строках (чтение и парсинг конфигурации).

Добавлены тесты для проверки обеих проблем (TestLoadNotExist, TestLoadValid, TestValidationError). 

Результаты:
- go test -v -race: все 4 теста PASSED
- go vet: ошибок нет  
- gofmt: форматирование корректно
- Ручное тестирование всех сценариев: успешно

Файлы: /tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v3-3/svcconfig/config.go и config_test.go
