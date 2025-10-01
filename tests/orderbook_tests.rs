use betfair_rs::orderbook::Orderbook;
use rust_decimal_macros::dec;

#[test]
fn test_new_orderbook() {
    let orderbook = Orderbook::new();
    assert_eq!(orderbook.ts, 0);
    assert!(orderbook.bids.is_empty());
    assert!(orderbook.asks.is_empty());
}

#[test]
fn test_set_timestamp() {
    let mut orderbook = Orderbook::new();
    orderbook.set_ts(1234567890);
    assert_eq!(orderbook.ts, 1234567890);
}

#[test]
fn test_add_and_remove_bids() {
    let mut orderbook = Orderbook::new();

    // Add bids
    orderbook.add_bid(1, dec!(2.0), dec!(10.0));
    orderbook.add_bid(2, dec!(1.9), dec!(20.0));
    orderbook.add_bid(3, dec!(1.8), dec!(30.0));

    assert_eq!(orderbook.bids.len(), 3);
    assert_eq!(orderbook.bids[0].level, 1);
    assert_eq!(orderbook.bids[0].price, dec!(2.0));
    assert_eq!(orderbook.bids[0].size, dec!(10.0));

    // Remove bid by setting size to 0
    orderbook.add_bid(2, dec!(1.9), dec!(0.0));
    assert_eq!(orderbook.bids.len(), 2);
    assert!(!orderbook.bids.iter().any(|b| b.level == 2));
}

#[test]
fn test_add_and_remove_asks() {
    let mut orderbook = Orderbook::new();

    // Add asks
    orderbook.add_ask(1, dec!(2.1), dec!(10.0));
    orderbook.add_ask(2, dec!(2.2), dec!(20.0));
    orderbook.add_ask(3, dec!(2.3), dec!(30.0));

    assert_eq!(orderbook.asks.len(), 3);
    assert_eq!(orderbook.asks[0].level, 1);
    assert_eq!(orderbook.asks[0].price, dec!(2.1));
    assert_eq!(orderbook.asks[0].size, dec!(10.0));

    // Remove ask by setting size to 0
    orderbook.add_ask(2, dec!(2.2), dec!(0.0));
    assert_eq!(orderbook.asks.len(), 2);
    assert!(!orderbook.asks.iter().any(|a| a.level == 2));
}

#[test]
fn test_update_existing_levels() {
    let mut orderbook = Orderbook::new();

    // Add initial levels
    orderbook.add_bid(1, dec!(2.0), dec!(10.0));
    orderbook.add_ask(1, dec!(2.1), dec!(10.0));

    // Update levels
    orderbook.add_bid(1, dec!(2.0), dec!(15.0));
    orderbook.add_ask(1, dec!(2.1), dec!(15.0));

    assert_eq!(orderbook.bids[0].size, dec!(15.0));
    assert_eq!(orderbook.asks[0].size, dec!(15.0));
}

#[test]
fn test_best_bid_and_ask() {
    let mut orderbook = Orderbook::new();

    // Add multiple levels
    orderbook.add_bid(1, dec!(2.0), dec!(10.0));
    orderbook.add_bid(2, dec!(1.9), dec!(20.0));
    orderbook.add_bid(3, dec!(1.8), dec!(30.0));

    orderbook.add_ask(1, dec!(2.1), dec!(10.0));
    orderbook.add_ask(2, dec!(2.2), dec!(20.0));
    orderbook.add_ask(3, dec!(2.3), dec!(30.0));

    let best_bid = orderbook.get_best_bid().unwrap();
    let best_ask = orderbook.get_best_ask().unwrap();

    assert_eq!(best_bid.level, 1);
    assert_eq!(best_bid.price, dec!(2.0));
    assert_eq!(best_ask.level, 1);
    assert_eq!(best_ask.price, dec!(2.1));
}

#[test]
fn test_empty_orderbook() {
    let orderbook = Orderbook::new();
    assert!(orderbook.get_best_bid().is_none());
    assert!(orderbook.get_best_ask().is_none());
}

#[test]
fn test_pretty_print() {
    let mut orderbook = Orderbook::new();
    orderbook.add_bid(1, dec!(2.0), dec!(10.0));
    orderbook.add_ask(1, dec!(2.1), dec!(10.0));

    let printed = orderbook.pretty_print();
    assert!(printed.contains("2.0"));
    assert!(printed.contains("2.1"));
    assert!(printed.contains("10.0"));
}
