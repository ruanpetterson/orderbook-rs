use std::borrow::Borrow;
use std::cmp::Reverse;
use std::collections::{BTreeMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

use compact_str::CompactString;
use indexmap::IndexMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Asset, Exchange, Opposite, Order, OrderId, OrderSide, Trade};

pub struct Orderbook<Order: Asset> {
    pair: CompactString,
    orders: IndexMap<<Order as Asset>::OrderId, Order>,
    ask: BTreeMap<u64, VecDeque<<Order as Asset>::OrderId>>,
    bid: BTreeMap<Reverse<u64>, VecDeque<<Order as Asset>::OrderId>>,
}

impl<Order> Orderbook<Order>
where
    Order: Asset,
{
    #[inline]
    pub fn new(pair: &str) -> Self {
        Self {
            pair: CompactString::new_inline(pair),
            orders: IndexMap::new(),
            ask: BTreeMap::new(),
            bid: BTreeMap::new(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum OrderbookEvent<Order: Asset>
where
    <Order as Asset>::Trade: Serialize,
    <Order as Asset>::OrderId: Serialize,
{
    Added(<Order as Asset>::OrderId),
    Removed(<Order as Asset>::OrderId),
    Trade(<Order as Asset>::Trade),
}

impl<Order> From<Trade> for OrderbookEvent<Order>
where
    Order: Asset<Trade = Trade>,
    <Order as Asset>::OrderId: Serialize,
{
    #[inline]
    fn from(trade: Trade) -> Self {
        OrderbookEvent::Trade(trade)
    }
}

// TODO: impl remove event as well.
impl<Order> From<OrderId> for OrderbookEvent<Order>
where
    Order: Asset<OrderId = OrderId>,
    <Order as Asset>::Trade: Serialize,
{
    #[inline]
    fn from(order_id: OrderId) -> Self {
        OrderbookEvent::Added(order_id)
    }
}

impl<Order> Exchange for Orderbook<Order>
where
    Order: Debug,
    Order: Asset<OrderSide = OrderSide>,
    Order: Asset<Trade = Trade>,
    <Order as Asset>::OrderId: Hash,
    <Order as Asset>::OrderId: Serialize,
{
    type Order = Order;
    type Event = OrderbookEvent<Self::Order>;

    #[inline]
    fn insert(&mut self, order: Self::Order) {
        let level = match order.side() {
            OrderSide::Ask => self
                .ask
                .entry(order.limit_price())
                .or_insert_with(|| VecDeque::with_capacity(8)),
            OrderSide::Bid => self
                .bid
                .entry(Reverse(order.limit_price()))
                .or_insert_with(|| VecDeque::with_capacity(8)),
        };

        level.push_back(order.id());

        self.orders.insert(order.id(), order);
    }

    #[inline]
    fn remove(
        &mut self,
        order_id: &<Self::Order as Asset>::OrderId,
    ) -> Option<Self::Order> {
        let mut order = self.orders.remove(order_id)?;
        let level = match order.side() {
            OrderSide::Ask => self.ask.get_mut(&order.limit_price())?,
            OrderSide::Bid => {
                self.bid.get_mut(&Reverse(order.limit_price()))?
            }
        };
        let index = level.iter().position(|&o| o == order.id())?;
        level.remove(index);

        order.cancel();
        Some(order)
    }

    #[inline]
    fn peek(&self, side: &OrderSide) -> Option<&Self::Order> {
        let order_id = match side {
            OrderSide::Ask => {
                self.ask.first_key_value().map(|(_, level)| level)?
            }
            OrderSide::Bid => {
                self.bid.first_key_value().map(|(_, level)| level)?
            }
        }
        .front()?;
        self.orders.get(order_id)
    }

    #[inline]
    fn peek_mut(&mut self, side: &OrderSide) -> Option<&mut Self::Order> {
        let order_id = match side {
            OrderSide::Ask => {
                self.ask.first_key_value().map(|(_, level)| level)?
            }
            OrderSide::Bid => {
                self.bid.first_key_value().map(|(_, level)| level)?
            }
        }
        .front()?;
        self.orders.get_mut(order_id)
    }

    #[inline]
    fn pop(&mut self, side: &OrderSide) -> Option<Self::Order> {
        let order_id = match side {
            OrderSide::Ask => {
                let mut level = self.ask.first_entry()?;
                if level.get().len() == 1 {
                    level.remove().pop_front()
                } else {
                    level.get_mut().pop_front()
                }
            }
            OrderSide::Bid => {
                let mut level = self.bid.first_entry()?;
                if level.get().len() == 1 {
                    level.remove().pop_front()
                } else {
                    level.get_mut().pop_front()
                }
            }
        }?;
        self.orders.remove(&order_id)
    }
}
