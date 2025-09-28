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
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MarketChange {
    pub id: String,
    #[serde(rename = "rc")]
    pub runner_changes: Option<Vec<RunnerChange>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RunnerChange {
    pub id: u64,
    #[serde(rename = "batb")]
    pub available_to_back: Option<Vec<[f64; 3]>>,
    #[serde(rename = "batl")]
    pub available_to_lay: Option<Vec<[f64; 3]>>,
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
    #[serde(rename = "oc")]
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
    pub handicap: Option<f64>,
    #[serde(rename = "fullImage", default)]
    pub full_image: bool,
    #[serde(rename = "uo")]
    pub unmatched_orders: Option<Vec<UnmatchedOrder>>,
    #[serde(rename = "mb")]
    pub matched_backs: Option<Vec<Vec<f64>>>,
    #[serde(rename = "ml")]
    pub matched_lays: Option<Vec<Vec<f64>>>,
    #[serde(rename = "smc")]
    pub strategy_matches: Option<std::collections::HashMap<String, StrategyMatchChange>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnmatchedOrder {
    pub id: String,
    pub p: f64,
    pub s: f64,
    #[serde(default)]
    pub bsp: Option<f64>,
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
    pub avp: Option<f64>,
    #[serde(default)]
    pub sm: Option<f64>,
    #[serde(default)]
    pub sr: Option<f64>,
    #[serde(default)]
    pub sl: Option<f64>,
    #[serde(default)]
    pub sc: Option<f64>,
    #[serde(default)]
    pub sv: Option<f64>,
    #[serde(default)]
    pub rac: Option<String>,
    #[serde(default)]
    pub rc: Option<String>,
    #[serde(default)]
    pub rfo: Option<String>,
    #[serde(default)]
    pub rfs: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyMatchChange {
    #[serde(rename = "mb")]
    pub matched_backs: Option<Vec<Vec<f64>>>,
    #[serde(rename = "ml")]
    pub matched_lays: Option<Vec<Vec<f64>>>,
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
}
