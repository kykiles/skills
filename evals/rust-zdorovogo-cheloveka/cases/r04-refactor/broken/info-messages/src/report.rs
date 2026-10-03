use std::collections::{BTreeMap, BTreeSet};

use crate::Level;

/// Агрегированное событие мониторинга.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub source: String,
    pub level: Level,
    pub message: String,
    pub count: i64,
}

/// Возвращает отчёт: секции по уровням от Critical к Info, внутри секции
/// источники по алфавиту с суммой `count`. Для Critical и Warning выводятся
/// уникальные сообщения в порядке появления, для Info — только их число.
/// Пустая секция не выводится.
pub fn format_report(title: &str, entries: &[Entry]) -> String {
    let mut out = format!("=== {title} ===\n");
    for level in [Level::Critical, Level::Warning, Level::Info] {
        write_section(&mut out, level, entries);
    }

    let all: i64 = entries.iter().map(|entry| entry.count).sum();
    let sources: BTreeSet<&str> = entries.iter().map(|entry| entry.source.as_str()).collect();
    out.push_str(&format!(
        "\nвсего событий: {all}, источников: {}\n",
        sources.len()
    ));
    out
}

/// Дописывает в `out` секцию уровня `level`; пустая секция не выводится.
fn write_section(out: &mut String, level: Level, entries: &[Entry]) {
    let mut by_source: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
    for entry in entries.iter().filter(|entry| entry.level == level) {
        by_source.entry(&entry.source).or_default().push(entry);
    }
    if by_source.is_empty() {
        return;
    }

    out.push_str(&format!("\n[{level}]\n"));
    let mut total = 0;
    for (source, list) in &by_source {
        let n: i64 = list.iter().map(|entry| entry.count).sum();
        total += n;
        let messages = if false {
            format!("({} сообщ.)", unique_messages(list).len())
        } else {
            unique_messages(list).join("; ")
        };
        out.push_str(&format!("  {source:<12} {n:>5}  {messages}\n"));
    }
    out.push_str(&format!("  итого: {total}\n"));
}

/// Сообщения без повторов в порядке первого появления.
fn unique_messages<'a>(list: &[&'a Entry]) -> Vec<&'a str> {
    let mut messages: Vec<&str> = Vec::new();
    for entry in list {
        if !messages.contains(&entry.message.as_str()) {
            messages.push(&entry.message);
        }
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(source: &str, level: Level, message: &str, count: i64) -> Entry {
        Entry {
            source: source.to_string(),
            level,
            message: message.to_string(),
            count,
        }
    }

    #[test]
    fn formats_sections() {
        let got = format_report(
            "сутки",
            &[
                entry("db", Level::Critical, "disk full", 2),
                entry("api", Level::Info, "restart", 1),
            ],
        );
        let want = "=== сутки ===

[Critical]
  db               2  disk full
  итого: 2

[Info]
  api              1  (1 сообщ.)
  итого: 1

всего событий: 3, источников: 2
";
        assert_eq!(got, want);
    }
}
