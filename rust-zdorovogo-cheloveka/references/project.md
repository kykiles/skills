# Проект: зависимости, MSRV, форматирование, Clippy и линты

## Зависимости

- Не обновляй edition, MSRV, зависимости, lockfile или соседние модули «заодно». Не добавляй крейты (`thiserror`, `anyhow` и т. п.) без просьбы пользователя, если их ещё нет в проекте.
- Модели выдумывают и путают имена крейтов, и такое имя может оказаться занято чужим пакетом. Перед добавлением зависимости проверь её через `cargo info <крейт>` или страницу на crates.io: крейт существует, его репозиторий и описание соответствуют задаче, у него есть история выпусков и загрузок. В отчёте назови каждую новую зависимость и причину.
- API зависимости не угадывай по памяти: модели путают версии, имена методов и features. Посмотри версию в `Cargo.lock` и сверь сигнатуры и необходимые features с исходниками этой версии (`~/.cargo/registry/src/*/<крейт>-<версия>/`) или с `cargo doc -p <крейт>`.
- Если правка меняет зависимости, а в проекте настроены `cargo deny` или `cargo audit` (есть `deny.toml` или вызов в CI), запусти их.

## MSRV

Если проект заявляет MSRV (`rust-version`) и нужный компилятор установлен, проверь совместимость: `cargo +<rust-version> check -p <пакет> --locked`. Успешная сборка на более новой версии совместимости не доказывает; не устанавливай toolchain без согласия пользователя — просто сообщи, что проверка не проведена. Частичная замена на текущем компиляторе — Clippy: линт `incompatible_msrv` (включён по умолчанию) берёт `rust-version` из манифеста и предупреждает о функциях и типах std новее неё, но языковые конструкции (`let … else`, новый синтаксис) не проверяет.

## Сборка, форматирование и Clippy

Если `Cargo.lock` уже есть и зависимости не меняются, используй `--locked` при Cargo-проверках. Форматируй и проверяй только свои файлы: `rustfmt --edition <edition> <файлы>`, `rustfmt --edition <edition> --check <файлы>`. `cargo fmt --all` запускай, только если `git status` до правки был чистым: иначе он перепишет чужие незавершённые правки. При уместности запусти Clippy для затронутого пакета (`cargo clippy -p <пакет> --all-targets`). Не выдавай предупреждения, существовавшие до правки, за результат своей работы и не считай их своими. Форматирование поведения не меняет, поэтому после `cargo fmt` тесты не повторяй; после `cargo clippy --fix` просмотри diff и повтори тесты, если исправление затронуло логику. Cargo может долго ждать блокировку каталога сборки (`Blocking waiting for file lock`) — это нормально, дождись завершения команды.

## Линты в конфигурации проекта

Правило, которое умеет проверять компилятор или Clippy, надёжнее закрепить в `[lints]`/`clippy.toml`, чем в тексте. Если такое правило раз за разом нарушается, предложи пользователю линт — это изменение конфигурации проекта, а не часть текущей правки.

## Подавление линтов

**Принцип:** подавляй линт на самом узком участке (выражение, функция, а не модуль) атрибутом `#[expect(lint, reason = "...")]` (Rust 1.81+). В отличие от `#[allow]`, `expect` сам предупредит, когда подавление станет лишним, а `reason` объясняет читателю, почему правило здесь не действует.

```rust
#[expect(clippy::cast_possible_truncation, reason = "бокс собирается из заголовков и меньше 4 ГиБ")]
fn box_len(body: &[u8]) -> u32 {
    (body.len() + 8) as u32
}
```

**Проверка выбора:** сначала попробуй исправить код так, чтобы линт не срабатывал; подавление — для случаев, когда линт ошибается или правило здесь неприменимо. Если `rust-version` проекта ниже 1.81, используй `#[allow(...)]` с комментарием о причине. Предупреждение компилятора — тоже сигнал: исправь причину, а не подавляй её.

## Источники

- Проект и инструменты: [Cargo Manifest](https://doc.rust-lang.org/cargo/reference/manifest.html), [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html), [Rust version](https://doc.rust-lang.org/cargo/reference/rust-version.html), [Clippy — incompatible_msrv](https://rust-lang.github.io/rust-clippy/stable/index.html#incompatible_msrv), [rustfmt](https://github.com/rust-lang/rustfmt#verifying-code-is-formatted), [Clippy](https://doc.rust-lang.org/clippy/usage.html).
- Зависимости: [cargo info](https://doc.rust-lang.org/cargo/commands/cargo-info.html), [cargo-deny](https://embarkstudios.github.io/cargo-deny/), [cargo-audit и RustSec](https://rustsec.org/), [Spracklen et al. — галлюцинации имён пакетов в коде от LLM](https://arxiv.org/abs/2406.10279).
- Линты: [Rust 1.81 — `#[expect]` и `reason`](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/#expect-lint), [Clippy — allow_attributes_without_reason](https://rust-lang.github.io/rust-clippy/stable/index.html#allow_attributes_without_reason), [Cargo — `[lints]`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section).
- Практика крупного Rust-проекта с агентами: [openai/codex — AGENTS.md](https://github.com/openai/codex/blob/main/AGENTS.md) и [workspace-линты codex-rs](https://github.com/openai/codex/blob/main/codex-rs/Cargo.toml).
