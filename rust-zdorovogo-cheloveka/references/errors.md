# Ошибки, паники и арифметика

Правила описывают причину выбора, а не вводят запрет на одну из конструкций. Сначала проверь контракт функции, владение данными и поведение при ошибке; затем выбери самый ясный вариант, который решает текущую задачу.

## `Result` или паника (`unwrap`, `expect`, `panic!`, индексация)

**Принцип:** ошибка, которую может вызвать вход, файл, сеть или пользователь, — это `Result` или `Option`. Паника допустима, когда случившееся означает баг в программе: нарушен инвариант, который код гарантирует сам. В таком случае предпочитай `expect` с сообщением о том, *почему* значение должно быть, а не о том, что пошло не так. Индексация `v[i]`, деление на ноль и `unwrap` паникуют так же, как `panic!`, — оценивай их по тому же правилу.

```rust
// Пустой срез — нормальный случай, а не баг: возвращаем Option.
fn middle(values: &[i32]) -> Option<i32> {
    values.get(values.len() / 2).copied()
}

// Непустота проверена строкой выше; expect фиксирует инвариант.
fn first_char(word: &str) -> char {
    assert!(!word.is_empty(), "word must be non-empty");
    word.chars().next().expect("строка непуста: проверено выше")
}
```

**Проверка выбора:** спроси, может ли `None`/`Err` здесь возникнуть при корректной программе и некорректных внешних данных. Если да — паника недопустима, верни ошибку. Если нет — `expect` с объяснением инварианта. В тестах, примерах и прототипах по просьбе пользователя `unwrap` уместен. Публичная функция библиотеки, которая может паниковать, документирует это в разделе `# Panics`, а возвращающая `Result` — в разделе `# Errors`: какие ошибки и при каких условиях. Не заменяй `unwrap` на `unwrap_or_default()` молча: это меняет поведение, а не только стиль. Если ошибку нельзя вернуть наверх и нужно поведение по умолчанию, выбирай его по последствиям: проверка доступа, путь к файлу, подпись, лимит — отказ (fail-closed); отображение, логирование, диагностика — продолжение с запасным значением и записью в лог (fail-open).

## `?` или `match`

**Принцип:** используй `?`, когда текущая функция должна прекратить работу и передать ошибку вызывающей стороне. Используй `match` (или `if let`/комбинаторы), когда нужно принять решение по конкретной ошибке: выбрать запасной путь, повторить операцию, преобразовать значение или выполнить разное действие для разных случаев. При `?` ошибка может преобразовываться через `From`; это должно соответствовать контракту возвращаемого типа.

```rust
// Ошибку чтения обрабатывает вызывающая сторона.
fn line_count(path: &str) -> std::io::Result<usize> {
    let text = std::fs::read_to_string(path)?;
    Ok(text.lines().count())
}

// Отсутствие файла — отдельный допустимый случай, остальные ошибки передаются выше.
fn read_or_empty(path: &str) -> std::io::Result<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error),
    }
}
```

**Проверка выбора:** если все ветви `match` только передают `Err` наверх, попробуй `?` и убери шаблонный код. Если `?` скрывает важное для задачи различие между ошибками, явно обработай его. Не вставляй `?` там, где функция возвращает `()` или несовместимый с ошибкой тип. Не пиши `Ok(operation()?)`, когда тип ошибки совпадает и можно вернуть `operation()` напрямую; если нужно преобразование через `From`, эта форма оправданна (см. пример в разделе «Какой тип ошибки» ниже).

## Какой тип ошибки

**Принцип:** сначала следуй тому, что уже принято в проекте. Библиотека обычно возвращает собственный тип ошибки (enum), чтобы вызывающий мог сопоставить варианты и получить исходную причину через `source()`. Приложение (бинарник) может собирать ошибки в `Box<dyn std::error::Error>` или `anyhow::Error`, если он уже в зависимостях, добавляя контекст там, где он помогает понять сбой. `thiserror` лишь генерирует тот же код, что и ручная реализация, — не добавляй его ради одной правки.

```rust
#[derive(Debug)]
enum ConfigError {
    Read(std::io::Error),
    Port(std::num::ParseIntError),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(_) => write!(f, "не удалось прочитать конфигурацию"),
            Self::Port(_) => write!(f, "некорректный номер порта"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(error) => Some(error),
            Self::Port(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        Self::Read(error)
    }
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::Port(error)
    }
}

// `Ok(...?)` здесь нужен: `?` преобразует ParseIntError в ConfigError.
fn read_port(path: &std::path::Path) -> Result<u16, ConfigError> {
    let text = std::fs::read_to_string(path)?;
    Ok(text.trim().parse()?)
}
```

Вариант с контекстом хранит место сбоя и исходную причину отдельным полем; `source()` возвращает её, а `Display` описывает только свой уровень:

```rust
#[derive(Debug)]
enum ParseError {
    BadField { line: usize, field: &'static str, source: std::num::ParseIntError },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadField { line, field, .. } => write!(f, "строка {line}: некорректное поле `{field}`"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BadField { source, .. } => Some(source),
        }
    }
}

fn parse_count(line: usize, text: &str) -> Result<u32, ParseError> {
    text.trim().parse().map_err(|source| ParseError::BadField { line, field: "count", source })
}
```

**Проверка выбора:** может ли вызывающий по-разному реагировать на разные сбои? Если да — нужны различимые варианты, а не строка. `map_err(|e| e.to_string())`, `map_err(|_| …)` и причина в поле `String` теряют тип и цепочку причин — используй только на границе, где ошибка превращается в сообщение для человека. `Display` описывает текущий уровень и не дублирует текст `source()`, иначе при выводе цепочки сообщение повторится. Добавление варианта в публичный enum ошибок — несовместимое изменение, если enum не помечен `#[non_exhaustive]`.

## Арифметика, `as` и размеры из внешних данных

**Принцип:** переполнение целого в debug-сборке паникует, а в release по умолчанию (`overflow-checks = false`) молча заворачивается. Если число пришло извне (длина или смещение из заголовка, количество элементов, размер от пользователя или из сети), выбери поведение явно: `checked_*` с ошибкой, `saturating_*`, когда насыщение и есть нужный смысл, `wrapping_*`, когда заворачивание задумано (хеши, счётчики по модулю). Приведение `as` между целыми молча обрезает значение и меняет знак; для внешних значений используй `TryFrom`/`try_into()`. Размер из входа сверяй с лимитом до выделения памяти или рекурсии: `Vec::with_capacity(n)` с `n` из заголовка позволяет одним запросом занять всю память.

```rust
/// Больше записей заранее не выделяем, даже если заголовок обещает.
const MAX_RECORDS: usize = 1 << 20;

// `count` и `record_size` пришли из заголовка файла.
fn body_len(count: u64, record_size: u32) -> Option<usize> {
    let count = usize::try_from(count).ok().filter(|&n| n <= MAX_RECORDS)?;
    count.checked_mul(usize::try_from(record_size).ok()?)
}
```

**Проверка выбора:** подставь вместо внешнего значения `0` и максимум типа. Если получится паника в debug, тихий неверный результат в release или попытка выделить гигабайты — нужна явная проверка. Индексы и счётчики, которые ограничивает сама программа, в `checked_*` не оборачивай. Не меняй `overflow-checks` в профиле и не включай линты вроде `clippy::arithmetic_side_effects` ради одной правки — это конфигурация проекта (см. [project.md](project.md#линты-в-конфигурации-проекта)).

## Источники

- Ошибки и паники: [The Book — To panic! or Not to panic!](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html), [The Book — Recoverable Errors with Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html), [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html), [API Guidelines — Error types are meaningful](https://rust-lang.github.io/api-guidelines/interoperability.html#c-good-err), [The Reference — Try propagation expression](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-try-propagation-expression).
- Разделы `# Errors` и `# Panics`: [API Guidelines — Function docs include error, panic, and safety considerations](https://rust-lang.github.io/api-guidelines/documentation.html#c-failure).
- Арифметика: [The Reference — Overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow), [Cargo — профиль `overflow-checks`](https://doc.rust-lang.org/cargo/reference/profiles.html#overflow-checks), [std::convert::TryFrom](https://doc.rust-lang.org/std/convert/trait.TryFrom.html).
