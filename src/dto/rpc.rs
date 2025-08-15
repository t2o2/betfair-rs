use serde::{Deserialize, Serialize};

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
    pub result: Option<T>,
    pub error: Option<ApiError>,
    pub id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    #[serde(rename = "sessionToken")]
    pub session_token: String,
    #[serde(rename = "loginStatus")]
    pub login_status: String,
}

// Alternative login response format (for cert login)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertLoginResponse {
    pub session_token: String,
    pub login_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: String,
    pub message: String,
}
