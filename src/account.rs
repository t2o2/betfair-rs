use serde::{Deserialize, Serialize};

/*
{"jsonrpc":"2.0","result":{"availableToBetBalance":11012.28,"exposure":-1917.08,"retainedCommission":0.0,"exposureLimit":-11000.0,"discountRate":18.0,"pointsBalance":14007,"wallet":"UK"},"id":1}
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountFundsResponse {
    #[serde(rename = "availableToBetBalance")]
    pub available_to_bet_balance: f64,
    #[serde(rename = "exposure")]
    pub exposure: f64,
    #[serde(rename = "retainedCommission")]
    pub retained_commission: f64,
    #[serde(rename = "exposureLimit")]
    pub exposure_limit: f64,
    #[serde(rename = "discountRate")]
    pub discount_rate: f64,
    #[serde(rename = "pointsBalance")]
    pub points_balance: f64,
    #[serde(rename = "wallet")]
    pub wallet: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAccountFundsRequest {
    #[serde(rename = "wallet", skip_serializing_if = "Option::is_none")]
    pub wallet: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAccountFundsResponse {
    #[serde(rename = "availableToBetBalance")]
    pub available_to_bet_balance: f64,
    #[serde(rename = "exposure")]
    pub exposure: f64,
    #[serde(rename = "retainedCommission")]
    pub retained_commission: f64,
    #[serde(rename = "exposureLimit")]
    pub exposure_limit: f64,
    #[serde(rename = "discountRate")]
    pub discount_rate: f64,
    #[serde(rename = "pointsBalance")]
    pub points_balance: f64,
    #[serde(rename = "wallet")]
    pub wallet: String,
}
