use rust_decimal::Decimal;

#[derive(Debug, Clone, Default)]
pub struct Orderbook {
    pub ts: i64,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub level: usize,
    pub price: Decimal,
    pub size: Decimal,
}

impl Orderbook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_ts(&mut self, ts: i64) {
        self.ts = ts;
    }

    pub fn add_bid(&mut self, level: usize, price: Decimal, size: Decimal) {
        if size.is_zero() {
            // Remove the level if size is 0
            if let Some(index) = self.bids.iter().position(|l| l.level == level) {
                self.bids.remove(index);
            }
        } else {
            // Find if level exists
            if let Some(index) = self.bids.iter().position(|l| l.level == level) {
                // Update existing level
                self.bids[index] = PriceLevel { level, price, size };
            } else {
                // Insert new level maintaining order
                let insert_pos = self
                    .bids
                    .iter()
                    .position(|l| l.level > level)
                    .unwrap_or(self.bids.len());
                self.bids
                    .insert(insert_pos, PriceLevel { level, price, size });
            }
        }
    }

    pub fn add_ask(&mut self, level: usize, price: Decimal, size: Decimal) {
        if size.is_zero() {
            // Remove the level if size is 0
            if let Some(index) = self.asks.iter().position(|l| l.level == level) {
                self.asks.remove(index);
            }
        } else {
            // Find if level exists
            if let Some(index) = self.asks.iter().position(|l| l.level == level) {
                // Update existing level
                self.asks[index] = PriceLevel { level, price, size };
            } else {
                // Insert new level maintaining order
                let insert_pos = self
                    .asks
                    .iter()
                    .position(|l| l.level > level)
                    .unwrap_or(self.asks.len());
                self.asks
                    .insert(insert_pos, PriceLevel { level, price, size });
            }
        }
    }

    pub fn get_best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    pub fn get_best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    pub fn pretty_print(&self) -> String {
        let mut output = String::new();
        output.push_str("\n  Asks:\n");
        for ask in self.asks.iter().rev() {
            output.push_str(&format!(
                "    Level {}: Price = {:.2}, Size = {:.2}\n",
                ask.level, ask.price, ask.size
            ));
        }
        output.push_str("  Bids:\n");
        for bid in &self.bids {
            output.push_str(&format!(
                "    Level {}: Price = {:.2}, Size = {:.2}\n",
                bid.level, bid.price, bid.size
            ));
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_new() {
        let ob = Orderbook::new();
        assert_eq!(ob.ts, 0);
        assert!(ob.asks.is_empty());
        assert!(ob.bids.is_empty());
    }

    #[test]
    fn test_orderbook_add_ask() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_ask(0, dec!(2.0), dec!(100.0));
        ob.add_ask(1, dec!(2.1), dec!(150.0));

        assert_eq!(ob.asks.len(), 2);
        assert_eq!(ob.asks[0].price, dec!(2.0));
        assert_eq!(ob.asks[1].price, dec!(2.1));
    }

    #[test]
    fn test_orderbook_add_bid() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_bid(0, dec!(1.9), dec!(200.0));
        ob.add_bid(1, dec!(1.8), dec!(250.0));

        assert_eq!(ob.bids.len(), 2);
        assert_eq!(ob.bids[0].price, dec!(1.9));
        assert_eq!(ob.bids[1].price, dec!(1.8));
    }

    #[test]
    fn test_orderbook_remove_ask_with_zero_size() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_ask(0, dec!(2.0), dec!(100.0));
        ob.add_ask(1, dec!(2.1), dec!(150.0));
        assert_eq!(ob.asks.len(), 2);

        ob.add_ask(0, dec!(2.0), dec!(0.0));
        assert_eq!(ob.asks.len(), 1);
        assert_eq!(ob.asks[0].price, dec!(2.1));
    }

    #[test]
    fn test_orderbook_remove_bid_with_zero_size() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_bid(0, dec!(1.9), dec!(200.0));
        ob.add_bid(1, dec!(1.8), dec!(250.0));
        assert_eq!(ob.bids.len(), 2);

        ob.add_bid(1, dec!(1.8), dec!(0.0));
        assert_eq!(ob.bids.len(), 1);
        assert_eq!(ob.bids[0].price, dec!(1.9));
    }

    #[test]
    fn test_orderbook_update_existing_levels() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_ask(0, dec!(2.0), dec!(100.0));
        ob.add_ask(1, dec!(2.1), dec!(150.0));

        ob.add_ask(0, dec!(2.0), dec!(200.0));

        assert_eq!(ob.asks.len(), 2);
        assert_eq!(ob.asks[0].size, dec!(200.0));
        assert_eq!(ob.asks[1].size, dec!(150.0));
    }

    #[test]
    fn test_orderbook_best_bid_and_ask() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        assert!(ob.get_best_bid().is_none());
        assert!(ob.get_best_ask().is_none());

        ob.add_bid(0, dec!(1.9), dec!(200.0));
        ob.add_bid(1, dec!(1.8), dec!(250.0));

        ob.add_ask(0, dec!(2.0), dec!(100.0));
        ob.add_ask(1, dec!(2.1), dec!(150.0));

        assert_eq!(ob.get_best_bid().unwrap().price, dec!(1.9));
        assert_eq!(ob.get_best_ask().unwrap().price, dec!(2.0));
    }

    #[test]
    fn test_orderbook_set_ts() {
        let mut ob = Orderbook::new();

        ob.set_ts(12345678);
        assert_eq!(ob.ts, 12345678);
    }

    #[test]
    fn test_orderbook_pretty_print() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_ask(0, dec!(2.0), dec!(100.0));
        ob.add_bid(0, dec!(1.9), dec!(200.0));

        let output = ob.pretty_print();
        assert!(output.contains("Asks:"));
        assert!(output.contains("Bids:"));
        assert!(output.contains("Price = 2.00"));
        assert!(output.contains("Price = 1.90"));
    }

    #[test]
    fn test_orderbook_empty() {
        let ob = Orderbook::new();

        let output = ob.pretty_print();
        assert!(output.contains("Asks:"));
        assert!(output.contains("Bids:"));
    }

    #[test]
    fn test_orderbook_mixed_operations() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_ask(0, dec!(2.0), dec!(100.0));
        ob.add_ask(1, dec!(2.1), dec!(150.0));
        ob.add_ask(2, dec!(2.2), dec!(200.0));

        ob.add_bid(0, dec!(1.9), dec!(200.0));
        ob.add_bid(1, dec!(1.8), dec!(250.0));

        ob.add_ask(1, dec!(2.1), dec!(0.0));
        ob.add_ask(3, dec!(2.3), dec!(300.0));

        assert_eq!(ob.asks.len(), 3);
        assert_eq!(ob.asks[2].price, dec!(2.3));

        ob.add_bid(0, dec!(1.9), dec!(0.0));
        assert_eq!(ob.bids.len(), 1);
        assert_eq!(ob.get_best_bid().unwrap().price, dec!(1.8));
    }

    #[test]
    fn test_price_level() {
        use rust_decimal_macros::dec;
        let level = PriceLevel {
            level: 0,
            price: dec!(1.95),
            size: dec!(500.0),
        };

        assert_eq!(level.level, 0);
        assert_eq!(level.price, dec!(1.95));
        assert_eq!(level.size, dec!(500.0));
    }

    #[test]
    fn test_orderbook_with_zero_size_removes_level() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_ask(0, dec!(2.0), dec!(100.0));
        assert_eq!(ob.asks.len(), 1);

        ob.add_ask(0, dec!(2.0), dec!(0.0));
        assert_eq!(ob.asks.len(), 0);
    }

    #[test]
    fn test_orderbook_maintain_order() {
        use rust_decimal_macros::dec;
        let mut ob = Orderbook::new();

        ob.add_ask(2, dec!(2.2), dec!(200.0));
        ob.add_ask(0, dec!(2.0), dec!(100.0));
        ob.add_ask(1, dec!(2.1), dec!(150.0));

        assert_eq!(ob.asks[0].level, 0);
        assert_eq!(ob.asks[1].level, 1);
        assert_eq!(ob.asks[2].level, 2);
    }
}
