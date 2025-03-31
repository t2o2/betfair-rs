#[derive(Debug, Clone)]
pub struct Orderbook {
    pub ts: i64,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub level: usize,
    pub price: f64,
    pub size: f64,
}

impl Orderbook {
    pub fn new() -> Self {
        Self { ts: 0, bids: Vec::new(), asks: Vec::new() }
    }

    pub fn set_ts(&mut self, ts: i64) {
        self.ts = ts;
    }
    
    pub fn add_bid(&mut self, level: usize, price: f64, size: f64) {
        if size == 0.0 {
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
                let insert_pos = self.bids.iter()
                    .position(|l| l.level > level)
                    .unwrap_or(self.bids.len());
                self.bids.insert(insert_pos, PriceLevel { level, price, size });
            }
        }
    }

    pub fn add_ask(&mut self, level: usize, price: f64, size: f64) {
        if size == 0.0 {
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
                let insert_pos = self.asks.iter()
                    .position(|l| l.level > level)
                    .unwrap_or(self.asks.len());
                self.asks.insert(insert_pos, PriceLevel { level, price, size });
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
            output.push_str(&format!("    Level {}: Price = {:.2}, Size = {:.2}\n", ask.level, ask.price, ask.size));
        }
        output.push_str("  Bids:\n");
        for bid in &self.bids {
            output.push_str(&format!("    Level {}: Price = {:.2}, Size = {:.2}\n", bid.level, bid.price, bid.size));
        }
        output
    }
}