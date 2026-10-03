use crate::{Order, Status};

/// Заказ ещё может перейти в другой статус.
pub fn is_open(status: Status) -> bool {
    !matches!(status, Status::Delivered | Status::Cancelled)
}

/// Покупатель может отменить заказ сам.
pub fn can_cancel(status: Status) -> bool {
    matches!(status, Status::New | Status::Paid)
}

/// Допустим ли переход из `from` в `to`.
pub fn can_move(from: Status, to: Status) -> bool {
    matches!(
        (from, to),
        (Status::New, Status::Paid)
            | (Status::Paid, Status::Shipped)
            | (Status::Shipped, Status::Delivered)
            | (Status::New | Status::Paid, Status::Cancelled)
            | (Status::Delivered, Status::Returned)
    )
}

/// Сколько копеек вернуть покупателю.
pub fn refund(order: &Order) -> u64 {
    match order.status {
        Status::Cancelled | Status::Returned => order.total,
        Status::New | Status::Paid | Status::Shipped | Status::Delivered => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle() {
        assert!(can_move(Status::New, Status::Paid));
        assert!(!can_move(Status::Delivered, Status::Paid));
        assert!(is_open(Status::Shipped));
        assert!(!is_open(Status::Cancelled));
        assert!(can_cancel(Status::Paid));
        assert!(!can_cancel(Status::Shipped));
    }
}
