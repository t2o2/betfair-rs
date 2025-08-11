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
    pub runner_changes: Vec<RunnerChange>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RunnerChange {
    pub id: i64,
    #[serde(rename = "batb")]
    pub available_to_back: Option<Vec<Vec<f64>>>, // Array of [index, price, size]
    #[serde(rename = "batl")]
    pub available_to_lay: Option<Vec<Vec<f64>>>, // Array of [index, price, size]
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct HeartbeatMessage {
    pub clk: String,
    pub ct: String,
    pub op: String,
    pub pt: i64,
    pub status: Option<i64>, // TODO: handle this status when it becomes 503, indicating streaming is not reliable
}
/*
Stream API Status - latency
If any latency occurs, the ChangeMessage for the Order and Market Stream will contain a 'status' field which will
give an indication of the health of the stream data provided by the service.  This feature will be used in addition
to the heartbeat mechanism which only gives an indication that the service is up but doesn't provide an indication
of the latency of the data provided.

By default, when the stream data is up to date the value is set to null and will be set to 503 when the stream data
is unreliable (i.e. not all bets and market changes will be reflected on the stream) due to an increase in push
latency.  Clients shouldn't disconnect if status 503 is returned; when the stream recovers updates will be sent
containing the latest data.  The status is sent per subscription on heartbeats and change messages.

Example of message containing the status field:

{"op":"ocm","id":3,"clk":"AAAAAAAA","status":503,"pt":1498137379766,"ct":"HEARTBEAT"}

*/

#[derive(Debug, Deserialize, Clone)]
pub struct OrderChangeMessage {
    pub op: String,
    pub clk: String,
    pub pt: i64,
    pub oc: Vec<OrderChange>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderChange {
    pub id: String,
    pub orc: Vec<OrderRunnerChange>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderRunnerChange {
    pub id: i64,
    pub uo: Option<Vec<UnmatchedOrder>>,
    pub mb: Option<Vec<Vec<f64>>>, // Array of [price, size]
    pub ml: Option<Vec<Vec<f64>>>, // Array of [price, size]
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnmatchedOrder {
    pub id: String,     // Order ID
    pub p: f64,         // Price
    pub s: f64,         // Size
    pub side: String,   // Side: B = BACK, L = LAY
    pub status: String, // Status: EC = Execution Complete, E = Executable
    pub pt: String,     // Persistent Type: L = LAPSE, P = PERSIST, MOC = Market On Close
    pub ot: String,     // Order Type: L = LIMIT, MOC = MARKET_ON_CLOSE, LOC = LIMIT_ON_CLOSE
    pub pd: i64,        // Placement Date
    pub sm: f64,        // Size Matched
    pub sr: f64,        // Size Remaining
    pub sl: f64,        // Size Lay
    pub sc: f64,        // Size Cancelled
    pub sv: f64,        // Size Voided
    pub rac: String,    // Regulator Auth Code
    pub rc: String,     // Regulator Code
    pub rfo: String,    // Reference Order - the customer supplied order reference
    pub rfs: String,    // Reference Strategy - the customer supplied strategy reference
}
