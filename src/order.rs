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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderSide {
    Back,
    Lay,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(rename = "persistenceType")]
    pub persistence_type: PersistenceType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum PersistenceType {
    Lapse,
    Persist,
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
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
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

impl Order {
    pub fn new(market_id: String, selection_id: u64, side: OrderSide, price: f64, size: f64) -> Self {
        Self {
            market_id,
            selection_id,
            side,
            order_type: OrderType::Limit,
            limit_order: Some(LimitOrder {
                size,
                price,
                persistence_type: PersistenceType::Persist,
            }),
            handicap: 0.0,
        }
    }

    pub fn to_place_instruction(&self) -> PlaceInstruction {
        PlaceInstruction {
            selection_id: self.selection_id,
            handicap: self.handicap,
            side: self.side.clone(),
            order_type: self.order_type.clone(),
            limit_order: self.limit_order.clone(),
        }
    }
}
