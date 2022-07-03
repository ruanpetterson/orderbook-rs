#![allow(dead_code, unused)]
#![feature(map_first_last)]
#![feature(const_btree_new)]

#[cfg(test)]
mod tests;

mod engine;
mod internals;
mod order;
mod order_id;
mod order_request;
mod order_side;
mod order_status;
mod orderbook;
mod trade;

pub use crate::engine::Engine;
pub use crate::internals::{Asset, Exchange, Opposite};
pub use crate::order::{AskOrder, BidOrder, Order};
pub use crate::order_id::OrderId;
pub use crate::order_request::OrderRequest;
pub use crate::order_side::OrderSide;
pub use crate::order_status::OrderStatus;
pub use crate::orderbook::Orderbook;
pub use crate::trade::Trade;
