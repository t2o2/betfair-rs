use super::common::{
    BetOutcome, OrderStatus, OrderType, PersistenceType, PriceSize, Side, TimeInForce, TimeRange,
};
use super::market::MarketVersion;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Simple order structure for placing orders
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub market_id: String,
    pub selection_id: u64,
    pub side: Side,
    pub order_type: OrderType,
    pub limit_order: Option<LimitOrder>,
    #[serde(with = "super::decimal_serde")]
    pub handicap: Decimal,
}

impl Order {
    pub fn to_place_instruction(&self) -> PlaceInstruction {
        PlaceInstruction {
            order_type: self.order_type.clone(),
            selection_id: self.selection_id as i64,
            handicap: Some(self.handicap),
            side: self.side.clone(),
            limit_order: self.limit_order.clone(),
            limit_on_close_order: None,
            market_on_close_order: None,
            customer_order_ref: None,
        }
    }
}

// Order status response for tracking order status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderStatusResponse {
    pub bet_id: String,
    pub market_id: String,
    pub selection_id: i64,
    pub side: Side,
    pub order_status: String,
    pub placed_date: Option<String>,
    pub matched_date: Option<String>,
    #[serde(with = "super::decimal_serde::option")]
    pub average_price_matched: Option<Decimal>,
    #[serde(with = "super::decimal_serde::option")]
    pub size_matched: Option<Decimal>,
    #[serde(with = "super::decimal_serde::option")]
    pub size_remaining: Option<Decimal>,
    #[serde(with = "super::decimal_serde::option")]
    pub size_lapsed: Option<Decimal>,
    #[serde(with = "super::decimal_serde::option")]
    pub size_cancelled: Option<Decimal>,
    #[serde(with = "super::decimal_serde::option")]
    pub size_voided: Option<Decimal>,
    #[serde(with = "super::decimal_serde::option")]
    pub profit: Option<Decimal>,
}

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
pub struct PlaceInstruction {
    pub order_type: OrderType,
    pub selection_id: i64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub handicap: Option<Decimal>,
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
    #[serde(with = "super::decimal_serde")]
    pub size: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub price: Decimal,
    pub persistence_type: PersistenceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub min_fill_size: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_target_type: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub bet_target_size: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOnCloseOrder {
    #[serde(with = "super::decimal_serde")]
    pub liability: Decimal,
    #[serde(with = "super::decimal_serde")]
    pub price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOnCloseOrder {
    #[serde(with = "super::decimal_serde")]
    pub liability: Decimal,
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
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub average_price_matched: Option<Decimal>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_matched: Option<Decimal>,
}

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
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "sizeCancelled")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_reduction: Option<Decimal>,
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
    #[serde(with = "super::decimal_serde::option")]
    pub size_cancelled: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCurrentOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_projection: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub handicap: Option<Decimal>,
    pub price_size: PriceSize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub bsp_liability: Option<Decimal>,
    pub side: Side,
    pub status: OrderStatus,
    pub persistence_type: PersistenceType,
    pub order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placed_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub average_price_matched: Option<Decimal>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_order_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClearedOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_status: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub handicap: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placed_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_type: Option<PersistenceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_description: Option<ItemDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_outcome: Option<BetOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub price_requested: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_matched_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bet_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub commission: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub price_matched: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_reduced: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_settled: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub profit: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub size_cancelled: Option<Decimal>,
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
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::decimal_serde::option")]
    pub each_way_divisor: Option<Decimal>,
}
