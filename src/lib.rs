#![feature(map_first_last)]

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::str::FromStr;

use bigdecimal::BigDecimal;

use log::{debug, info};
use serde::{Deserialize, Serialize};

mod util;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    CREATE,
    DELETE,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Criptocurrency {
    BTC,
    USDC,
}

impl Display for Criptocurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Criptocurrency {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BTC" => Ok(Self::BTC),
            "USDC" => Ok(Self::USDC),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "")),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pair(Criptocurrency, Criptocurrency);

impl Display for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

impl FromStr for Pair {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<_> = s.split("/").collect();

        if v.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "pair has incorrect syntax",
            ));
        }

        Ok(Pair(
            Criptocurrency::from_str(v.get(0).unwrap())?,
            Criptocurrency::from_str(v.get(1).unwrap())?,
        ))
    }
}

#[derive(Clone, Debug, Eq, Serialize, Deserialize)]
pub struct Order {
    type_op: Operation,
    #[serde(deserialize_with = "util::from_str")]
    account_id: usize,
    amount: RefCell<BigDecimal>,
    #[serde(deserialize_with = "util::from_str")]
    order_id: usize,
    #[serde(deserialize_with = "util::from_str")]
    pair: Pair,
    limit_price: BigDecimal,
    side: Side,
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.order_id == other.order_id
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.limit_price.cmp(&other.limit_price) {
            Ordering::Equal => self.order_id.cmp(&other.order_id),
            // Dirty trick to order
            ord => match self.side {
                Side::BUY => ord.reverse(),
                Side::SELL => ord,
            },
        }
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Order {
    pub fn new(
        type_op: Operation,
        account_id: usize,
        amount: BigDecimal,
        order_id: usize,
        pair: Pair,
        limit_price: BigDecimal,
        side: Side,
    ) -> Self {
        Order {
            type_op,
            account_id,
            amount: RefCell::new(amount),
            order_id,
            pair,
            limit_price,
            side,
        }
    }
}

#[derive(Serialize)]
pub struct Orderbook {
    buy: BTreeSet<Order>,
    sell: BTreeSet<Order>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Trade {
    buy_order: Order,
    sell_order: Order,
}

impl Trade {
    pub fn new(buy_order: Order, sell_order: Order) -> Self {
        Self {
            buy_order,
            sell_order,
        }
    }
}

impl Orderbook {
    pub fn new() -> Self {
        Orderbook {
            buy: BTreeSet::new(),
            sell: BTreeSet::new(),
        }
    }

    pub fn transaction(&mut self, order: Order) -> Option<Vec<Trade>> {
        match order.type_op {
            Operation::CREATE => self.insert(order),
            Operation::DELETE => self.delete(order),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buy.is_empty() && self.sell.is_empty()
    }

    // NEEDS REFACTORING
    // - several unnecessary allocations (.clone() everywhere)
    fn insert(&mut self, order: Order) -> Option<Vec<Trade>> {
        let mut trades: Vec<Trade> = Vec::new();
        info!("Received a {:?} order", order.side);
        debug!("{:?}", order);
        match order.side {
            Side::BUY => {
                loop {
                    if let Some(head) = self.sell.first() {
                        info!("Found a sell order.");
                        if head.limit_price <= order.limit_price {
                            info!("Found an order in limit price.");
                            match head.amount.cmp(&order.amount) {
                                Ordering::Less => {
                                    info!("Found an order with less amount than needed. Buying it all...");
                                    let (buy_order, sell_order) =
                                        (order.clone(), self.sell.pop_first().unwrap());
                                    order.amount.replace_with(|old| {
                                        old.clone() - sell_order.clone().amount.into_inner()
                                    });
                                    buy_order.amount.replace_with(|old| {
                                        old.clone() - order.amount.clone().into_inner()
                                    });
                                    info!("Trying to found another one...");
                                    trades.push(Trade::new(buy_order, sell_order));
                                }
                                Ordering::Equal => {
                                    info!("Found an order with same amount. Buying it all...");
                                    let (sell_order, buy_order) =
                                        (self.sell.pop_first().unwrap(), order);
                                    trades.push(Trade::new(buy_order, sell_order));
                                    return Some(trades);
                                }
                                Ordering::Greater => {
                                    info!(
                                    "Found an order with more amount than needed. Buying a little..."
                                );
                                    let (sell_order, buy_order) = (head.clone(), order.clone());
                                    sell_order.amount.replace(buy_order.amount.borrow().clone());
                                    head.amount.replace_with(|old| {
                                        old.clone() - order.amount.clone().into_inner()
                                    });
                                    trades.push(Trade::new(buy_order, sell_order));
                                    return Some(trades);
                                }
                            }
                        } else {
                            info!("Sell order below limit price. Inserting into orderbook...");
                            self.buy.insert(order.clone());

                            match trades.len() {
                                0 => return None,
                                _ => return Some(trades),
                            }
                        }
                    } else {
                        info!("Not found sell order to buy. Inserting into orderbook...");
                        self.buy.insert(order.clone());

                        match trades.len() {
                            0 => return None,
                            _ => return Some(trades),
                        }
                    }
                }
            }
            Side::SELL => loop {
                if let Some(head) = self.buy.first() {
                    info!("Found a buy order.");
                    if head.limit_price >= order.limit_price {
                        info!("Found an order in limit price.");
                        match head.amount.cmp(&order.amount) {
                            Ordering::Less => {
                                info!("Found an order with less amount than needed. Buy it all...");
                                let (sell_order, buy_order) =
                                    (self.buy.pop_first().unwrap(), order.clone());
                                order.amount.replace_with(|old| {
                                    old.clone() - sell_order.clone().amount.into_inner()
                                });
                                buy_order.amount.replace_with(|old| {
                                    old.clone() - order.amount.clone().into_inner()
                                });
                                info!("Trying to found another order...");
                                trades.push(Trade::new(buy_order, sell_order));
                            }
                            Ordering::Equal => {
                                info!("Found an order with same amount. Buying it all...");
                                let (buy_order, sell_order) =
                                    (self.buy.pop_first().unwrap(), order);
                                trades.push(Trade::new(buy_order, sell_order));
                                return Some(trades);
                            }
                            Ordering::Greater => {
                                info!(
                                    "Found an order with more amount than needed. Buying a little..."
                                );
                                let (buy_order, sell_order) = (head.clone(), order.clone());
                                sell_order.amount.replace(buy_order.amount.borrow().clone());
                                head.amount.replace_with(|old| {
                                    old.clone() - order.amount.clone().into_inner()
                                });
                                trades.push(Trade::new(buy_order, sell_order));
                                return Some(trades);
                            }
                        }
                    } else {
                        info!("Buy order below limit price. Inserting into orderbook...");
                        self.sell.insert(order.clone());

                        match trades.len() {
                            0 => return None,
                            _ => return Some(trades),
                        }
                    }
                } else {
                    info!("Not found buy order to sell. Inserting into orderbook...");
                    self.sell.insert(order.clone());

                    match trades.len() {
                        0 => return None,
                        _ => return Some(trades),
                    }
                }
            },
        }
    }

    fn delete(&mut self, order: Order) -> Option<Vec<Trade>> {
        match order.side {
            Side::BUY => self.buy.remove(&order),
            Side::SELL => self.sell.remove(&order),
        };

        None
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;

    use crate::{Criptocurrency, Operation, Order, Orderbook, Pair, Side, Trade};

    #[test]
    fn create_and_delete_order() {
        for side in [Side::SELL, Side::BUY] {
            let mut orderbook = Orderbook::new();

            let order_1_add: Order = Order::new(
                Operation::CREATE,
                1,
                BigDecimal::from_str("1.500").unwrap(),
                1,
                Pair(Criptocurrency::BTC, Criptocurrency::USDC),
                BigDecimal::from_str("5000.00").unwrap(),
                side,
            );
            let order_1_del: Order = Order::new(
                Operation::DELETE,
                1,
                BigDecimal::from_str("1.500").unwrap(),
                1,
                Pair(Criptocurrency::BTC, Criptocurrency::USDC),
                BigDecimal::from_str("5000.00").unwrap(),
                side,
            );

            assert!(orderbook.is_empty());
            orderbook.transaction(order_1_add);
            assert!(!orderbook.is_empty());
            orderbook.transaction(order_1_del);
            assert!(orderbook.is_empty());
        }
    }

    #[test]
    fn sell_and_buy_mirror_orders() {
        for (side_a, side_b) in [(Side::SELL, Side::BUY), (Side::BUY, Side::SELL)] {
            let mut orderbook = Orderbook::new();
            let mut trades: Vec<Trade> = vec![];

            let order_1_add: Order = Order::new(
                Operation::CREATE,
                1,
                BigDecimal::from_str("1.500").unwrap(),
                1,
                Pair(Criptocurrency::BTC, Criptocurrency::USDC),
                BigDecimal::from_str("5000.00").unwrap(),
                side_a,
            );
            let order_2_add: Order = Order::new(
                Operation::CREATE,
                1,
                BigDecimal::from_str("1.500").unwrap(),
                2,
                Pair(Criptocurrency::BTC, Criptocurrency::USDC),
                BigDecimal::from_str("5000.00").unwrap(),
                side_b,
            );

            assert!(orderbook.is_empty());
            assert!(trades.is_empty());

            assert!(orderbook.transaction(order_1_add).is_none());

            assert!(!orderbook.is_empty());
            assert!(trades.is_empty());

            trades.append(&mut orderbook.transaction(order_2_add).unwrap());

            assert!(orderbook.is_empty());
            assert!(!trades.is_empty());
        }
    }

    #[test]
    fn assymetrical_orders() {
        for (side_a, side_b) in [(Side::SELL, Side::BUY), (Side::BUY, Side::SELL)] {
            let mut orderbook = Orderbook::new();
            let mut trades: Vec<Trade> = vec![];

            let order_1_add: Order = Order::new(
                Operation::CREATE,
                1,
                BigDecimal::from_str("1.000").unwrap(),
                1,
                Pair(Criptocurrency::BTC, Criptocurrency::USDC),
                BigDecimal::from_str("5000.00").unwrap(),
                side_a,
            );
            let order_2_add: Order = Order::new(
                Operation::CREATE,
                1,
                BigDecimal::from_str("1.500").unwrap(),
                2,
                Pair(Criptocurrency::BTC, Criptocurrency::USDC),
                BigDecimal::from_str("5000.00").unwrap(),
                side_b,
            );
            let order_3_add: Order = Order::new(
                Operation::CREATE,
                1,
                BigDecimal::from_str("0.250").unwrap(),
                3,
                Pair(Criptocurrency::BTC, Criptocurrency::USDC),
                BigDecimal::from_str("5000.00").unwrap(),
                side_a,
            );
            let order_4_add: Order = Order::new(
                Operation::CREATE,
                1,
                BigDecimal::from_str("0.250").unwrap(),
                4,
                Pair(Criptocurrency::BTC, Criptocurrency::USDC),
                BigDecimal::from_str("5000.00").unwrap(),
                side_a,
            );

            assert!(orderbook.is_empty());
            assert!(trades.is_empty());

            assert!(orderbook.transaction(order_1_add).is_none());

            assert!(!orderbook.is_empty());
            assert!(trades.is_empty());

            trades.append(&mut orderbook.transaction(order_2_add).unwrap());

            assert!(!orderbook.is_empty());
            assert_eq!(trades.len(), 1);

            trades.append(&mut orderbook.transaction(order_3_add).unwrap());

            assert!(!orderbook.is_empty());
            assert_eq!(trades.len(), 2);

            trades.append(&mut orderbook.transaction(order_4_add).unwrap());

            assert!(orderbook.is_empty());
            assert_eq!(trades.len(), 3);
        }
    }

    #[test]
    fn mixed_orders() {
        let mut orderbook = Orderbook::new();
        let mut trades: Vec<Trade> = vec![];

        let order_1_add: Order = Order::new(
            Operation::CREATE,
            1,
            BigDecimal::from_str("1.000").unwrap(),
            1,
            Pair(Criptocurrency::BTC, Criptocurrency::USDC),
            BigDecimal::from_str("4000.00").unwrap(),
            Side::SELL,
        );
        let order_2_add: Order = Order::new(
            Operation::CREATE,
            1,
            BigDecimal::from_str("1.000").unwrap(),
            2,
            Pair(Criptocurrency::BTC, Criptocurrency::USDC),
            BigDecimal::from_str("3000.00").unwrap(),
            Side::SELL,
        );
        let order_3_add: Order = Order::new(
            Operation::CREATE,
            1,
            BigDecimal::from_str("1.000").unwrap(),
            3,
            Pair(Criptocurrency::BTC, Criptocurrency::USDC),
            BigDecimal::from_str("6000.00").unwrap(),
            Side::SELL,
        );
        let order_4_add: Order = Order::new(
            Operation::CREATE,
            1,
            BigDecimal::from_str("2.000").unwrap(),
            4,
            Pair(Criptocurrency::BTC, Criptocurrency::USDC),
            BigDecimal::from_str("5000.00").unwrap(),
            Side::BUY,
        );
        let order_5_add: Order = Order::new(
            Operation::CREATE,
            1,
            BigDecimal::from_str("1.000").unwrap(),
            5,
            Pair(Criptocurrency::BTC, Criptocurrency::USDC),
            BigDecimal::from_str("6000.00").unwrap(),
            Side::BUY,
        );
        let order_6_add: Order = Order::new(
            Operation::CREATE,
            1,
            BigDecimal::from_str("1.000").unwrap(),
            6,
            Pair(Criptocurrency::BTC, Criptocurrency::USDC),
            BigDecimal::from_str("6000.00").unwrap(),
            Side::BUY,
        );
        let order_7_add: Order = Order::new(
            Operation::CREATE,
            1,
            BigDecimal::from_str("1.500").unwrap(),
            7,
            Pair(Criptocurrency::BTC, Criptocurrency::USDC),
            BigDecimal::from_str("5000.00").unwrap(),
            Side::SELL,
        );
        let order_8_add: Order = Order::new(
            Operation::CREATE,
            1,
            BigDecimal::from_str("0.500").unwrap(),
            8,
            Pair(Criptocurrency::BTC, Criptocurrency::USDC),
            BigDecimal::from_str("5000.00").unwrap(),
            Side::BUY,
        );

        assert!(orderbook.is_empty());
        assert!(trades.is_empty());

        assert!(orderbook.transaction(order_1_add).is_none());
        assert!(orderbook.transaction(order_2_add).is_none());
        assert!(orderbook.transaction(order_3_add).is_none());

        assert!(!orderbook.is_empty());
        assert!(trades.is_empty());

        trades.append(&mut orderbook.transaction(order_4_add).unwrap());

        assert!(!orderbook.is_empty());
        assert_eq!(trades.len(), 2);

        trades.append(&mut orderbook.transaction(order_5_add).unwrap());

        assert!(orderbook.is_empty());
        assert_eq!(trades.len(), 3);

        assert!(orderbook.transaction(order_6_add).is_none());
        assert!(!orderbook.is_empty());

        trades.append(&mut orderbook.transaction(order_7_add).unwrap());
        assert!(!orderbook.is_empty());
        assert_eq!(trades.len(), 4);

        trades.append(&mut orderbook.transaction(order_8_add).unwrap());
        assert!(orderbook.is_empty());
        assert_eq!(trades.len(), 5);
    }
}
