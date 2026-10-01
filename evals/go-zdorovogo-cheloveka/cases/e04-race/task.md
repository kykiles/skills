Сервис, который использует ttlcache, иногда падает с `fatal error: concurrent map read and map write`, а `go test -race` в сервисе ругается на код ttlcache. Исправь кэш.
