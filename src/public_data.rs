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

        let mut request = self
            .client
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
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_data_client_new() {
        let client = PublicDataClient::new();
        assert!(client.app_key.is_none());
    }

    #[test]
    fn test_public_data_client_with_app_key() {
        let client = PublicDataClient::with_app_key("test_key".to_string());
        assert_eq!(client.app_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_public_data_client_default() {
        let client = PublicDataClient::default();
        assert!(client.app_key.is_none());
    }

    #[test]
    fn test_sport_struct() {
        let sport = Sport {
            event_type: EventType {
                id: "1".to_string(),
                name: "Soccer".to_string(),
            },
            market_count: 100,
        };

        assert_eq!(sport.event_type.id, "1");
        assert_eq!(sport.event_type.name, "Soccer");
        assert_eq!(sport.market_count, 100);
    }

    #[test]
    fn test_event_type_struct() {
        let event_type = EventType {
            id: "2".to_string(),
            name: "Tennis".to_string(),
        };

        assert_eq!(event_type.id, "2");
        assert_eq!(event_type.name, "Tennis");
    }

    #[test]
    fn test_sport_serialization() {
        let sport = Sport {
            event_type: EventType {
                id: "1".to_string(),
                name: "Soccer".to_string(),
            },
            market_count: 50,
        };

        let json = serde_json::to_string(&sport).unwrap();
        assert!(json.contains("\"eventType\""));
        assert!(json.contains("\"marketCount\":50"));
    }

    #[test]
    fn test_sport_deserialization() {
        let json = r#"{
            "eventType": {
                "id": "3",
                "name": "Golf"
            },
            "marketCount": 25
        }"#;

        let sport: Sport = serde_json::from_str(json).unwrap();
        assert_eq!(sport.event_type.id, "3");
        assert_eq!(sport.event_type.name, "Golf");
        assert_eq!(sport.market_count, 25);
    }
}
