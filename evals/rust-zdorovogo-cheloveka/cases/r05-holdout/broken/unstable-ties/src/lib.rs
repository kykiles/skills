//! Статистика по access-логу.
//!
//! Строка лога: `<метод> <путь> <код ответа> <байт в ответе>`, поля разделены
//! пробелами. Строки другого вида пропускаются.

use std::collections::{BTreeMap, HashMap};

struct Request<'a> {
    path: &'a str,
    status: u16,
    bytes: u64,
}

fn parse_line(line: &str) -> Option<Request<'_>> {
    let mut fields = line.split_whitespace();
    let (Some(_method), Some(path), Some(status), Some(bytes), None) = (
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
    ) else {
        return None;
    };
    Some(Request {
        path,
        status: status.parse().ok()?,
        bytes: bytes.parse().ok()?,
    })
}

fn requests(log: &str) -> impl Iterator<Item = Request<'_>> {
    log.lines().filter_map(parse_line)
}

/// `n` самых частых путей с числом запросов: по убыванию числа, при равенстве —
/// по алфавиту.
pub fn top_paths(log: &str, n: usize) -> Vec<(String, usize)> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for request in requests(log) {
        *counts.entry(request.path).or_insert(0) += 1;
    }
    let mut sorted: Vec<(&str, usize)> = counts.into_iter().collect();
    sorted.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
    sorted
        .into_iter()
        .take(n)
        .map(|(path, count)| (path.to_string(), count))
        .collect()
}

/// Сумма байт в ответах по каждому коду ответа.
pub fn bytes_by_status(log: &str) -> BTreeMap<u16, u64> {
    let mut totals = BTreeMap::new();
    for request in requests(log) {
        *totals.entry(request.status).or_insert(0) += request.bytes;
    }
    totals
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "GET /a 200 10\nGET /b 404 0\nbroken line\nPOST /a 200 5\nGET /c 500 x\n";

    #[test]
    fn top() {
        assert_eq!(
            top_paths(LOG, 5),
            vec![("/a".to_string(), 2), ("/b".to_string(), 1)]
        );
    }

    #[test]
    fn bytes() {
        assert_eq!(bytes_by_status(LOG), BTreeMap::from([(200, 15), (404, 0)]));
    }
}
