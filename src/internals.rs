use std::hash::Hash;

pub trait Exchange
where
    <<Self as Exchange>::Order as Asset>::OrderId: Hash,
{
    type Order: Asset;
    type Event: From<<Self::Order as Asset>::Trade>;

    fn insert(&mut self, order: Self::Order, new: bool);
    fn remove(
        &mut self,
        order: &<Self::Order as Asset>::OrderId,
    ) -> Option<Self::Order>;
    fn matching(
        &mut self,
        order: Self::Order,
    ) -> Vec<<Self as Exchange>::Event> {
        let mut events = Vec::with_capacity(8);
        let mut incoming_order = order;
        while let (false, Some(mut top_order)) = (
            incoming_order.is_closed(),
            self.pop(&incoming_order.side().opposite()),
        ) {
            if let Some(trade) = incoming_order.trade(&mut top_order) {
                events.push(trade.into());
                match (incoming_order.is_closed(), top_order.is_closed()) {
                    (true, _) => {
                        // Top order should go back if it do not become
                        // completed
                        if !top_order.is_closed() {
                            self.insert(top_order, false);
                        }
                        break;
                    }
                    (false, true) => continue,
                    (false, false) => unreachable!(),
                }
            } else {
                // Top order should go back if it do not become completed
                if !top_order.is_closed() {
                    self.insert(top_order, false);
                }
                break;
            }
        }

        if !incoming_order.is_closed() {
            self.insert(incoming_order, true);
        }

        events
    }
    fn peek(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<&Self::Order>;
    fn pop(
        &mut self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<Self::Order>;
}

pub trait Asset<Order = Self>: Ord + Eq {
    /// Order unique identifier.
    type OrderId: Eq + Copy + Clone;
    /// Order current status.
    type OrderStatus: Eq + Copy + Clone;
    /// Order side.
    type OrderSide: Opposite;
    /// Trade struct.
    type Trade;
    /// Return order unique identifier.
    fn id(&self) -> Self::OrderId;
    /// Return order side.
    fn side(&self) -> Self::OrderSide;
    /// Return order limit price.
    fn limit_price(&self) -> u64;
    /// Return order remaining amount.
    fn remaining(&self) -> u64;
    /// Return current order status.
    fn status(&self) -> Self::OrderStatus;
    fn is_closed(&self) -> bool;
    fn trade(&mut self, order: &mut Order) -> Option<Self::Trade>;
    fn cancel(&mut self);
}

pub trait Opposite<Opposite = Self> {
    fn opposite(&self) -> Opposite;
}
