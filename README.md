# skills

Мои скиллы для Claude и Codex (формат Agent Skills). Один скилл — один каталог с `SKILL.md`; подробности, которые нужны не в каждой задаче, — в его `references/`.

| Скилл | Для чего |
|-------|----------|
| [`rust-zdorovogo-cheloveka`](rust-zdorovogo-cheloveka/SKILL.md) | Писать, чинить, рефакторить и тестировать Rust-код минимальными правками. |
| [`go-zdorovogo-cheloveka`](go-zdorovogo-cheloveka/SKILL.md) | Писать, чинить, рефакторить и тестировать Go-код минимальными правками; подробности по темам — в `references/`. |

Испытания скиллов и их результаты — в [`evals/`](evals/), заметки о разработке — в [`docs/`](docs/).

## Добавить скилл

1. Создать каталог `<имя-скилла>/` с файлом `SKILL.md` (в frontmatter — `name` и `description`).
2. Добавить строку в таблицу выше.
