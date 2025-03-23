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
pub struct StreamMessage {
    pub id: i64,
    pub op: String,
    pub ct: String,
    pub clk: String, // Checkpoint, allow resuming from this point when reconnecting to the stream
    pub pt: i64, // publishTime
    pub mc: Vec<MarketChange>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct MarketChange {
    pub id: i64,
    pub rc: Vec<RunnerChange>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct RunnerChange {
    pub id: i64, // runner id
    pub atb: Option<Vec<f64>>, // available to back
    pub atl: Option<Vec<f64>>, // available to lay
}
