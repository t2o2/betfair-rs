use crate::dto::streaming::UnmatchedOrder;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OrderCache {
    pub market_id: String,
    pub runners: HashMap<u64, RunnerOrders>,
    pub last_update: i64,
}

#[derive(Debug, Clone)]
pub struct RunnerOrders {
    pub selection_id: u64,
    pub handicap: Option<Decimal>,
    pub orders: HashMap<String, UnmatchedOrder>,
    pub matched_backs: HashMap<String, Decimal>,
    pub matched_lays: HashMap<String, Decimal>,
}

impl OrderCache {
    pub fn new(market_id: String) -> Self {
        Self {
            market_id,
            runners: HashMap::new(),
            last_update: 0,
        }
    }

    pub fn update_timestamp(&mut self, timestamp: i64) {
        self.last_update = timestamp;
    }

    pub fn get_runner(&self, selection_id: u64) -> Option<&RunnerOrders> {
        self.runners.get(&selection_id)
    }

    pub fn get_runner_mut(&mut self, selection_id: u64) -> &mut RunnerOrders {
        self.runners
            .entry(selection_id)
            .or_insert_with(|| RunnerOrders::new(selection_id))
    }

    pub fn get_all_orders(&self) -> Vec<&UnmatchedOrder> {
        self.runners
            .values()
            .flat_map(|r| r.orders.values())
            .collect()
    }

    pub fn get_active_orders(&self) -> Vec<&UnmatchedOrder> {
        self.get_all_orders()
            .into_iter()
            .filter(|o| o.status == "E")
            .collect()
    }

    pub fn clear(&mut self) {
        self.runners.clear();
    }
}

impl RunnerOrders {
    pub fn new(selection_id: u64) -> Self {
        Self {
            selection_id,
            handicap: None,
            orders: HashMap::new(),
            matched_backs: HashMap::new(),
            matched_lays: HashMap::new(),
        }
    }

    pub fn set_handicap(&mut self, handicap: Option<Decimal>) {
        self.handicap = handicap;
    }

    pub fn apply_full_image(&mut self, orders: Vec<UnmatchedOrder>) {
        self.orders.clear();
        for order in orders {
            self.orders.insert(order.id.clone(), order);
        }
    }

    pub fn update_order(&mut self, order: UnmatchedOrder) {
        if order.status == "EC" {
            self.orders.remove(&order.id);
        } else {
            self.orders.insert(order.id.clone(), order);
        }
    }

    pub fn update_matched_backs(&mut self, matched_backs: Vec<Vec<Decimal>>) {
        for entry in matched_backs {
            if entry.len() >= 2 {
                let price = entry[0].to_string();
                let size = entry[1];
                if size.is_zero() {
                    self.matched_backs.remove(&price);
                } else {
                    self.matched_backs.insert(price, size);
                }
            }
        }
    }

    pub fn update_matched_lays(&mut self, matched_lays: Vec<Vec<Decimal>>) {
        for entry in matched_lays {
            if entry.len() >= 2 {
                let price = entry[0].to_string();
                let size = entry[1];
                if size.is_zero() {
                    self.matched_lays.remove(&price);
                } else {
                    self.matched_lays.insert(price, size);
                }
            }
        }
    }

    pub fn clear_matched_backs(&mut self) {
        self.matched_backs.clear();
    }

    pub fn clear_matched_lays(&mut self) {
        self.matched_lays.clear();
    }

    pub fn get_order(&self, bet_id: &str) -> Option<&UnmatchedOrder> {
        self.orders.get(bet_id)
    }

    pub fn get_total_back_matched(&self) -> Decimal {
        self.matched_backs.values().sum()
    }

    pub fn get_total_lay_matched(&self) -> Decimal {
        self.matched_lays.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_order(id: &str, price: Decimal, size: Decimal, status: &str) -> UnmatchedOrder {
        UnmatchedOrder {
            id: id.to_string(),
            p: price,
            s: size,
            bsp: None,
            side: "B".to_string(),
            status: status.to_string(),
            pt: "L".to_string(),
            ot: "L".to_string(),
            pd: 1234567890000,
            md: None,
            cd: None,
            ld: None,
            lsrc: None,
            avp: None,
            sm: None,
            sr: Some(size),
            sl: None,
            sc: None,
            sv: None,
            rac: None,
            rc: None,
            rfo: None,
            rfs: None,
        }
    }

    #[test]
    fn test_order_cache_new() {
        let cache = OrderCache::new("1.123456".to_string());
        assert_eq!(cache.market_id, "1.123456");
        assert!(cache.runners.is_empty());
        assert_eq!(cache.last_update, 0);
    }

    #[test]
    fn test_runner_orders_new() {
        let runner = RunnerOrders::new(12345);
        assert_eq!(runner.selection_id, 12345);
        assert!(runner.handicap.is_none());
        assert!(runner.orders.is_empty());
        assert!(runner.matched_backs.is_empty());
        assert!(runner.matched_lays.is_empty());
    }

    #[test]
    fn test_update_order() {
        let mut runner = RunnerOrders::new(12345);
        let order = create_test_order("bet1", dec!(2.0), dec!(10.0), "E");

        runner.update_order(order.clone());
        assert_eq!(runner.orders.len(), 1);
        assert!(runner.get_order("bet1").is_some());
    }

    #[test]
    fn test_remove_completed_order() {
        let mut runner = RunnerOrders::new(12345);
        let order1 = create_test_order("bet1", dec!(2.0), dec!(10.0), "E");
        let order2 = create_test_order("bet1", dec!(2.0), dec!(10.0), "EC");

        runner.update_order(order1);
        assert_eq!(runner.orders.len(), 1);

        runner.update_order(order2);
        assert_eq!(runner.orders.len(), 0);
    }

    #[test]
    fn test_apply_full_image() {
        let mut runner = RunnerOrders::new(12345);
        let order1 = create_test_order("bet1", dec!(2.0), dec!(10.0), "E");
        let order2 = create_test_order("bet2", dec!(3.0), dec!(20.0), "E");

        runner.update_order(order1.clone());
        assert_eq!(runner.orders.len(), 1);

        runner.apply_full_image(vec![order1, order2]);
        assert_eq!(runner.orders.len(), 2);
    }

    #[test]
    fn test_update_matched_backs() {
        let mut runner = RunnerOrders::new(12345);

        runner.update_matched_backs(vec![
            vec![dec!(2.0), dec!(50.0)],
            vec![dec!(2.5), dec!(100.0)],
        ]);
        assert_eq!(runner.matched_backs.len(), 2);

        let keys: Vec<String> = runner.matched_backs.keys().cloned().collect();
        assert!(
            keys.contains(&"2".to_string()) || keys.contains(&"2.0".to_string()),
            "Expected key '2' or '2.0', got: {keys:?}"
        );

        let price_key = dec!(2.0).to_string();
        assert_eq!(*runner.matched_backs.get(&price_key).unwrap(), dec!(50.0));

        let price_key_2 = dec!(2.5).to_string();
        assert_eq!(
            *runner.matched_backs.get(&price_key_2).unwrap(),
            dec!(100.0)
        );
    }

    #[test]
    fn test_remove_matched_backs_with_zero() {
        let mut runner = RunnerOrders::new(12345);

        runner.update_matched_backs(vec![
            vec![dec!(2.0), dec!(50.0)],
            vec![dec!(2.5), dec!(100.0)],
        ]);
        assert_eq!(runner.matched_backs.len(), 2);

        runner.update_matched_backs(vec![vec![dec!(2.0), Decimal::ZERO]]);
        assert_eq!(runner.matched_backs.len(), 1);
        assert!(!runner.matched_backs.contains_key("2"));
    }

    #[test]
    fn test_get_all_orders() {
        let mut cache = OrderCache::new("1.123456".to_string());
        let order1 = create_test_order("bet1", dec!(2.0), dec!(10.0), "E");
        let order2 = create_test_order("bet2", dec!(3.0), dec!(20.0), "E");

        cache.get_runner_mut(12345).update_order(order1);
        cache.get_runner_mut(12346).update_order(order2);

        let all_orders = cache.get_all_orders();
        assert_eq!(all_orders.len(), 2);
    }

    #[test]
    fn test_get_active_orders() {
        let mut cache = OrderCache::new("1.123456".to_string());
        let order1 = create_test_order("bet1", dec!(2.0), dec!(10.0), "E");
        let order2 = create_test_order("bet2", dec!(3.0), dec!(20.0), "EC");

        cache.get_runner_mut(12345).update_order(order1);
        cache.get_runner_mut(12345).update_order(order2);

        let active_orders = cache.get_active_orders();
        assert_eq!(active_orders.len(), 1);
        assert_eq!(active_orders[0].id, "bet1");
    }

    #[test]
    fn test_get_total_matched() {
        let mut runner = RunnerOrders::new(12345);

        runner.update_matched_backs(vec![
            vec![dec!(2.0), dec!(50.0)],
            vec![dec!(2.5), dec!(100.0)],
        ]);
        runner.update_matched_lays(vec![vec![dec!(3.0), dec!(75.0)]]);

        assert_eq!(runner.get_total_back_matched(), dec!(150.0));
        assert_eq!(runner.get_total_lay_matched(), dec!(75.0));
    }

    #[test]
    fn test_clear_cache() {
        let mut cache = OrderCache::new("1.123456".to_string());
        let order = create_test_order("bet1", dec!(2.0), dec!(10.0), "E");

        cache.get_runner_mut(12345).update_order(order);
        assert_eq!(cache.runners.len(), 1);

        cache.clear();
        assert_eq!(cache.runners.len(), 0);
    }
}
