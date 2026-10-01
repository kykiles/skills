Исправлены race conditions в ttlcache:

ИЗМЕНЕНИЯ:
1. Метод Get() - добавлена защита mutex.Lock()/Unlock() при доступе к c.items (строки 36-37). Было: открытый доступ к карте без блокировки. Теперь: безопасный доступ с мьютексом.

2. Метод Len() - изменен receiver с Cache на *Cache (строка 46). Было: неправильный receiver value type. Теперь: согласованно используется pointer receiver как в остальных методах.

ПРОВЕРКИ:
- go test -race -v: PASSED - базовые тесты без ошибок
- TestConcurrentReadWrite: PASSED - новый стресс-тест с одновременными операциями (чтение, запись, Len, Snapshot, cleanup goroutine) без race conditions
- go test -race -count=5: PASSED - 5 прогонов без ошибок
- go build: SUCCESS - код компилируется без ошибок

Проблема исправлена. Кэш теперь безопасен для одновременного использования и не будет вызывать "fatal error: concurrent map read and map write" при параллельном доступе.
