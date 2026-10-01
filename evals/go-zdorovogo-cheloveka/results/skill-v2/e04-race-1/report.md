Исправлены race condition проблемы в ttlcache:

1. Метод Get теперь защищён мьютексом (добавлены c.mu.Lock() и defer c.mu.Unlock())
2. Метод Len имеет правильный получатель *Cache вместо Cache

Добавлен конкурентный тест TestConcurrentAccess.

Все проверки пройдены:
- go test -race -v: PASS
- go test -v -race -count=3: PASS
- go vet ./...: OK
- go build ./...: OK

Файлы изменены:
- cache.go (Get и Len методы)
- cache_test.go (добавлен TestConcurrentAccess)
