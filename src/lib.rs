#![feature(map_first_last)]
#![feature(const_btree_new)]

#[cfg(test)]
mod tests;

mod internals;
pub use crate::internals::{
    Asset, Exchange, ExchangeEvent, ExchangeExt, Opposite,
};

mod order_side;
pub use crate::order_side::OrderSide;

mod orderbook;
pub use crate::orderbook::Orderbook;

pub mod engine;
