use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Pending,
    ExecutionComplete,
    Executable,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Back,
    Lay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    #[serde(rename = "MARKET_ON_CLOSE")]
    MarketOnClose,
    #[serde(rename = "LIMIT_ON_CLOSE")]
    LimitOnClose,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketStatus {
    Inactive,
    Open,
    Suspended,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunnerStatus {
    Active,
    Winner,
    Loser,
    Removed,
    #[serde(rename = "REMOVED_VACANT")]
    RemovedVacant,
    Hidden,
    Placed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PriceData {
    #[serde(rename = "SP_AVAILABLE")]
    SpAvailable,
    #[serde(rename = "SP_TRADED")]
    SpTraded,
    #[serde(rename = "EX_BEST_OFFERS")]
    ExBestOffers,
    #[serde(rename = "EX_ALL_OFFERS")]
    ExAllOffers,
    #[serde(rename = "EX_TRADED")]
    ExTraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketBettingType {
    Odds,
    Line,
    Range,
    #[serde(rename = "ASIAN_HANDICAP_DOUBLE_LINE")]
    AsianHandicapDoubleLine,
    #[serde(rename = "ASIAN_HANDICAP_SINGLE_LINE")]
    AsianHandicapSingleLine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketProjection {
    Competition,
    Event,
    #[serde(rename = "EVENT_TYPE")]
    EventType,
    #[serde(rename = "MARKET_START_TIME")]
    MarketStartTime,
    #[serde(rename = "MARKET_DESCRIPTION")]
    MarketDescription,
    #[serde(rename = "RUNNER_DESCRIPTION")]
    RunnerDescription,
    #[serde(rename = "RUNNER_METADATA")]
    RunnerMetadata,
}

// Remove duplicate - PriceData already exists above

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderProjection {
    All,
    Executable,
    #[serde(rename = "EXECUTION_COMPLETE")]
    ExecutionComplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchProjection {
    #[serde(rename = "NO_ROLLUP")]
    NoRollup,
    #[serde(rename = "ROLLED_UP_BY_PRICE")]
    RolledUpByPrice,
    #[serde(rename = "ROLLED_UP_BY_AVG_PRICE")]
    RolledUpByAvgPrice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstructionReportStatus {
    Success,
    Failure,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstructionReportErrorCode {
    #[serde(rename = "INVALID_BET_SIZE")]
    InvalidBetSize,
    #[serde(rename = "INVALID_RUNNER")]
    InvalidRunner,
    #[serde(rename = "BET_TAKEN_OR_LAPSED")]
    BetTakenOrLapsed,
    #[serde(rename = "BET_IN_PROGRESS")]
    BetInProgress,
    #[serde(rename = "RUNNER_REMOVED")]
    RunnerRemoved,
    #[serde(rename = "MARKET_NOT_OPEN_FOR_BETTING")]
    MarketNotOpenForBetting,
    #[serde(rename = "LOSS_LIMIT_EXCEEDED")]
    LossLimitExceeded,
    #[serde(rename = "MARKET_NOT_OPEN_FOR_BSP_BETTING")]
    MarketNotOpenForBspBetting,
    #[serde(rename = "INVALID_PRICE_EDIT")]
    InvalidPriceEdit,
    #[serde(rename = "INVALID_ODDS")]
    InvalidOdds,
    #[serde(rename = "INSUFFICIENT_FUNDS")]
    InsufficientFunds,
    #[serde(rename = "INVALID_PERSISTENCE_TYPE")]
    InvalidPersistenceType,
    #[serde(rename = "ERROR_IN_MATCHER")]
    ErrorInMatcher,
    #[serde(rename = "INVALID_BACK_LAY_COMBINATION")]
    InvalidBackLayCombination,
    #[serde(rename = "ERROR_IN_ORDER")]
    ErrorInOrder,
    #[serde(rename = "INVALID_BID_TYPE")]
    InvalidBidType,
    #[serde(rename = "INVALID_BET_ID")]
    InvalidBetId,
    #[serde(rename = "CANCELLED_NOT_PLACED")]
    CancelledNotPlaced,
    #[serde(rename = "RELATED_ACTION_FAILED")]
    RelatedActionFailed,
    #[serde(rename = "NO_ACTION_REQUIRED")]
    NoActionRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BetOutcome {
    Won,
    Lost,
    Void,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Wallet {
    Uk,
    Australian,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSize {
    pub price: f64,
    pub size: f64,
}
