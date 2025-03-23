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

// {"op":"mcm","id":1,"clk":"AJctAKk5AJMu","pt":1742747423927,"mc":[{"id":"1.241200277","rc":[{"batb":[[0,4.3,943.24]],"id":58805}]}]}
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
    pub runner_changes: Vec<RunnerChange>
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RunnerChange {
    pub id: i64,
    #[serde(rename = "batb")]
    pub available_to_back: Option<Vec<Vec<f64>>>, // Array of [index, price, size]
    #[serde(rename = "batl")]
    pub available_to_lay: Option<Vec<Vec<f64>>> // Array of [index, price, size]
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct HeartbeatMessage {
    pub clk: String,
    pub ct: String,  // This will always be "HEARTBEAT"
    pub id: i64,
    pub op: String,  // This will always be "mcm"
    pub pt: i64,    // Timestamp
}