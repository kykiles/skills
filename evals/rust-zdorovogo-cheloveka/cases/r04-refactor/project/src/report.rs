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
    let mut out = String::new();
    out.push_str(&format!("=== {title} ===\n"));

    let mut critical: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
    let mut warning: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
    let mut info: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
    for entry in entries {
        match entry.level {
            Level::Critical => critical.entry(&entry.source).or_default().push(entry),
            Level::Warning => warning.entry(&entry.source).or_default().push(entry),
            Level::Info => info.entry(&entry.source).or_default().push(entry),
        }
    }

    if !critical.is_empty() {
        out.push_str(&format!("\n[{}]\n", Level::Critical));
        let mut total = 0;
        for (source, list) in &critical {
            let mut n = 0;
            let mut messages: Vec<&str> = Vec::new();
            for entry in list {
                n += entry.count;
                if !messages.contains(&entry.message.as_str()) {
                    messages.push(&entry.message);
                }
            }
            total += n;
            out.push_str(&format!(
                "  {:<12} {:>5}  {}\n",
                source,
                n,
                messages.join("; ")
            ));
        }
        out.push_str(&format!("  итого: {total}\n"));
    }

    if !warning.is_empty() {
        out.push_str(&format!("\n[{}]\n", Level::Warning));
        let mut total = 0;
        for (source, list) in &warning {
            let mut n = 0;
            let mut messages: Vec<&str> = Vec::new();
            for entry in list {
                n += entry.count;
                if !messages.contains(&entry.message.as_str()) {
                    messages.push(&entry.message);
                }
            }
            total += n;
            out.push_str(&format!(
                "  {:<12} {:>5}  {}\n",
                source,
                n,
                messages.join("; ")
            ));
        }
        out.push_str(&format!("  итого: {total}\n"));
    }

    if !info.is_empty() {
        out.push_str(&format!("\n[{}]\n", Level::Info));
        let mut total = 0;
        for (source, list) in &info {
            let mut n = 0;
            let mut messages: BTreeSet<&str> = BTreeSet::new();
            for entry in list {
                n += entry.count;
                messages.insert(&entry.message);
            }
            total += n;
            out.push_str(&format!(
                "  {:<12} {:>5}  ({} сообщ.)\n",
                source,
                n,
                messages.len()
            ));
        }
        out.push_str(&format!("  итого: {total}\n"));
    }

    let all: i64 = entries.iter().map(|entry| entry.count).sum();
    let sources: BTreeSet<&str> = entries.iter().map(|entry| entry.source.as_str()).collect();
    out.push_str(&format!(
        "\nвсего событий: {all}, источников: {}\n",
        sources.len()
    ));
    out
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
