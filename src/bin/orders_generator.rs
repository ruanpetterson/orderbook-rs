use std::io::Result;

const _N: usize = 5_000_000;

fn main() -> Result<()> {
    // TODO: Fix OrderRequest
    /*
    let mut rng = rand::thread_rng();
    let mut orders = Vec::with_capacity(N);

    for i in 1..=N {
        let order = match rng.gen_range(0..2) {
            0 => OrderRequest::Delete {
                order_id: OrderId::new(rng.gen_range(1..=i as u64)),
            },
            _ => OrderRequest::Create {
                account_id: rng.gen_range(1..10),
                amount: rng.gen_range(1000..2000),
                order_id: OrderId::new(i as u64),
                pair: CompactString::new_inline("BTC/USDC"),
                limit_price: rng.gen_range(1000..2000),
                order_side: match rng.gen_range(0..2) {
                    0 => OrderSide::Ask,
                    _ => OrderSide::Bid,
                },
            },
        };

        orders.push(order);
    }

    let content = serde_json::to_string(&orders)?;
    let path = Path::new("./orders.json");

    std::fs::write(&path, content)?;
    */

    Ok(())
}
