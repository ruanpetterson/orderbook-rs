use compact_str::CompactString;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{Asset, Exchange, Order, Orderbook};

const PAIR: CompactString = CompactString::new_inline("BTC/USDC");
const MOCK_SIZE: usize = 6;
const ORDERS: Lazy<[Order; MOCK_SIZE]> = Lazy::new(|| {
    let input = include_str!("./mock_orders.json");
    serde_json::from_str(input)
        .expect("a set of valid orders with MOCK_SIZE length")
});

#[test]
fn simple_match() {
    let mut ask = ORDERS[0];
    let mut bid = ORDERS[1];

    assert!(ask.trade(&mut bid).is_some());
    assert!(ask.is_closed());
    assert!(bid.is_closed());
}

#[test]
fn orderbook() {
    let mut orderbook = Orderbook::<Order>::new(&PAIR);

    assert_eq!(orderbook.matching(ORDERS[0]).len(), 1);
    assert_eq!(orderbook.matching(ORDERS[1]).len(), 1);
    assert_eq!(orderbook.matching(ORDERS[2]).len(), 1);
    assert_eq!(orderbook.matching(ORDERS[3]).len(), 2);
    assert_eq!(orderbook.matching(ORDERS[4]).len(), 1);
    assert_eq!(orderbook.matching(ORDERS[5]).len(), 2);
}
