use super::common::{
    MarketProjection, MarketStatus, MatchProjection, OrderProjection, PriceData, PriceSize,
    RunnerStatus, TimeRange,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarketFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venues: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_in_play_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_play_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_betting_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_countries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_type_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_start_time: Option<TimeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_orders: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketSort {
    #[serde(rename = "MINIMUM_TRADED")]
    MinimumTraded,
    #[serde(rename = "MAXIMUM_TRADED")]
    MaximumTraded,
    #[serde(rename = "MINIMUM_AVAILABLE")]
    MinimumAvailable,
    #[serde(rename = "MAXIMUM_AVAILABLE")]
    MaximumAvailable,
    #[serde(rename = "FIRST_TO_START")]
    FirstToStart,
    #[serde(rename = "LAST_TO_START")]
    LastToStart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMarketCatalogueRequest {
    pub filter: MarketFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_projection: Option<Vec<MarketProjection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<MarketSort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketCatalogue {
    pub market_id: String,
    pub market_name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_start_time: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<MarketDescription>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub total_matched: Option<Decimal>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners: Option<Vec<RunnerCatalog>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<EventType>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition: Option<Competition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDescription {
    pub persistence_enabled: bool,
    pub bsp_market: bool,
    pub market_time: String,
    pub suspend_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settle_time: Option<String>,
    pub betting_type: String,
    pub turn_in_play_enabled: bool,
    pub market_type: String,
    pub regulator: String,
    #[serde(with = "super::decimal_serde")]
    pub market_base_rate: Decimal,
    pub discount_allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_has_date: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub each_way_divisor: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarifications: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerCatalog {
    pub selection_id: i64,
    pub runner_name: String,
    #[serde(with = "super::decimal_serde")]
    pub handicap: Decimal,
    pub sort_priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventType {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Competition {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMarketBookRequest {
    pub market_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_projection: Option<PriceProjectionDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_projection: Option<OrderProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_projection: Option<MatchProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_overall_position: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_matched_by_strategy_ref: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceProjectionDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_data: Option<Vec<PriceData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ex_best_offers_overrides: Option<ExBestOffersOverrides>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtualise: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollover_stakes: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExBestOffersOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_prices_depth: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollup_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollup_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub rollup_liability_threshold: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollup_liability_factor: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBook {
    pub market_id: String,
    pub is_market_data_delayed: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MarketStatus>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_delay: Option<i32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp_reconciled: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inplay: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_winners: Option<i32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_runners: Option<i32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_active_runners: Option<i32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_match_time: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub total_matched: Option<Decimal>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub total_available: Option<Decimal>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_matching: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners_voidable: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners: Option<Vec<Runner>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_line_description: Option<KeyLineDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyLineDescription {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_line: Option<Vec<KeyLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyLine {
    pub selection_id: i64,
    #[serde(with = "super::decimal_serde")]
    pub handicap: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runner {
    pub selection_id: i64,
    #[serde(with = "super::decimal_serde")]
    pub handicap: Decimal,
    pub status: RunnerStatus,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub adjustment_factor: Option<Decimal>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub last_price_traded: Option<Decimal>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub total_matched: Option<Decimal>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_date: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sp: Option<StartingPrices>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ex: Option<ExchangePrices>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<MarketOrder>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<Match>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_preplay: Option<HashMap<String, Match>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartingPrices {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub near_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub far_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back_stake_taken: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lay_liability_taken: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub actual_sp: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangePrices {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_to_back: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_to_lay: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traded_volume: Option<Vec<PriceSize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOrder {
    pub bet_id: String,
    pub order_type: String,
    pub status: String,
    pub persistence_type: String,
    pub side: String,
    #[serde(with = "super::decimal_serde")]
    pub price: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub size: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub bsp_liability: Decimal,
    pub placed_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub avg_price_matched: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_matched: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_remaining: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_lapsed: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_cancelled: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_voided: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulator_auth_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulator_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub bet_id: String,
    pub match_id: String,
    pub side: String,
    #[serde(with = "super::decimal_serde")]
    pub price: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub size: Decimal,
    pub match_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketVersion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEventTypesRequest {
    pub filter: MarketFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventTypeResult {
    pub event_type: EventType,
    pub market_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEventsRequest {
    pub filter: MarketFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventResult {
    pub event: Event,
    pub market_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCompetitionsRequest {
    pub filter: MarketFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionResult {
    pub competition: Competition,
    pub market_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition_region: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_description_with_each_way_divisor() {
        let json = r#"{
            "persistenceEnabled": true,
            "bspMarket": false,
            "marketTime": "2024-01-01T12:00:00.000Z",
            "suspendTime": "2024-01-01T12:00:00.000Z",
            "bettingType": "ODDS",
            "turnInPlayEnabled": true,
            "marketType": "WIN",
            "regulator": "MR_INT",
            "marketBaseRate": 5.0,
            "discountAllowed": true,
            "eachWayDivisor": 4.0
        }"#;

        let result: Result<MarketDescription, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
        let desc = result.unwrap();
        assert!(desc.each_way_divisor.is_some());
        assert_eq!(desc.each_way_divisor.unwrap().to_string(), "4.0");
    }

    #[test]
    fn test_market_description_without_each_way_divisor() {
        let json = r#"{
            "persistenceEnabled": true,
            "bspMarket": false,
            "marketTime": "2024-01-01T12:00:00.000Z",
            "suspendTime": "2024-01-01T12:00:00.000Z",
            "bettingType": "ODDS",
            "turnInPlayEnabled": true,
            "marketType": "WIN",
            "regulator": "MR_INT",
            "marketBaseRate": 5.0,
            "discountAllowed": true
        }"#;

        let result: Result<MarketDescription, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
        let desc = result.unwrap();
        assert!(desc.each_way_divisor.is_none());
    }

    #[test]
    fn test_runner_with_adjustment_factor() {
        let json = r#"{
            "selectionId": 12345,
            "handicap": 0.0,
            "status": "ACTIVE",
            "adjustmentFactor": 2.5
        }"#;

        let result: Result<Runner, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
        let runner = result.unwrap();
        assert!(runner.adjustment_factor.is_some());
        assert_eq!(runner.adjustment_factor.unwrap().to_string(), "2.5");
    }

    #[test]
    fn test_runner_without_adjustment_factor() {
        let json = r#"{
            "selectionId": 12345,
            "handicap": 0.0,
            "status": "ACTIVE"
        }"#;

        let result: Result<Runner, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
        let runner = result.unwrap();
        assert!(runner.adjustment_factor.is_none());
    }
}
