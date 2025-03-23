struct Orderbook {
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

struct PriceLevel {
    price: f64,
    size: f64,
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