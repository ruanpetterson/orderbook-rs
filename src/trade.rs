use super::OrderId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[cfg_attr(test, derive(Copy, Clone))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Trade {
    pub(super) taker: OrderId,
    pub(super) maker: OrderId,
    pub(super) amount: u64,
    pub(super) price: u64,
}
