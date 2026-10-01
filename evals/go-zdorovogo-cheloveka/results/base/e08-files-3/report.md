Исправлены две проблемы в функции CountMatches пакета logscan:

1. "too many open files": удалён defer из цикла, добавлены явные f.Close() после каждого файла
2. Неполный результат на больших логах: добавлена проверка ошибок сканера, буфер увеличен до 10MB

Файл: /tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/base-3/logscan/scan.go

Запущены 3 теста: базовый, для больших строк (>64KB), для 150+ файлов - все PASSED
