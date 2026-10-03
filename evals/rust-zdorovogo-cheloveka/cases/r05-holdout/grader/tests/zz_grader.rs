// Скрытые тесты оценщика r05-holdout: результаты совпадают с исходной реализацией.
use logtop::{bytes_by_status, top_paths};
use std::collections::{BTreeMap, HashMap};

// Исходная реализация.
#[allow(clippy::all)]
mod original {
    use super::*;

    struct ZzRequest {
        path: String,
        status: u16,
        bytes: u64,
    }

    fn zz_parse_line(line: String) -> Option<ZzRequest> {
        let fields: Vec<String> = line.split_whitespace().map(|f| f.to_string()).collect();
        if fields.len() != 4 {
            return None;
        }
        Some(ZzRequest {
            path: fields[1].clone(),
            status: fields[2].parse().ok()?,
            bytes: fields[3].parse().ok()?,
        })
    }

    fn zz_requests(log: &str) -> Vec<ZzRequest> {
        let lines: Vec<String> = log.lines().map(|line| line.to_string()).collect();
        let mut out = Vec::new();
        for line in &lines {
            if let Some(request) = zz_parse_line(line.clone()) {
                out.push(request);
            }
        }
        out
    }

    /// `n` самых частых путей с числом запросов: по убыванию числа, при равенстве —
    /// по алфавиту.
    fn zz_top_paths(log: &str, n: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for request in zz_requests(log) {
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
    fn zz_bytes_by_status(log: &str) -> BTreeMap<u16, u64> {
        let mut totals = BTreeMap::new();
        for request in zz_requests(log) {
            *totals.entry(request.status).or_insert(0) += request.bytes;
        }
        totals
    }

    pub fn top(log: &str, n: usize) -> Vec<(String, usize)> {
        zz_top_paths(log, n)
    }

    pub fn bytes(log: &str) -> BTreeMap<u16, u64> {
        zz_bytes_by_status(log)
    }
}

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_f491_4f6c_dd1d) % n as u64) as usize
    }
}

fn random_log(rng: &mut Rng) -> String {
    const METHODS: [&str; 3] = ["GET", "POST", "PUT"];
    const PATHS: [&str; 6] = ["/", "/a", "/b", "/путь", "/a/b?x=1", "/B"];
    const STATUSES: [&str; 6] = ["200", "404", "500", "+200", "70000", "x"];
    const BYTES: [&str; 5] = ["0", "10", "4294967296", "-1", "5"];
    const SEPARATORS: [&str; 3] = [" ", "  ", "\t"];
    let mut log = String::new();
    for _ in 0..rng.below(40) {
        let fields = 3 + rng.below(3);
        let mut line: Vec<&str> = vec![
            METHODS[rng.below(3)],
            PATHS[rng.below(PATHS.len())],
            STATUSES[rng.below(STATUSES.len())],
            BYTES[rng.below(BYTES.len())],
            "extra",
        ];
        line.truncate(fields);
        for (i, field) in line.iter().enumerate() {
            if i > 0 {
                log.push_str(SEPARATORS[rng.below(3)]);
            }
            log.push_str(field);
        }
        log.push_str(if rng.below(5) == 0 { "\r\n" } else { "\n" });
    }
    log
}

#[test]
fn grader_same_results_as_original() {
    let mut rng = Rng(20_261_003);
    for i in 0..3000 {
        let log = random_log(&mut rng);
        let n = rng.below(8);
        assert_eq!(
            top_paths(&log, n),
            original::top(&log, n),
            "top_paths, случай {i}: {log:?}"
        );
        assert_eq!(
            bytes_by_status(&log),
            original::bytes(&log),
            "bytes_by_status, случай {i}: {log:?}"
        );
    }
}

#[test]
fn grader_signatures() {
    let _: fn(&str, usize) -> Vec<(String, usize)> = top_paths;
    let _: fn(&str) -> BTreeMap<u16, u64> = bytes_by_status;
    let _ = HashMap::<(), ()>::new();
}
