use super::common::Wallet;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Legacy response format for account funds
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountFundsResponse {
    #[serde(rename = "availableToBetBalance")]
    #[serde(with = "super::decimal_serde")]
    pub available_to_bet_balance: Decimal,
    #[serde(rename = "exposure")]
    #[serde(with = "super::decimal_serde")]
    pub exposure: Decimal,
    #[serde(rename = "retainedCommission")]
    #[serde(with = "super::decimal_serde")]
    pub retained_commission: Decimal,
    #[serde(rename = "exposureLimit")]
    #[serde(with = "super::decimal_serde")]
    pub exposure_limit: Decimal,
    #[serde(rename = "discountRate")]
    #[serde(with = "super::decimal_serde")]
    pub discount_rate: Decimal,
    #[serde(rename = "pointsBalance")]
    #[serde(with = "super::decimal_serde")]
    pub points_balance: Decimal,
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
    #[serde(with = "super::decimal_serde")]
    pub available_to_bet_balance: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub exposure: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub retained_commission: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub exposure_limit: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub discount_rate: Option<Decimal>,
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
    #[serde(with = "super::decimal_serde::option")]
    pub discount_rate: Option<Decimal>,
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
    #[serde(with = "super::decimal_serde")]
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFundsResponse {
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCurrencyRatesRequest {
    /// The currency from which the rates are computed. Currently only "GBP" is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRate {
    /// Three letter ISO 4217 code
    pub currency_code: String,
    /// Exchange rate for the currency
    #[serde(with = "super::decimal_serde")]
    pub rate: Decimal,
}
