use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "sessionToken")]
    pub session_token: Option<String>,
    #[serde(rename = "loginStatus")]
    pub login_status: String,
}

impl fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LoginResponse {{ status: {} }}", self.login_status)
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MarketChangeMessage {
    #[serde(rename = "clk")]
    pub clock: String,
    pub id: i64,
    #[serde(rename = "mc")]
    pub market_changes: Vec<MarketChange>,
    pub op: String,
    pub pt: i64,
    pub ct: Option<String>,
    #[serde(rename = "initialClk", default)]
    pub initial_clock: Option<String>,
    #[serde(rename = "conflateMs", default)]
    pub conflate_ms: Option<i64>,
    #[serde(rename = "heartbeatMs", default)]
    pub heartbeat_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MarketChange {
    pub id: String,
    #[serde(rename = "rc")]
    pub runner_changes: Option<Vec<RunnerChange>>,
    #[serde(rename = "marketDefinition")]
    pub market_definition: Option<MarketDefinition>,
    #[serde(default)]
    pub img: Option<bool>,
    #[serde(default)]
    pub con: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketDefinition {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(rename = "inPlay", default)]
    pub in_play: bool,
    #[serde(default)]
    pub complete: bool,
    #[serde(rename = "marketTime")]
    pub market_time: Option<String>,
    #[serde(rename = "numberOfActiveRunners")]
    pub number_of_active_runners: Option<i32>,
    #[serde(rename = "bspMarket", default)]
    pub bsp_market: Option<bool>,
    #[serde(rename = "turnInPlayEnabled", default)]
    pub turn_in_play_enabled: Option<bool>,
    #[serde(rename = "persistenceEnabled", default)]
    pub persistence_enabled: Option<bool>,
    #[serde(rename = "marketBaseRate", default)]
    pub market_base_rate: Option<f64>,
    #[serde(rename = "eventId", default)]
    pub event_id: Option<String>,
    #[serde(rename = "eventTypeId", default)]
    pub event_type_id: Option<String>,
    #[serde(rename = "numberOfWinners", default)]
    pub number_of_winners: Option<i32>,
    #[serde(rename = "bettingType", default)]
    pub betting_type: Option<String>,
    #[serde(rename = "marketType", default)]
    pub market_type: Option<String>,
    #[serde(rename = "suspendTime", default)]
    pub suspend_time: Option<String>,
    #[serde(rename = "bspReconciled", default)]
    pub bsp_reconciled: Option<bool>,
    #[serde(rename = "crossMatching", default)]
    pub cross_matching: Option<bool>,
    #[serde(rename = "runnersVoidable", default)]
    pub runners_voidable: Option<bool>,
    #[serde(rename = "betDelay", default)]
    pub bet_delay: Option<i32>,
    #[serde(default)]
    pub runners: Option<serde_json::Value>,
    #[serde(default)]
    pub regulators: Option<Vec<String>>,
    #[serde(rename = "countryCode", default)]
    pub country_code: Option<String>,
    #[serde(rename = "discountAllowed", default)]
    pub discount_allowed: Option<bool>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(rename = "openDate", default)]
    pub open_date: Option<String>,
    #[serde(default)]
    pub version: Option<i64>,
    #[serde(rename = "priceLadderDefinition", default)]
    pub price_ladder_definition: Option<serde_json::Value>,
    #[serde(rename = "eachWayDivisor", default)]
    pub eachway_divisor: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RunnerChange {
    pub id: u64,
    #[serde(rename = "batb", default)]
    #[serde(with = "super::decimal_serde::option_vec_array3")]
    pub available_to_back: Option<Vec<[Decimal; 3]>>,
    #[serde(rename = "batl", default)]
    #[serde(with = "super::decimal_serde::option_vec_array3")]
    pub available_to_lay: Option<Vec<[Decimal; 3]>>,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatMessage {
    pub op: String,
    pub id: i64,
}

impl fmt::Display for HeartbeatMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HeartbeatMessage {{ op: {}, id: {} }}", self.op, self.id)
    }
}

#[derive(Debug, Serialize)]
pub struct HeartbeatRequest {
    pub op: String,
    pub id: i64,
}

impl HeartbeatRequest {
    pub fn new(id: i64) -> Self {
        HeartbeatRequest {
            op: "heartbeat".to_string(),
            id,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderChangeMessage {
    #[serde(rename = "clk")]
    pub clock: String,
    pub pt: i64,
    #[serde(rename = "oc", default)]
    pub order_changes: Vec<OrderChange>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderChange {
    pub id: String,
    #[serde(rename = "orc")]
    pub order_runner_change: Option<Vec<OrderRunnerChange>>,
    #[serde(rename = "fullImage", default)]
    pub full_image: bool,
    #[serde(default)]
    pub closed: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderRunnerChange {
    pub id: u64,
    #[serde(rename = "hc")]
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option")]
    pub handicap: Option<Decimal>,
    #[serde(rename = "fullImage", default)]
    pub full_image: bool,
    #[serde(rename = "uo")]
    #[serde(default)]
    pub unmatched_orders: Option<Vec<UnmatchedOrder>>,
    #[serde(rename = "mb")]
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option_vec_vec_decimal")]
    pub matched_backs: Option<Vec<Vec<Decimal>>>,
    #[serde(rename = "ml")]
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option_vec_vec_decimal")]
    pub matched_lays: Option<Vec<Vec<Decimal>>>,
    #[serde(rename = "smc")]
    #[serde(default)]
    pub strategy_matches: Option<std::collections::HashMap<String, StrategyMatchChange>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnmatchedOrder {
    pub id: String,
    #[serde(with = "super::decimal_serde")]
    pub p: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub s: Decimal,
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option")]
    pub bsp: Option<Decimal>,
    pub side: String,
    pub status: String,
    pub pt: String,
    pub ot: String,
    pub pd: i64,
    #[serde(default)]
    pub md: Option<i64>,
    #[serde(default)]
    pub cd: Option<i64>,
    #[serde(default)]
    pub ld: Option<i64>,
    #[serde(default)]
    pub lsrc: Option<String>,
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option")]
    pub avp: Option<Decimal>,
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option")]
    pub sm: Option<Decimal>,
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option")]
    pub sr: Option<Decimal>,
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option")]
    pub sl: Option<Decimal>,
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option")]
    pub sc: Option<Decimal>,
    #[serde(default)]
    #[serde(with = "super::decimal_serde::option")]
    pub sv: Option<Decimal>,
    #[serde(default)]
    pub rac: Option<String>,
    #[serde(default)]
    pub rc: Option<String>,
    /// Customer Order Reference (rfo = Reference For Order)
    /// This field contains the sanitized internal_order_id we passed when placing the order
    /// (no hyphens, max 32 chars)
    #[serde(default)]
    pub rfo: Option<String>,
    #[serde(default)]
    pub rfs: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyMatchChange {
    #[serde(rename = "mb")]
    #[serde(with = "super::decimal_serde::option_vec_vec_decimal")]
    pub matched_backs: Option<Vec<Vec<Decimal>>>,
    #[serde(rename = "ml")]
    #[serde(with = "super::decimal_serde::option_vec_vec_decimal")]
    pub matched_lays: Option<Vec<Vec<Decimal>>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderFilter {
    #[serde(
        rename = "includeOverallPosition",
        skip_serializing_if = "Option::is_none"
    )]
    pub include_overall_position: Option<bool>,
    #[serde(
        rename = "customerStrategyRefs",
        skip_serializing_if = "Option::is_none"
    )]
    pub customer_strategy_refs: Option<Vec<String>>,
    #[serde(
        rename = "partitionMatchedByStrategyRef",
        skip_serializing_if = "Option::is_none"
    )]
    pub partition_matched_by_strategy_ref: Option<bool>,
}

impl Default for OrderFilter {
    fn default() -> Self {
        Self {
            include_overall_position: Some(true),
            customer_strategy_refs: None,
            partition_matched_by_strategy_ref: Some(false),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderSubscriptionMessage {
    pub op: String,
    #[serde(rename = "orderFilter", skip_serializing_if = "Option::is_none")]
    pub order_filter: Option<OrderFilter>,
    #[serde(rename = "segmentationEnabled")]
    pub segmentation_enabled: bool,
    #[serde(rename = "heartbeatMs", skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_definition_deserialize_with_eachway_divisor() {
        let json = r#"{
            "status": "OPEN",
            "inPlay": false,
            "complete": false,
            "marketTime": "2024-01-01T12:00:00.000Z",
            "eachWayDivisor": 4.0
        }"#;

        let result: Result<MarketDefinition, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        let market_def = result.unwrap();
        assert_eq!(market_def.eachway_divisor, Some(4.0));
    }

    #[test]
    fn test_market_definition_deserialize_without_eachway_divisor() {
        let json = r#"{
            "status": "OPEN",
            "inPlay": false,
            "complete": false,
            "marketTime": "2024-01-01T12:00:00.000Z"
        }"#;

        let result: Result<MarketDefinition, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        let market_def = result.unwrap();
        assert_eq!(market_def.eachway_divisor, None);
    }
}
