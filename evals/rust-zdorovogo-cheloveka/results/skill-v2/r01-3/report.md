Задача выполнена. Исправлена обработка ошибок парсинга конфига в relay.

**Изменено:**
- `/tmp/ws/skill-v2/r01-3/src/lib.rs` (78 строк изменено, 73 добавлено)

**Что исправлено:**
1. Добавлены три новых варианта ошибки в `ConfigError` enum:
   - `BadPort` - для ошибок парсинга `port`
   - `BadTimeout` - для ошибок парсинга `timeout_ms`
   - `BadVerbose` - для ошибок парсинга `verbose`

2. Каждый вариант содержит номер строки (`line`) и исходную ошибку парсинга (`source`), что позволяет выводить полную информацию через цепочку `source()`.

3. Заменены три `unwrap()` вызова (строки 95-97) на `map_err()` для нормальной обработки ошибок:
   - `port.parse().map_err(|source| ConfigError::BadPort { line, source })?`
   - `timeout_ms.parse().map_err(|source| ConfigError::BadTimeout { line, source })?`
   - `verbose.parse().map_err(|source| ConfigError::BadVerbose { line, source })?`

4. Обновлена документация функции `parse()` с описанием новых вариантов ошибок.

5. Добавлены три новых теста: `rejects_bad_port()`, `rejects_bad_timeout()`, `rejects_bad_verbose()`.

**Результаты тестирования:**
- `cargo test --offline --all`: 5 тестов пройдено ✓
- `cargo clippy --offline --all`: ошибок не найдено ✓
- `rustfmt --check`: форматирование корректно ✓
- Тестирование поведения:
  - `port = 80o` → "строка 2: некорректный номер порта: invalid digit found in string" (exit code 2) ✓
  - `timeout_ms = abc` → "строка 3: некорректное значение `timeout_ms`: invalid digit found in string" (exit code 2) ✓
  - `verbose = maybe` → "строка 4: некорректное значение `verbose`: provided string was not `true` or `false`" (exit code 2) ✓
  - Отсутствующий файл: "не удалось прочитать /nonexistent.conf: No such file or directory" (exit code 2) ✓
  - Неизвестный ключ: "строка 3: неизвестный ключ `unknown_key`" (exit code 2) ✓
  - Синтаксис: "строка 2: ожидается `ключ = значение`" (exit code 2) ✓
  - Валидный конфиг: корректная загрузка (exit code 0) ✓

**Прочитанные справочники:**
- `/tmp/skill/rust-zdorovogo-cheloveka/references/errors.md` - для правильной обработки ошибок из внешних данных

Код совместим с MSRV 1.74 (используемые API существуют с более ранних версий Rust).
