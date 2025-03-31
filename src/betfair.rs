use anyhow::{Ok, Result};
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::fs;
use std::collections::HashMap;
use tracing::info;
use crate::config::Config;
use crate::msg_model::LoginResponse;
use crate::streamer::BetfairStreamer;
use crate::orderbook::Orderbook;
use crate::order::{Order, PlaceOrdersRequest, PlaceOrdersResponse, JsonRpcResponse, JsonRpcRequest};

const LOGIN_URL: &str = "https://identitysso-cert.betfair.com/api/certlogin";
const PLACE_ORDERS_URL: &str = "https://api.betfair.com/exchange/betting/json-rpc/v1";

#[allow(dead_code)]
pub struct BetfairClient {
    client: Client,
    config: Config,
    session_token: Option<String>,
    streamer: Option<BetfairStreamer>,
}

impl BetfairClient {
    #[allow(dead_code)]
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
            session_token: None,
            streamer: None,
        }
    }

    #[allow(dead_code)]
    pub async fn login(&mut self) -> Result<()> {
        let pem_contents = fs::read(&self.config.betfair.pfx_path)?;
        let identity = reqwest::Identity::from_pkcs12_der(&pem_contents, &self.config.betfair.pfx_password)?;
        let mut headers = HeaderMap::new();

        headers.insert("X-Application", self.config.betfair.api_key.parse()?);
        headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);
    
        let client = Client::builder().identity(identity).build()?;
        let form = [
            ("username", self.config.betfair.username.as_str()),
            ("password", self.config.betfair.password.as_str()),
        ];
    
        let response: LoginResponse = client
            .post(LOGIN_URL) 
            .headers(headers)
            .header(
                "X-Application",
                format!("schroedinger_{}", rand::random::<u128>()),
            )
            .form(&form)
            .send()?
            .json()?;
    
        match response.session_token {
            Some(token) => {
                self.session_token = Some(token);
                self.streamer = Some(BetfairStreamer::new(self.config.betfair.api_key.clone(), self.session_token.clone().unwrap()));
                Ok(())
            }
            None => Err(anyhow::anyhow!("loginStatus: {}", response.login_status)),
        }
    }

    pub async fn get_session_token(&self) -> Option<String> {
        self.session_token.clone()
    }

    pub async fn subscribe_to_markets(&mut self, market_ids: Vec<String>, levels: usize) -> Result<()> {
        if levels < 1 || levels > 10 {
            return Err(anyhow::anyhow!("Levels must be between 1 and 10, got {}", levels));
        }
        let streamer = self.streamer.as_mut().unwrap();
        for market_id in market_ids {
            streamer.subscribe(market_id, levels).await?;
        }
        Ok(())
    }

    pub async fn connect(&mut self) -> Result<()> {
        let streamer = self.streamer.as_mut().unwrap();
        streamer.set_callback(Self::callback);
        streamer.connect_betfair_tls_stream().await?;
        Ok(())
    }

    pub fn set_orderbook_callback<F>(&mut self, callback: F)
    where
        F: Fn(String, HashMap<String, Orderbook>) + Send + Sync + 'static,
    {
        let streamer = self.streamer.as_mut().unwrap();
        streamer.set_orderbook_callback(callback);
    }

    pub async fn start_listening(&mut self) -> Result<()> {
        let streamer = self.streamer.as_mut().unwrap();
        streamer.start().await?;
        Ok(())
    }

    fn callback(message: String) {
        info!("callback message: {}", message);
        let json_data: serde_json::Value = serde_json::from_str(&message).unwrap();
        let op = json_data["op"].as_str();
        if op == Some("mcm") {
            let ct = json_data["ct"].as_str();
            if ct == Some("HEARTBEAT") {
                info!("Heartbeat received");
            }
        }
    }

    pub async fn place_order(&self, order: Order) -> Result<PlaceOrdersResponse> {
        let session_token = self.session_token.as_ref().ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        
        let mut headers = HeaderMap::new();
        headers.insert("X-Application", self.config.betfair.api_key.parse()?);
        headers.insert("X-Authentication", session_token.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let request = PlaceOrdersRequest {
            market_id: order.market_id.clone(),
            instructions: vec![order.to_place_instruction()],
        };

        let jsonrpc_request = vec![JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "SportsAPING/v1.0/placeOrders".to_string(),
            params: request,
            id: 1,
        }];

        info!("Place order request: {}", serde_json::to_string_pretty(&jsonrpc_request).unwrap());

        let mut response = self.client
            .post(PLACE_ORDERS_URL)
            .headers(headers)
            .json(&jsonrpc_request)
            .send()?;

        info!("Place order response status: {}", response.status());
        
        let response_text = response.text()?;
        info!("Place order response body: {}", response_text);

        let raw_response: Vec<JsonRpcResponse<PlaceOrdersResponse>> = serde_json::from_str(&response_text)?;
        let response = raw_response[0].result.to_owned();
        Ok(response)
    }
}