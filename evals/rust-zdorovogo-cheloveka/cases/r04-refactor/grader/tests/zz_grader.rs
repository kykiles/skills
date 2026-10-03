// Скрытые тесты оценщика r04-refactor: вывод format_report совпадает
// с исходной реализацией байт в байт, сигнатуры публичных функций прежние.
use report::{format_csv, format_report, Entry, Level};
use std::collections::{BTreeMap, BTreeSet};

fn zz_original_format_report(title: &str, entries: &[Entry]) -> String {
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

// xorshift64*: тесту не нужны внешние крейты.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn random_entries(rng: &mut Rng) -> Vec<Entry> {
    const SOURCES: [&str; 7] = [
        "db",
        "api",
        "очередь",
        "",
        "a-very-long-source-name",
        "API",
        "db ",
    ];
    const MESSAGES: [&str; 6] = ["disk full", "timeout", "", "перезапуск", "timeout ", "x; y"];
    const LEVELS: [Level; 4] = [
        Level::Info,
        Level::Warning,
        Level::Critical,
        Level::Critical,
    ];
    if rng.below(10) == 0 {
        return Vec::new();
    }
    (0..rng.below(25))
        .map(|_| Entry {
            source: SOURCES[rng.below(SOURCES.len())].to_string(),
            level: LEVELS[rng.below(LEVELS.len())],
            message: MESSAGES[rng.below(MESSAGES.len())].to_string(),
            count: rng.below(201_000) as i64 - 1000,
        })
        .collect()
}

#[test]
fn grader_same_output_as_original() {
    let mut rng = Rng(20_261_003);
    for i in 0..3000 {
        let entries = random_entries(&mut rng);
        let title = format!("отчёт {i}");
        let want = zz_original_format_report(&title, &entries);
        let got = format_report(&title, &entries);
        assert_eq!(got, want, "случай {i}, entries: {entries:#?}");
    }
}

#[test]
fn grader_edge_cases() {
    let e = |source: &str, level, message: &str, count| Entry {
        source: source.to_string(),
        level,
        message: message.to_string(),
        count,
    };
    let cases = [
        vec![],
        vec![e("s", Level::Info, "a", 1), e("s", Level::Info, "a", 1)],
        vec![
            e("s", Level::Warning, "b", 1),
            e("s", Level::Warning, "a", 1),
            e("s", Level::Warning, "b", 1),
        ],
        vec![e("очень-длинное-имя-источника", Level::Critical, "x", -5)],
    ];
    for (i, entries) in cases.iter().enumerate() {
        assert_eq!(
            format_report("t", entries),
            zz_original_format_report("t", entries),
            "случай {i}"
        );
    }
}

#[test]
fn grader_public_signatures() {
    let _: fn(&str, &[Entry]) -> String = format_report;
    let _: fn(&[Entry]) -> String = format_csv;
    let entry = Entry {
        source: String::new(),
        level: Level::Info,
        message: String::new(),
        count: 0_i64,
    };
    let _ = (
        entry.clone() == entry,
        format!("{entry:?} {}", Level::Warning),
    );
}
