//! Статистика по access-логу.
//!
//! Строка лога: `<метод> <путь> <код ответа> <байт в ответе>`, поля разделены
//! пробелами. Строки другого вида пропускаются.

use std::collections::{BTreeMap, HashMap};

struct Request {
    path: String,
    status: u16,
    bytes: u64,
}

fn parse_line(line: String) -> Option<Request> {
    let fields: Vec<String> = line.split_whitespace().map(|f| f.to_string()).collect();
    if fields.len() != 4 {
        return None;
    }
    Some(Request {
        path: fields[1].clone(),
        status: fields[2].parse().ok()?,
        bytes: fields[3].parse().ok()?,
    })
}

fn requests(log: &str) -> Vec<Request> {
    let lines: Vec<String> = log.lines().map(|line| line.to_string()).collect();
    let mut out = Vec::new();
    for line in &lines {
        if let Some(request) = parse_line(line.clone()) {
            out.push(request);
        }
    }
    out
}

/// `n` самых частых путей с числом запросов: по убыванию числа, при равенстве —
/// по алфавиту.
pub fn top_paths(log: &str, n: usize) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for request in requests(log) {
        *counts.entry(request.path.clone()).or_insert(0) += 1;
    }
    let mut sorted: Vec<(String, usize)> = counts
        .iter()
        .map(|(path, count)| (path.clone(), *count))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    sorted.truncate(n);
    sorted
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
