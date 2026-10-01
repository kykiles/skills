Исправлены обе проблемы в svcconfig/config.go:

## Проблема 1: Typed nil при ошибке валидации
**Симптом:** С корректным файлом конфигурации сервис не стартует с ошибкой "ошибка конфигурации: <nil>"

**Причина:** Функция Load() возвращала `&cfg, validate(&cfg)`. Когда валидация не пройдёт, функция возвращала конфиг И ошибку, нарушая контракт ("При ошибке возвращает nil и ошибку"). Это создавало typed nil проблему.

**Исправление (строки 47-50):**
```go
// До:
return &cfg, validate(&cfg)

// После:
if ve := validate(&cfg); ve != nil {
    return nil, ve
}
return &cfg, nil
```

## Проблема 2: errors.Is не работает с fs.ErrNotExist
**Симптом:** Проверка `errors.Is(err, fs.ErrNotExist)` в cmd/svc никогда не срабатывает

**Причина:** В config.go строки 40 и 44 использовали `%v` вместо `%w` при оборачивании ошибок. Согласно типов-errors.md, `%v` скрывает причину ошибки от `errors.Is`.

**Исправление (строки 40, 44):**
```go
// До:
return nil, fmt.Errorf("чтение конфигурации %s: %v", path, err)
return nil, fmt.Errorf("разбор конфигурации %s: %v", path, err)

// После:
return nil, fmt.Errorf("чтение конфигурации %s: %w", path, err)
return nil, fmt.Errorf("разбор конфигурации %s: %w", path, err)
```

## Результаты проверок

✓ `go test ./...` — PASSED (TestLoadBadJSON)
✓ `go vet ./...` — no issues  
✓ Integration tests:
  - Missing file: errors.Is(err, fs.ErrNotExist) теперь работает ✓
  - Valid config: загружается корректно ✓
  - Bad validation: Load возвращает nil конфиг и ошибку ✓
  - Invalid JSON: ошибка обёрнута с %w и доступна для проверки ✓

**Файлы изменены:** `/tmp/claude-0/-home-user-skills/8db0c349-fe6f-5994-8f93-8a7d3bda3742/scratchpad/ws/skill-v2-1/svcconfig/config.go` (13 строк изменено)
