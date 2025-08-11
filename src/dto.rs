use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Common Types and Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Pending,
    ExecutionComplete,
    Executable,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Back,
    Lay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    #[serde(rename = "MARKET_ON_CLOSE")]
    MarketOnClose,
    #[serde(rename = "LIMIT_ON_CLOSE")]
    LimitOnClose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistenceType {
    Lapse,
    Persist,
    #[serde(rename = "MARKET_ON_CLOSE")]
    MarketOnClose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    #[serde(rename = "FILL_OR_KILL")]
    FillOrKill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketSort {
    MinimumTraded,
    MaximumTraded,
    MinimumAvailable,
    MaximumAvailable,
    FirstToStart,
    LastToStart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketProjection {
    Competition,
    Event,
    EventType,
    MarketStartTime,
    MarketDescription,
    RunnerDescription,
    RunnerMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PriceProjection {
    SpAvailable,
    SpTraded,
    ExTraded,
    ExAllOffers,
    ExBestOffers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchProjection {
    NoRollup,
    RolledUpByPrice,
    RolledUpByAvgPrice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderProjection {
    All,
    Executable,
    ExecutionComplete,
}

// ============================================================================
// Base API Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
    pub id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: T,
    pub id: i32,
}

// ============================================================================
// Authentication DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub session_token: Option<String>,
    pub login_status: String,
}

// ============================================================================
// Market DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<MarketDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners: Option<Vec<RunnerCatalog>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<EventType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition: Option<Competition>,
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
    pub market_base_rate: f64,
    pub discount_allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_has_date: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub each_way_divisor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarifications: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerCatalog {
    pub selection_id: i64,
    pub runner_name: String,
    pub handicap: f64,
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

// ============================================================================
// Market Book DTOs
// ============================================================================

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
    pub price_data: Option<Vec<PriceProjection>>,
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
    pub rollup_liability_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollup_liability_factor: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBook {
    pub market_id: String,
    pub is_market_data_delayed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_delay: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp_reconciled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inplay: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_winners: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_runners: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_active_runners: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_match_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_available: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_matching: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners_voidable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners: Option<Vec<Runner>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runner {
    pub selection_id: i64,
    pub handicap: f64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price_traded: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sp: Option<StartingPrices>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ex: Option<ExchangePrices>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<Order>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<Match>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartingPrices {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub near_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub far_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back_stake_taken: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lay_liability_taken: Option<Vec<PriceSize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_sp: Option<f64>,
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
pub struct PriceSize {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub bet_id: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub persistence_type: PersistenceType,
    pub side: Side,
    pub price: f64,
    pub size: f64,
    pub bsp_liability: f64,
    pub placed_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_price_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_remaining: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_lapsed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_cancelled: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_voided: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub bet_id: String,
    pub match_id: String,
    pub side: Side,
    pub price: f64,
    pub size: f64,
    pub match_date: String,
}

// ============================================================================
// Order Placement DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrdersRequest {
    pub market_id: String,
    pub instructions: Vec<PlaceInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_version: Option<MarketVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub async_: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketVersion {
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceInstruction {
    pub order_type: OrderType,
    pub selection_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handicap: Option<f64>,
    pub side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_order: Option<LimitOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_on_close_order: Option<LimitOnCloseOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_on_close_order: Option<MarketOnCloseOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOrder {
    pub size: f64,
    pub price: f64,
    pub persistence_type: PersistenceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fill_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_target_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_target_size: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOnCloseOrder {
    pub liability: f64,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOnCloseOrder {
    pub liability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrdersResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    pub market_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reports: Option<Vec<PlaceInstructionReport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceInstructionReport {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_status: Option<OrderStatus>,
    pub instruction: PlaceInstruction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placed_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_price_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_matched: Option<f64>,
}

// ============================================================================
// Order Cancellation DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrdersRequest {
    pub market_id: String,
    pub instructions: Vec<CancelInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelInstruction {
    pub bet_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_reduction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrdersResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    pub market_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reports: Option<Vec<CancelInstructionReport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelInstructionReport {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    pub instruction: CancelInstruction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_cancelled: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_date: Option<String>,
}

// ============================================================================
// Order Query DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCurrentOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_projection: Option<OrderProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_range: Option<TimeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_record: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCurrentOrdersResponse {
    pub current_orders: Vec<CurrentOrderSummary>,
    pub more_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentOrderSummary {
    pub bet_id: String,
    pub market_id: String,
    pub selection_id: i64,
    pub handicap: f64,
    pub price_size: PriceSize,
    pub bsp_liability: f64,
    pub side: Side,
    pub status: OrderStatus,
    pub persistence_type: PersistenceType,
    pub order_type: OrderType,
    pub placed_date: String,
    pub matched_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_price_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_matched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_remaining: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_lapsed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_cancelled: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_voided: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulator_auth_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulator_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClearedOrdersRequest {
    pub bet_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_date_range: Option<TimeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_item_description: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_record: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClearedOrdersResponse {
    pub cleared_orders: Vec<ClearedOrderSummary>,
    pub more_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearedOrderSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    pub market_id: String,
    pub selection_id: i64,
    pub handicap: f64,
    pub bet_id: String,
    pub placed_date: String,
    pub persistence_type: PersistenceType,
    pub order_type: OrderType,
    pub side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_description: Option<ItemDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_outcome: Option<String>,
    pub price_requested: f64,
    pub settled_date: String,
    pub last_matched_date: String,
    pub bet_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission: Option<f64>,
    pub price_matched: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_reduced: Option<bool>,
    pub size_settled: f64,
    pub profit: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_cancelled: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDescription {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type_desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_winners: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub each_way_divisor: Option<f64>,
}

// ============================================================================
// Account DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountFundsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<String>,
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
    pub from: String,
    pub to: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFundsResponse {
    pub transaction_id: String,
}

// ============================================================================
// API Error Response
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: String,
    pub message: String,
}
