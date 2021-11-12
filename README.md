## Example Orderbook 

Usage:

    Example Orderbook 0.1.0
    Ruan Petterson <ruan@ruan.eng.br>
    The simplest orderbook implementation
    
    USAGE:
        orderbook [FILE]
    
    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    
    ARGS:
        <FILE>    List of orders in JSON

Example JSON:

    [
        {
            "type_op":"CREATE",
            "account_id":"1",
            "amount":"0.00230",
            "order_id":"1",
            "pair":"BTC/USDC",
            "limit_price":"63500.00",
        "    side":"SELL"
        },
        {
            "type_op":"CREATE",
            "account_id":"2",
            "amount":"0.00230",
            "order_id":"2",
            "pair":"BTC/USDC",
            "limit_price":"63500.00",
            "side":"BUY"
        }
    ]