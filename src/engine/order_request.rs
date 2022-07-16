use super::{Order, OrderId};
use crate::OrderSide;

use compact_str::CompactString;
use rust_decimal::{prelude::ToPrimitive, Decimal};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderRequestError {
    #[error("order type mismatch")]
    MismatchType,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type_op", rename_all = "UPPERCASE"))]
pub enum OrderRequest {
    Create {
        account_id: CompactString,
        amount: Decimal,
        order_id: CompactString,
        pair: CompactString,
        limit_price: Decimal,
        side: OrderSide,
    },
    Delete {
        order_id: CompactString,
    },
}

impl TryFrom<OrderRequest> for Order {
    type Error = OrderRequestError;

    #[inline]
    fn try_from(order_request: OrderRequest) -> Result<Self, Self::Error> {
        match order_request {
            OrderRequest::Create {
                account_id,
                amount,
                limit_price,
                side,
                ..
            } => Ok(Order::new(
                OrderId::new(u64::from_str_radix(&account_id, 10).unwrap()),
                u64::from_str_radix(&account_id, 10).unwrap(),
                side,
                limit_price.trunc().to_u64().unwrap() * 100
                    + limit_price.fract().to_u64().unwrap(),
                amount.trunc().to_u64().unwrap() * 100
                    + amount.fract().to_u64().unwrap(),
            )),
            OrderRequest::Delete { .. } => Err(OrderRequestError::MismatchType),
        }
    }
}
