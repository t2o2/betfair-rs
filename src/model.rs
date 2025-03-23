pub struct Orderbook {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

impl Orderbook {
    pub fn new() -> Self {
        Self { bids: Vec::new(), asks: Vec::new() }
    }
    
    pub fn add_bid(&mut self, price: f64, size: f64) {
        if size == 0.0 {
            if let Ok(index) = self.bids.binary_search_by(|level| level.price.partial_cmp(&price).unwrap().reverse()) {
                self.bids.remove(index);
            }
        } else {
            let index = match self.bids.binary_search_by(|level| level.price.partial_cmp(&price).unwrap().reverse()) {
                Ok(i) => i,
                Err(i) => i,
            };
            self.bids.insert(index, PriceLevel { price, size });
        }
    }

    pub fn add_ask(&mut self, price: f64, size: f64) {
        if size == 0.0 {
            if let Ok(index) = self.asks.binary_search_by(|level| level.price.partial_cmp(&price).unwrap()) {
                self.asks.remove(index);
            }
        } else {
            let index = match self.asks.binary_search_by(|level| level.price.partial_cmp(&price).unwrap()) {
                Ok(i) => i,
                Err(i) => i,
            };
            self.asks.insert(index, PriceLevel { price, size });
        }
    }

    pub fn get_best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    pub fn get_best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orderbook() {
        let orderbook = Orderbook::new();
        assert!(orderbook.get_best_bid().is_none());
        assert!(orderbook.get_best_ask().is_none());
    }

    #[test]
    fn test_add_bids() {
        let mut orderbook = Orderbook::new();
        
        // Add bids in random order
        orderbook.add_bid(10.0, 100.0);
        orderbook.add_bid(11.0, 50.0);
        orderbook.add_bid(9.0, 200.0);

        // Check best bid is highest price
        let best_bid = orderbook.get_best_bid().unwrap();
        assert_eq!(best_bid.price, 11.0);
        assert_eq!(best_bid.size, 50.0);
    }

    #[test]
    fn test_add_asks() {
        let mut orderbook = Orderbook::new();
        
        // Add asks in random order
        orderbook.add_ask(10.0, 100.0);
        orderbook.add_ask(9.0, 50.0);
        orderbook.add_ask(11.0, 200.0);

        // Check best ask is lowest price
        let best_ask = orderbook.get_best_ask().unwrap();
        assert_eq!(best_ask.price, 9.0);
        assert_eq!(best_ask.size, 50.0);
    }

    #[test]
    fn test_remove_orders() {
        let mut orderbook = Orderbook::new();

        // Add and remove bid
        orderbook.add_bid(10.0, 100.0);
        orderbook.add_bid(10.0, 0.0); // Remove by setting size to 0
        assert!(orderbook.get_best_bid().is_none());

        // Add and remove ask
        orderbook.add_ask(10.0, 100.0);
        orderbook.add_ask(10.0, 0.0); // Remove by setting size to 0
        assert!(orderbook.get_best_ask().is_none());
    }

    #[test]
    fn test_update_orders() {
        let mut orderbook = Orderbook::new();

        // Update bid
        orderbook.add_bid(10.0, 100.0);
        orderbook.add_bid(10.0, 200.0); // Update size at same price
        let best_bid = orderbook.get_best_bid().unwrap();
        assert_eq!(best_bid.size, 200.0);

        // Update ask
        orderbook.add_ask(10.0, 100.0);
        orderbook.add_ask(10.0, 200.0); // Update size at same price
        let best_ask = orderbook.get_best_ask().unwrap();
        assert_eq!(best_ask.size, 200.0);
    }
}