use std::fmt::Write;

use crate::{is_open, refund, Order, Status};

/// Сводка по заказам: число заказов по статусам, открытые и закрытые,
/// выручка и возвраты в рублях.
pub fn summary(orders: &[Order]) -> String {
    let mut out = String::new();
    for status in Status::ALL {
        let count = orders.iter().filter(|order| order.status == status).count();
        writeln!(out, "{}: {count}", status.title()).unwrap();
    }

    let open = orders.iter().filter(|order| is_open(order.status)).count();
    let revenue: u64 = orders
        .iter()
        .filter(|order| {
            matches!(
                order.status,
                Status::Paid | Status::Shipped | Status::Delivered
            )
        })
        .map(|order| order.total)
        .sum();
    let refunds: u64 = orders.iter().map(refund).sum();

    writeln!(out, "открытых: {open}, закрытых: {}", orders.len() - open).unwrap();
    writeln!(out, "выручка: {}", rubles(revenue)).unwrap();
    writeln!(out, "возвраты: {}", rubles(refunds)).unwrap();
    out
}

fn rubles(kopecks: u64) -> String {
    format!("{}.{:02} ₽", kopecks / 100, kopecks % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts() {
        let orders = [
            Order {
                id: 1,
                status: Status::Paid,
                total: 15000,
            },
            Order {
                id: 2,
                status: Status::Cancelled,
                total: 9950,
            },
            Order {
                id: 3,
                status: Status::Delivered,
                total: 100000,
            },
        ];
        assert_eq!(
            summary(&orders),
            "новый: 0\nоплачен: 1\nв доставке: 0\nдоставлен: 1\nвозвращён: 0\nотменён: 1\n\
             открытых: 1, закрытых: 2\nвыручка: 1150.00 ₽\nвозвраты: 99.50 ₽\n"
        );
    }
}
