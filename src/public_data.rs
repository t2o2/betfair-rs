use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const PUBLIC_API_URL: &str = "https://api.betfair.com/exchange/betting/rest/v1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sport {
    pub event_type: EventType,
    pub market_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventType {
    pub id: String,
    pub name: String,
}

pub struct PublicDataClient {
    client: Client,
    app_key: Option<String>,
}

impl PublicDataClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            app_key: None,
        }
    }

    pub fn with_app_key(app_key: String) -> Self {
        Self {
            client: Client::new(),
            app_key: Some(app_key),
        }
    }

    pub fn list_sports(&self) -> Result<Vec<Sport>> {
        let url = format!("{PUBLIC_API_URL}/listEventTypes/");
        
        let mut request = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        
        if let Some(ref app_key) = self.app_key {
            request = request.header("X-Application", app_key);
        }
        
        let mut response = request
            .json(&serde_json::json!({
                "filter": {},
                "locale": "en"
            }))
            .send()?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "Failed to fetch sports: HTTP {} - {}",
                response.status(),
                error_text
            ));
        }

        let sports: Vec<Sport> = response.json()?;
        Ok(sports)
    }
}

impl Default for PublicDataClient {
    fn default() -> Self {
        Self::new()
    }
}