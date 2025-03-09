use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Debug, Serialize, Deserialize)]
pub struct Market {
    pub market_id: String,
    pub market_name: String,
    pub total_matched: f64,
}

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
