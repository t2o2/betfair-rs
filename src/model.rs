use serde::{Deserialize, Serialize};
use std::fmt;


#[derive(Debug, Serialize, Deserialize)]
pub struct Selection {
    pub selection_id: i64,
    pub runner_name: String,
    pub last_price_traded: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceSize {
    pub price: f64,
    pub size: f64,
} 

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct LoginResponse {
    pub sessionToken: Option<String>,
    pub loginStatus: String,
}

impl fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LoginResponse {{ status: {} }}", self.loginStatus)
    }
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct MarketChangeMessage {
    #[serde(rename = "clk")]
    pub clock: String,
    pub id: i64,
    #[serde(rename = "mc")]
    pub market_changes: Vec<MarketChange>,
    pub op: String,
    pub pt: i64,
    pub ct: String
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct MarketChange {
    pub id: String,
    #[serde(rename = "rc")]
    pub runner_changes: Vec<RunnerChange>
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct RunnerChange {
    pub id: i64,
    #[serde(rename = "batb")]
    pub available_to_back: Option<Vec<Vec<f64>>>, // Array of [index, price, size]
    #[serde(rename = "batl")]
    pub available_to_lay: Option<Vec<Vec<f64>>> // Array of [index, price, size]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub clk: String,
    pub ct: String,  // This will always be "HEARTBEAT"
    pub id: i64,
    pub op: String,  // This will always be "mcm"
    pub pt: i64,    // Timestamp
}

impl HeartbeatMessage {
    pub fn is_heartbeat(ct: &str) -> bool {
        ct == "HEARTBEAT"
    }
}