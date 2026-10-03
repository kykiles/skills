//! `orders < выгрузка.txt` — сводка по выгрузке заказов.
//! Формат строки выгрузки: `<id> <код статуса> <сумма в копейках>`.

use std::io::{self, BufRead};
use std::process::ExitCode;

use orders::{summary, Order, Status};

fn parse_line(line: &str) -> Option<Order> {
    let mut fields = line.split_whitespace();
    let id = fields.next()?.parse().ok()?;
    let status = Status::from_code(fields.next()?)?;
    let total = fields.next()?.parse().ok()?;
    if fields.next().is_some() {
        return None;
    }
    Some(Order { id, status, total })
}

fn main() -> ExitCode {
    let mut orders = Vec::new();
    for (index, line) in io::stdin().lock().lines().enumerate() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("ошибка чтения: {error}");
                return ExitCode::FAILURE;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let Some(order) = parse_line(&line) else {
            eprintln!("строка {}: не разобрана: {line}", index + 1);
            return ExitCode::FAILURE;
        };
        orders.push(order);
    }
    print!("{}", summary(&orders));
    ExitCode::SUCCESS
}
