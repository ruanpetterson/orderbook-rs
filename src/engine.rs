use crate::{Asset, Exchange, Order, OrderId, OrderRequest, Orderbook};

pub struct Engine {
    orderbook: Orderbook<Order>,
}

impl Engine {
    pub fn new(pair: &str) -> Self {
        Self {
            orderbook: Orderbook::new(pair),
        }
    }

    pub fn process(
        &mut self,
        incoming_order: OrderRequest,
    ) -> Vec<<Orderbook<Order> as Exchange>::Event> {
        let mut events = Vec::with_capacity(8);
        match incoming_order {
            OrderRequest::Create { .. } => {
                let order = Order::try_from(incoming_order).unwrap();
                events.append(&mut self.orderbook.matching(order));
            }
            OrderRequest::Delete { ref order_id } => {
                self.orderbook.remove(&OrderId::new(
                    u64::from_str_radix(order_id, 10).unwrap(),
                ));
            }
        };

        // TODO: Generate events
        events
    }
}
