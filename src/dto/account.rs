use serde::{Deserialize, Serialize};
use super::common::Wallet;

// Legacy response format for account funds
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountFundsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<Wallet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountFundsResponse {
    pub available_to_bet_balance: f64,
    pub exposure: f64,
    pub retained_commission: f64,
    pub exposure_limit: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_rate: Option<f64>,
    pub points_balance: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountDetailsRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountDetailsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_balance: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFundsRequest {
    pub from: Wallet,
    pub to: Wallet,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFundsResponse {
    pub transaction_id: String,
}