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
use crate::order::{
    Order, PlaceOrdersRequest, PlaceOrdersResponse, JsonRpcResponse, JsonRpcRequest, 
    CancelOrdersRequest, CancelOrdersResponse, CancelInstruction, OrderStatusResponse,
    ListCurrentOrdersRequest, ListCurrentOrdersResponse, ListClearedOrdersRequest, 
    ListClearedOrdersResponse
};
use crate::account::{AccountFundsResponse, GetAccountFundsRequest, GetAccountFundsResponse};

const LOGIN_URL: &str = "https://identitysso-cert.betfair.com/api/certlogin";
const BETTING_URL: &str = "https://api.betfair.com/exchange/betting/json-rpc/v1";
const ACCOUNT_URL: &str = "https://api.betfair.com/exchange/account/json-rpc/v1";

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
            .post(BETTING_URL)
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

    pub async fn cancel_order(&self, market_id: String, bet_id: String) -> Result<CancelOrdersResponse> {
        let session_token = self.session_token.as_ref().ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        
        let mut headers = HeaderMap::new();
        headers.insert("X-Application", self.config.betfair.api_key.parse()?);
        headers.insert("X-Authentication", session_token.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let request = CancelOrdersRequest {
            market_id: market_id.clone(),
            instructions: vec![CancelInstruction { bet_id: bet_id.clone() }],
        };

        let jsonrpc_request = vec![JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "SportsAPING/v1.0/cancelOrders".to_string(),
            params: request,
            id: 1,
        }];

        info!("Cancel order request: {}", serde_json::to_string_pretty(&jsonrpc_request).unwrap());

        let mut response = self.client
            .post(BETTING_URL)
            .headers(headers)
            .json(&jsonrpc_request)
            .send()?;

        info!("Cancel order response status: {}", response.status());
        
        let response_text = response.text()?;
        info!("Cancel order response body: {}", response_text);

        let raw_response: Vec<JsonRpcResponse<CancelOrdersResponse>> = serde_json::from_str(&response_text)?;
        let response = raw_response[0].result.to_owned();
        Ok(response)
    }

    pub async fn get_order_status(&self, bet_id: String) -> Result<Option<OrderStatusResponse>> {
        // First try to find the order in current orders
        let current_orders = self.list_current_orders(Some(vec![bet_id.clone()]), None).await?;
        
        if let Some(order) = current_orders.orders.first() {
            return Ok(Some(OrderStatusResponse {
                bet_id: order.bet_id.clone(),
                market_id: order.market_id.clone(),
                selection_id: order.selection_id,
                side: order.side.clone(),
                order_status: order.status.clone(),
                placed_date: Some(order.placed_date.clone()),
                matched_date: None,
                average_price_matched: Some(order.average_price_matched),
                size_matched: Some(order.size_matched),
                size_remaining: Some(order.size_remaining),
                size_lapsed: Some(order.size_lapsed),
                size_cancelled: Some(order.size_cancelled),
                size_voided: Some(order.size_voided),
                price_requested: Some(order.price_size.price),
                price_reduced: None,
                persistence_type: Some(order.persistence_type.clone()),
            }));
        }

        // If not found in current orders, try cleared orders
        let cleared_orders = self.list_cleared_orders(Some(vec![bet_id.clone()])).await?;
        
        if let Some(order) = cleared_orders.cleared_orders.first() {
            return Ok(Some(OrderStatusResponse {
                bet_id: order.bet_id.clone(),
                market_id: order.market_id.clone(),
                selection_id: order.selection_id,
                side: order.side.clone(),
                order_status: order.bet_status.clone(),
                placed_date: Some(order.placed_date.clone()),
                matched_date: Some(order.settled_date.clone()),
                average_price_matched: order.price_matched,
                size_matched: order.size_settled,
                size_remaining: None,
                size_lapsed: order.size_lapsed,
                size_cancelled: order.size_cancelled,
                size_voided: order.size_voided,
                price_requested: order.price_requested,
                price_reduced: None,
                persistence_type: None,
            }));
        }

        Ok(None)
    }

    async fn list_current_orders(&self, bet_ids: Option<Vec<String>>, market_ids: Option<Vec<String>>) -> Result<ListCurrentOrdersResponse> {
        let session_token = self.session_token.as_ref().ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        
        let mut headers = HeaderMap::new();
        headers.insert("X-Application", self.config.betfair.api_key.parse()?);
        headers.insert("X-Authentication", session_token.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let request = ListCurrentOrdersRequest {
            bet_ids,
            market_ids,
            order_projection: None,
            placed_date_range: None,
            date_range: None,
            order_by: None,
            sort_dir: None,
            from_record: None,
            record_count: None,
        };

        let jsonrpc_request = vec![JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "SportsAPING/v1.0/listCurrentOrders".to_string(),
            params: request,
            id: 1,
        }];

        info!("List current orders request: {}", serde_json::to_string_pretty(&jsonrpc_request).unwrap());

        let mut response = self.client
            .post(BETTING_URL)
            .headers(headers)
            .json(&jsonrpc_request)
            .send()?;

        info!("List current orders response status: {}", response.status());
        
        let response_text = response.text()?;
        info!("List current orders response body: {}", response_text);

        let raw_response: Vec<JsonRpcResponse<ListCurrentOrdersResponse>> = serde_json::from_str(&response_text)?;
        let response = raw_response[0].result.to_owned();
        Ok(response)
    }

    async fn list_cleared_orders(&self, bet_ids: Option<Vec<String>>) -> Result<ListClearedOrdersResponse> {
        let session_token = self.session_token.as_ref().ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        
        let mut headers = HeaderMap::new();
        headers.insert("X-Application", self.config.betfair.api_key.parse()?);
        headers.insert("X-Authentication", session_token.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let request = ListClearedOrdersRequest {
            bet_status: "SETTLED".to_string(),
            event_type_ids: None,
            event_ids: None,
            market_ids: None,
            runner_ids: None,
            bet_ids,
            side: None,
            settled_date_range: None,
            group_by: None,
            include_item_description: None,
            from_record: None,
            record_count: None,
        };

        let jsonrpc_request = vec![JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "SportsAPING/v1.0/listClearedOrders".to_string(),
            params: request,
            id: 1,
        }];

        info!("List cleared orders request: {}", serde_json::to_string_pretty(&jsonrpc_request).unwrap());

        let mut response = self.client
            .post(BETTING_URL)
            .headers(headers)
            .json(&jsonrpc_request)
            .send()?;

        info!("List cleared orders response status: {}", response.status());
        
        let response_text = response.text()?;
        info!("List cleared orders response body: {}", response_text);

        let raw_response: Vec<JsonRpcResponse<ListClearedOrdersResponse>> = serde_json::from_str(&response_text)?;
        let response = raw_response[0].result.to_owned();
        Ok(response)
    }

    pub async fn get_account_funds(&self) -> Result<GetAccountFundsResponse> {
        let session_token = self.session_token.as_ref().ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        
        let mut headers = HeaderMap::new();
        headers.insert("X-Application", self.config.betfair.api_key.parse()?);
        headers.insert("X-Authentication", session_token.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let request = GetAccountFundsRequest {
            wallet: None,
        };

        let jsonrpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "AccountAPING/v1.0/getAccountFunds".to_string(),
            params: request,
            id: 1,
        };

        info!("Get account funds request: {}", serde_json::to_string_pretty(&jsonrpc_request).unwrap());

        let mut response = self.client
            .post(ACCOUNT_URL)
            .headers(headers)
            .json(&jsonrpc_request)
            .send()?;

        info!("Get account funds response status: {}", response.status());
        
        let response_text = response.text()?;
        info!("Get account funds response body: {}", response_text);

        let raw_response: JsonRpcResponse<AccountFundsResponse> = serde_json::from_str(&response_text)?;
        let response = raw_response.result.to_owned();
        
        Ok(GetAccountFundsResponse {
            available_to_bet_balance: response.available_to_bet_balance,
            exposure: response.exposure,
            retained_commission: response.retained_commission,
            exposure_limit: response.exposure_limit,
            discount_rate: response.discount_rate,
            points_balance: response.points_balance,
            wallet: response.wallet,
        })
    }
}