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
    config: Box<Config>,
    session_token: Option<String>,
    streamer: Option<Box<BetfairStreamer>>,
}

impl BetfairClient {
    #[allow(dead_code)]
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config: Box::new(config),
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
                self.streamer = Some(Box::new(BetfairStreamer::new(self.config.betfair.api_key.clone(), self.session_token.clone().unwrap())));
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

    async fn make_api_request<T, U>(&self, url: &str, method: &str, params: T) -> Result<U> 
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        let session_token = self.session_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        
        let mut headers = HeaderMap::with_capacity(3);
        headers.insert("X-Application", self.config.betfair.api_key.parse()?);
        headers.insert("X-Authentication", session_token.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let jsonrpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        if tracing::enabled!(tracing::Level::INFO) {
            info!("API request: {}", serde_json::to_string_pretty(&jsonrpc_request)?);
        }

        let mut response = self.client
            .post(url)
            .headers(headers)
            .json(&jsonrpc_request)
            .send()?;

        if tracing::enabled!(tracing::Level::INFO) {
            info!("API response status: {}", response.status());
        }
        
        let response_text = response.text()?;
        
        if tracing::enabled!(tracing::Level::INFO) {
            info!("API response body: {}", serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&response_text)?)?);
        }

        let raw_response: JsonRpcResponse<U> = serde_json::from_str(&response_text)?;
        Ok(raw_response.result)
    }

    async fn make_betting_request<T, U>(&self, method: &str, params: T) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        self.make_api_request(BETTING_URL, &format!("SportsAPING/v1.0/{}", method), params).await
    }

    async fn make_account_request<T, U>(&self, method: &str, params: T) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        self.make_api_request(ACCOUNT_URL, &format!("AccountAPING/v1.0/{}", method), params).await
    }

    pub async fn place_order(&self, order: Order) -> Result<PlaceOrdersResponse> {
        let request = PlaceOrdersRequest {
            market_id: order.market_id.clone(),
            instructions: vec![order.to_place_instruction()],
        };
        self.make_betting_request("placeOrders", request).await
    }

    pub async fn cancel_order(&self, market_id: String, bet_id: String) -> Result<CancelOrdersResponse> {
        let request = CancelOrdersRequest {
            market_id,
            instructions: vec![CancelInstruction { bet_id }],
        };
        self.make_betting_request("cancelOrders", request).await
    }

    async fn list_current_orders(&self, bet_ids: Option<Vec<String>>, market_ids: Option<Vec<String>>) -> Result<ListCurrentOrdersResponse> {
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
        self.make_betting_request("listCurrentOrders", request).await
    }

    async fn list_cleared_orders(&self, bet_ids: Option<Vec<String>>) -> Result<ListClearedOrdersResponse> {
        let statuses = ["LAPSED", "SETTLED", "CANCELLED", "VOIDED"];
        let mut all_orders = Vec::with_capacity(100); // Pre-allocate with reasonable capacity
        let mut more_available = false;

        for status in statuses {
            let request = ListClearedOrdersRequest {
                bet_status: status.to_string(),
                event_type_ids: None,
                event_ids: None,
                market_ids: None,
                runner_ids: None,
                bet_ids: bet_ids.as_ref().cloned(),
                side: None,
                settled_date_range: None,
                group_by: None,
                include_item_description: None,
                from_record: None,
                record_count: None,
            };
            
            let response: ListClearedOrdersResponse = self.make_betting_request("listClearedOrders", request).await?;
            
            if !response.cleared_orders.is_empty() {
                all_orders.extend(response.cleared_orders);
                more_available = response.more_available;
                break; // Found orders, no need to continue searching other statuses
            }
        }

        Ok(ListClearedOrdersResponse {
            cleared_orders: all_orders,
            more_available,
        })
    }

    pub async fn get_account_funds(&self) -> Result<GetAccountFundsResponse> {
        let request = GetAccountFundsRequest { wallet: None };
        let response: AccountFundsResponse = self.make_account_request("getAccountFunds", request).await?;
        
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

    pub async fn get_order_status(&self, bet_ids: Vec<String>) -> Result<HashMap<String, OrderStatusResponse>> {
        let mut results = HashMap::with_capacity(bet_ids.len());
        let mut remaining_bet_ids = bet_ids;

        // First check current orders
        let current_orders = self.list_current_orders(Some(remaining_bet_ids.clone()), None).await?;
        
        for order in current_orders.orders {
            let bet_id = order.bet_id.clone();
            results.insert(bet_id.clone(), OrderStatusResponse {
                bet_id,
                market_id: order.market_id,
                selection_id: order.selection_id,
                side: order.side,
                order_status: order.status,
                placed_date: Some(order.placed_date),
                matched_date: None,
                average_price_matched: Some(order.average_price_matched),
                size_matched: Some(order.size_matched),
                size_remaining: Some(order.size_remaining),
                size_lapsed: Some(order.size_lapsed),
                size_cancelled: Some(order.size_cancelled),
                size_voided: Some(order.size_voided),
                price_requested: Some(order.price_size.price),
                price_reduced: None,
                persistence_type: Some(order.persistence_type),
            });
            // Remove found bet_id from remaining list
            remaining_bet_ids.retain(|id| id != &order.bet_id);
        }

        // If there are remaining bet_ids, check cleared orders
        if !remaining_bet_ids.is_empty() {
            let cleared_orders = self.list_cleared_orders(Some(remaining_bet_ids)).await?;
            
            for order in cleared_orders.cleared_orders {
                let bet_id = order.bet_id.clone();
                results.insert(bet_id.clone(), OrderStatusResponse {
                    bet_id,
                    market_id: order.market_id,
                    selection_id: order.selection_id,
                    side: order.side,
                    order_status: "SETTLED".to_string(),
                    placed_date: Some(order.placed_date),
                    matched_date: Some(order.settled_date),
                    average_price_matched: None,
                    size_matched: None,
                    size_remaining: None,
                    size_lapsed: None,
                    size_cancelled: None,
                    size_voided: None,
                    price_requested: Some(order.price_requested),
                    price_reduced: None,
                    persistence_type: Some(order.persistence_type),
                });
            }
        }

        Ok(results)
    }
}