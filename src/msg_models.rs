// {"op":"mcm","id":1,"initialClk":"ggKnloPPDYQClqbMuA34AfS/5s0N","clk":"AAAAAAAA","conflateMs":0,"heartbeatMs":5000,"pt":1742146796884,"ct":"SUB_IMAGE"}
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketChangeMessage {
    pub op: String,
    pub id: i64,
    #[serde(rename = "initialClk")]
    pub initial_clk: String,
    pub clk: String,
    #[serde(rename = "conflateMs")]
    pub conflate_ms: i64,
    #[serde(rename = "heartbeatMs")] 
    pub heartbeat_ms: i64,
    pub pt: i64,
    pub ct: String
}
