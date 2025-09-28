use serde::{Deserialize, Serialize};

// Note: These are DTO versions of config structures
// The actual config implementation is in src/config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetfairCredentialsDto {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub pem_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetfairConfigDto {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub pem_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDto {
    pub betfair: BetfairConfigDto,
}
