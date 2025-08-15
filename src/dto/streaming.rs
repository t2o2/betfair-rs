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
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderRunnerChange {
    pub id: u64,
    #[serde(rename = "uo")]
    pub unmatched_orders: Option<Vec<UnmatchedOrder>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnmatchedOrder {
    pub id: String,
    pub p: f64,
    pub s: f64,
    pub side: String,
    pub status: String,
    pub pt: i64,
    pub ot: String,
    pub pd: i64,
}
