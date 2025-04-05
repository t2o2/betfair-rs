use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub market_id: String,
    pub selection_id: u64,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub limit_order: Option<LimitOrder>,
    pub handicap: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderSide {
    Back,
    Lay,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    Limit,
    MarketOnClose,
    LimitOnClose,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LimitOrder {
    pub size: f64,
    pub price: f64,
    #[serde(rename = "persistenceType", skip_serializing_if = "Option::is_none")]
    pub persistence_type: Option<PersistenceType>,
    #[serde(rename = "timeInForce", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForceType>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PersistenceType {
    Lapse,
    Persist,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForceType {
    #[serde(rename = "FILL_OR_KILL")]
    FillOrKill,
    #[serde(rename = "GOOD_TILL_CANCEL")]
    GoodTillCancel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceOrdersRequest {
    #[serde(rename = "marketId")]
    pub market_id: String,
    pub instructions: Vec<PlaceInstruction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceInstruction {
    #[serde(rename = "selectionId")]
    pub selection_id: u64,
    pub handicap: f64,
    pub side: OrderSide,
    #[serde(rename = "orderType", skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    #[serde(rename = "limitOrder", skip_serializing_if = "Option::is_none")]
    pub limit_order: Option<LimitOrder>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceOrdersResponse {
    pub status: String,
    #[serde(rename = "marketId")]
    pub market_id: String,
    #[serde(rename = "instructionReports")]
    pub instruction_reports: Vec<InstructionReport>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: T,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstructionReport {
    pub status: String,
    pub instruction: PlaceInstruction,
    #[serde(rename = "betId")]
    pub bet_id: Option<String>,
    #[serde(rename = "placedDate")]
    pub placed_date: Option<String>,
    #[serde(rename = "averagePriceMatched")]
    pub average_price_matched: Option<f64>,
    #[serde(rename = "sizeMatched")]
    pub size_matched: Option<f64>,
    #[serde(rename = "orderStatus")]
    pub order_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CancelOrdersRequest {
    #[serde(rename = "marketId")]
    pub market_id: String,
    pub instructions: Vec<CancelInstruction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CancelInstruction {
    #[serde(rename = "betId")]
    pub bet_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CancelOrdersResponse {
    pub status: String,
    #[serde(rename = "marketId")]
    pub market_id: String,
    #[serde(rename = "instructionReports")]
    pub instruction_reports: Vec<CancelInstructionReport>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CancelInstructionReport {
    pub status: String,
    #[serde(rename = "betId")]
    pub bet_id: Option<String>,
    #[serde(rename = "cancelledDate")]
    pub cancelled_date: Option<String>,
    #[serde(rename = "orderStatus")]
    pub order_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderStatusResponse {
    pub bet_id: String,
    pub market_id: String,
    pub selection_id: u64,
    pub side: OrderSide,
    pub order_status: String,
    pub placed_date: Option<String>,
    pub matched_date: Option<String>,
    pub average_price_matched: Option<f64>,
    pub size_matched: Option<f64>,
    pub size_remaining: Option<f64>,
    pub size_lapsed: Option<f64>,
    pub size_cancelled: Option<f64>,
    pub size_voided: Option<f64>,
    pub price_requested: Option<f64>,
    pub price_reduced: Option<bool>,
    pub persistence_type: Option<PersistenceType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListCurrentOrdersRequest {
    #[serde(rename = "betIds")]
    pub bet_ids: Option<Vec<String>>,
    #[serde(rename = "marketIds")]
    pub market_ids: Option<Vec<String>>,
    #[serde(rename = "orderProjection")]
    pub order_projection: Option<String>,
    #[serde(rename = "placedDateRange")]
    pub placed_date_range: Option<TimeRange>,
    #[serde(rename = "dateRange")]
    pub date_range: Option<TimeRange>,
    #[serde(rename = "orderBy")]
    pub order_by: Option<String>,
    #[serde(rename = "sortDir")]
    pub sort_dir: Option<String>,
    #[serde(rename = "fromRecord")]
    pub from_record: Option<i32>,
    #[serde(rename = "recordCount")]
    pub record_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListCurrentOrdersResponse {
    #[serde(rename = "currentOrders")]
    pub orders: Vec<CurrentOrderSummary>,
    #[serde(rename = "moreAvailable")]
    pub more_available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentOrderSummary {
    #[serde(rename = "betId")]
    pub bet_id: String,
    #[serde(rename = "marketId")]
    pub market_id: String,
    #[serde(rename = "selectionId")]
    pub selection_id: u64,
    pub handicap: f64,
    #[serde(rename = "priceSize")]
    pub price_size: PriceSize,
    #[serde(rename = "bspLiability")]
    pub bsp_liability: f64,
    pub side: OrderSide,
    pub status: String,
    #[serde(rename = "persistenceType")]
    pub persistence_type: PersistenceType,
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    #[serde(rename = "placedDate")]
    pub placed_date: String,
    #[serde(rename = "averagePriceMatched")]
    pub average_price_matched: f64,
    #[serde(rename = "sizeMatched")]
    pub size_matched: f64,
    #[serde(rename = "sizeRemaining")]
    pub size_remaining: f64,
    #[serde(rename = "sizeLapsed")]
    pub size_lapsed: f64,
    #[serde(rename = "sizeCancelled")]
    pub size_cancelled: f64,
    #[serde(rename = "sizeVoided")]
    pub size_voided: f64,
    #[serde(rename = "regulatorCode")]
    pub regulator_code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceSize {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListClearedOrdersRequest {
    #[serde(rename = "betStatus")]
    pub bet_status: String,
    #[serde(rename = "eventTypeIds")]
    pub event_type_ids: Option<Vec<String>>,
    #[serde(rename = "eventIds")]
    pub event_ids: Option<Vec<String>>,
    #[serde(rename = "marketIds")]
    pub market_ids: Option<Vec<String>>,
    #[serde(rename = "runnerIds")]
    pub runner_ids: Option<Vec<u64>>,
    #[serde(rename = "betIds")]
    pub bet_ids: Option<Vec<String>>,
    #[serde(rename = "side")]
    pub side: Option<OrderSide>,
    #[serde(rename = "settledDateRange")]
    pub settled_date_range: Option<TimeRange>,
    #[serde(rename = "groupBy")]
    pub group_by: Option<String>,
    #[serde(rename = "includeItemDescription")]
    pub include_item_description: Option<bool>,
    #[serde(rename = "fromRecord")]
    pub from_record: Option<i32>,
    #[serde(rename = "recordCount")]
    pub record_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListClearedOrdersResponse {
    #[serde(rename = "clearedOrders")]
    pub cleared_orders: Vec<ClearedOrderSummary>,
    #[serde(rename = "moreAvailable")]
    pub more_available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClearedOrderSummary {
    #[serde(rename = "betId")]
    pub bet_id: String,
    #[serde(rename = "eventId")]
    pub event_id: String,
    #[serde(rename = "eventTypeId")]
    pub event_type_id: String,
    pub handicap: f64,
    #[serde(rename = "marketId")]
    pub market_id: String,
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    #[serde(rename = "persistenceType")]
    pub persistence_type: PersistenceType,
    #[serde(rename = "placedDate")]
    pub placed_date: String,
    #[serde(rename = "priceRequested")]
    pub price_requested: f64,
    #[serde(rename = "selectionId")]
    pub selection_id: u64,
    #[serde(rename = "settledDate")]
    pub settled_date: String,
    pub side: OrderSide,
}

impl Order {
    pub fn new(market_id: String, selection_id: u64, side: OrderSide, price: f64, size: f64, tif: Option<TimeInForceType>) -> Self {
        Self {
            market_id,
            selection_id,
            side,
            order_type: OrderType::Limit,
            limit_order: Some(LimitOrder {
                size,
                price,
                persistence_type: Some(PersistenceType::Persist),
                time_in_force: tif,
            }),
            handicap: 0.0,
        }
    }

    pub fn to_place_instruction(&self) -> PlaceInstruction {
        PlaceInstruction {
            selection_id: self.selection_id,
            handicap: self.handicap,
            side: self.side.clone(),
            order_type: Some(self.order_type.clone()),
            limit_order: self.limit_order.clone(),
        }
    }
}
