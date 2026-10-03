//! Учёт заказов интернет-магазина: статусы, правила и сводка.

mod policy;
mod report;
mod status;

pub use policy::{can_cancel, can_move, is_open, refund};
pub use report::summary;
pub use status::Status;

/// Заказ; сумма в копейках.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: u32,
    pub status: Status,
    pub total: u64,
}
