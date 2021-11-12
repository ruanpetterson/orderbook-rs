use std::io;
use std::io::prelude::*;
use std::{fs::File, io::BufReader};

use clap::{App, Arg};
use log::{debug, info};
use orderbook::{Order, Orderbook};
use serde_json;

fn main() -> io::Result<()> {
    env_logger::init();

    // CLI configuration
    let matches = App::new("Example Orderbook")
        .version("0.1.0")
        .author("Ruan Petterson <ruan@ruan.eng.br>")
        .about("The simplest orderbook implementation")
        .arg(
            Arg::with_name("orders")
                .help("List of orders in JSON")
                .takes_value(true)
                .value_name("FILE"),
        )
        .get_matches();
    let input_file = matches.value_of("orders").unwrap_or("./orders.json");

    // Reading and parsing file
    debug!("Opening {}...", &input_file);
    let input_file = File::open(input_file)?;
    let mut buf_reader = BufReader::new(input_file);
    let mut contents = String::new();
    debug!("Reading contents..");
    buf_reader.read_to_string(&mut contents)?;
    let orders: Vec<Order> = serde_json::from_str(&contents).unwrap();

    // Generating data
    let mut orderbook = Orderbook::new();
    let mut trades = Vec::new();

    info!("Processing file...");
    for order in orders {
        if let Some(ref mut trade) = orderbook.transaction(order) {
            trades.append(trade);
        }
    }

    // Writing results
    info!("Trying to save results...");
    let mut orderbook_file = File::create("orderbook.json")?;
    let mut trades_file = File::create("trades.json")?;

    orderbook_file.write_all(serde_json::to_string_pretty(&orderbook)?.as_bytes())?;
    trades_file.write_all(serde_json::to_string_pretty(&trades)?.as_bytes())?;

    orderbook_file.sync_all()?;
    trades_file.sync_all()?;

    Ok(())
}
