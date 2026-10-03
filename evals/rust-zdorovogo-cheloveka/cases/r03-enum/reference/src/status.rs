/// Статус заказа.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Создан, не оплачен.
    New,
    /// Оплачен, ждёт отправки.
    Paid,
    /// Передан в доставку.
    Shipped,
    /// Получен покупателем.
    Delivered,
    /// Возвращён покупателем после доставки.
    Returned,
    /// Отменён до отправки.
    Cancelled,
}

impl Status {
    /// Все статусы в порядке жизненного цикла.
    pub const ALL: [Status; 6] = [
        Status::New,
        Status::Paid,
        Status::Shipped,
        Status::Delivered,
        Status::Returned,
        Status::Cancelled,
    ];

    /// Код статуса в выгрузке и в CLI.
    pub fn code(self) -> &'static str {
        match self {
            Status::New => "new",
            Status::Paid => "paid",
            Status::Shipped => "shipped",
            Status::Delivered => "delivered",
            Status::Returned => "returned",
            Status::Cancelled => "cancelled",
        }
    }

    /// Статус по коду из выгрузки.
    pub fn from_code(code: &str) -> Option<Status> {
        match code {
            "new" => Some(Status::New),
            "paid" => Some(Status::Paid),
            "shipped" => Some(Status::Shipped),
            "delivered" => Some(Status::Delivered),
            "returned" => Some(Status::Returned),
            "cancelled" => Some(Status::Cancelled),
            _ => None,
        }
    }

    /// Подпись в сводке.
    pub fn title(self) -> &'static str {
        match self {
            Status::New => "новый",
            Status::Paid => "оплачен",
            Status::Shipped => "в доставке",
            Status::Delivered => "доставлен",
            Status::Returned => "возвращён",
            Status::Cancelled => "отменён",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        for status in Status::ALL {
            assert_eq!(Status::from_code(status.code()), Some(status));
        }
        assert_eq!(Status::from_code("lost"), None);
    }
}
