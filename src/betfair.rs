use crate::dto::{
    GetAccountFundsRequest, GetAccountFundsResponse, AccountFundsResponse,
    CancelInstruction, CancelOrdersRequest, CancelOrdersResponse, 
    JsonRpcRequest, JsonRpcResponse,
    ListClearedOrdersRequest, ListClearedOrdersResponse, ListCurrentOrdersRequest,
    ListCurrentOrdersResponse, PlaceOrdersRequest, PlaceOrdersResponse,
    Order, OrderStatusResponse, Side,
};
use crate::dto::rpc::LoginResponse;
use crate::config::Config;
use crate::orderbook::Orderbook;
use crate::rate_limiter::BetfairRateLimiter;
use crate::retry::RetryPolicy;
use crate::streamer::BetfairStreamer;
use anyhow::{Ok, Result};
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use tracing::info;

const LOGIN_URL: &str = "https://identitysso-cert.betfair.com/api/certlogin";
const BETTING_URL: &str = "https://api.betfair.com/exchange/betting/json-rpc/v1";
const ACCOUNT_URL: &str = "https://api.betfair.com/exchange/account/json-rpc/v1";

#[allow(dead_code)]
pub struct BetfairClient {
    client: Client,
    config: Box<Config>,
    session_token: Option<String>,
    streamer: Option<Box<BetfairStreamer>>,
    retry_policy: RetryPolicy,
    rate_limiter: BetfairRateLimiter,
}

impl BetfairClient {
    #[allow(dead_code)]
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config: Box::new(config),
            session_token: None,
            streamer: None,
            retry_policy: RetryPolicy::default(),
            rate_limiter: BetfairRateLimiter::new(),
        }
    }

    #[allow(dead_code)]
    pub async fn login(&mut self) -> Result<()> {
        let api_key = self.config.betfair.api_key.clone();
        let username = self.config.betfair.username.clone();
        let password = self.config.betfair.password.clone();
        let pfx_path = self.config.betfair.pfx_path.clone();
        let pfx_password = self.config.betfair.pfx_password.clone();

        let result = self
            .retry_policy
            .retry(|| {
                let api_key = api_key.clone();
                let username = username.clone();
                let password = password.clone();
                let pfx_path = pfx_path.clone();
                let pfx_password = pfx_password.clone();

                async move {
                    // Read and create identity inside the retry closure
                    let pem_contents = fs::read(&pfx_path)?;
                    let identity =
                        reqwest::Identity::from_pkcs12_der(&pem_contents, &pfx_password)?;

                    let mut headers = HeaderMap::new();
                    headers.insert("X-Application", api_key.parse()?);
                    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);

                    let client = Client::builder().identity(identity).build()?;
                    let form = [
                        ("username", username.as_str()),
                        ("password", password.as_str()),
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

                    if response.login_status == "SUCCESS" {
                        Ok(response.session_token)
                    } else {
                        Err(anyhow::anyhow!("loginStatus: {}", response.login_status))
                    }
                }
            })
            .await?;

        self.session_token = Some(result.clone());
        self.streamer = Some(Box::new(BetfairStreamer::new(
            self.config.betfair.api_key.clone(),
            result,
        )));
        Ok(())
    }

    pub async fn get_session_token(&self) -> Option<String> {
        self.session_token.clone()
    }

    pub async fn subscribe_to_markets(
        &mut self,
        market_ids: Vec<String>,
        levels: usize,
    ) -> Result<()> {
        if !(1..=10).contains(&levels) {
            return Err(anyhow::anyhow!(
                "Levels must be between 1 and 10, got {}",
                levels
            ));
        }
        let streamer = self
            .streamer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Streamer not initialized. Please login first"))?;
        for market_id in market_ids {
            streamer.subscribe(market_id, levels).await?;
        }
        Ok(())
    }

    pub async fn connect(&mut self) -> Result<()> {
        let streamer = self
            .streamer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Streamer not initialized. Please login first"))?;
        streamer.connect_betfair_tls_stream().await?;
        Ok(())
    }

    pub fn set_orderbook_callback<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(String, HashMap<String, Orderbook>) + Send + Sync + 'static,
    {
        let streamer = self
            .streamer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Streamer not initialized. Please login first"))?;
        streamer.set_orderbook_callback(callback);
        Ok(())
    }

    pub fn set_orderupdate_callback<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(crate::msg_model::OrderChangeMessage) + Send + Sync + 'static,
    {
        let streamer = self
            .streamer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Streamer not initialized. Please login first"))?;
        streamer.set_orderupdate_callback(callback);
        Ok(())
    }

    pub async fn start_listening(&mut self) -> Result<()> {
        info!("Starting to listen to streams");
        let streamer = self
            .streamer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Streamer not initialized. Please login first"))?;
        streamer.start().await?;
        Ok(())
    }

    async fn make_api_request<T, U>(&self, url: &str, method: &str, params: T) -> Result<U>
    where
        T: serde::Serialize + Clone,
        U: serde::de::DeserializeOwned,
    {
        let session_token = self
            .session_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not logged in"))?
            .clone();

        let api_key = self.config.betfair.api_key.clone();
        let method_str = method.to_string();
        let url_str = url.to_string();

        self.retry_policy
            .retry(|| {
                let session_token = session_token.clone();
                let api_key = api_key.clone();
                let method_str = method_str.clone();
                let url_str = url_str.clone();
                let params = params.clone();
                let client = self.client.clone();

                async move {
                    let mut headers = HeaderMap::with_capacity(3);
                    headers.insert("X-Application", api_key.parse()?);
                    headers.insert("X-Authentication", session_token.parse()?);
                    headers.insert("Content-Type", "application/json".parse()?);

                    let jsonrpc_request = JsonRpcRequest {
                        jsonrpc: "2.0".to_string(),
                        method: method_str,
                        params,
                        id: 1,
                    };

                    if tracing::enabled!(tracing::Level::INFO) {
                        info!(
                            "API request: {}",
                            serde_json::to_string_pretty(&jsonrpc_request)?
                        );
                    }

                    let mut response = client
                        .post(&url_str)
                        .headers(headers)
                        .json(&jsonrpc_request)
                        .send()?;

                    if tracing::enabled!(tracing::Level::INFO) {
                        info!("API response status: {}", response.status());
                    }

                    let response_text = response.text()?;

                    if tracing::enabled!(tracing::Level::INFO) {
                        info!(
                            "API response body: {}",
                            serde_json::to_string_pretty(&serde_json::from_str::<
                                serde_json::Value,
                            >(
                                &response_text
                            )?)?
                        );
                    }

                    let raw_response: JsonRpcResponse<U> = serde_json::from_str(&response_text)?;
                    raw_response.result.ok_or_else(|| {
                        anyhow::anyhow!("No result in response: {:?}", raw_response.error)
                    })
                }
            })
            .await
    }

    async fn make_betting_request<T, U>(&self, method: &str, params: T) -> Result<U>
    where
        T: serde::Serialize + Clone,
        U: serde::de::DeserializeOwned,
    {
        // Apply rate limiting based on method type
        match method {
            "placeOrders" | "cancelOrders" | "replaceOrders" | "updateOrders" => {
                self.rate_limiter.acquire_for_transaction().await?;
            }
            "listMarketCatalogue" | "listMarketBook" | "listRunnerBook" => {
                self.rate_limiter.acquire_for_navigation().await?;
            }
            _ => {
                self.rate_limiter.acquire_for_data().await?;
            }
        }

        self.make_api_request(BETTING_URL, &format!("SportsAPING/v1.0/{method}"), params)
            .await
    }

    async fn make_account_request<T, U>(&self, method: &str, params: T) -> Result<U>
    where
        T: serde::Serialize + Clone,
        U: serde::de::DeserializeOwned,
    {
        // Account requests are generally data requests
        self.rate_limiter.acquire_for_data().await?;
        self.make_api_request(
            ACCOUNT_URL,
            &format!("AccountAPING/v1.0/{method}"),
            params,
        )
        .await
    }

    pub async fn place_order(&self, order: Order) -> Result<PlaceOrdersResponse> {
        let request = PlaceOrdersRequest {
            market_id: order.market_id.clone(),
            instructions: vec![order.to_place_instruction()],
            customer_ref: None,
            market_version: None,
            customer_strategy_ref: None,
            async_: None,
        };
        self.make_betting_request("placeOrders", request).await
    }

    pub async fn cancel_order(
        &self,
        market_id: String,
        bet_id: String,
    ) -> Result<CancelOrdersResponse> {
        let request = CancelOrdersRequest {
            market_id,
            instructions: vec![CancelInstruction { 
                bet_id,
                size_reduction: None,
            }],
            customer_ref: None,
        };
        self.make_betting_request("cancelOrders", request).await
    }

    async fn list_current_orders(
        &self,
        bet_ids: Option<Vec<String>>,
        market_ids: Option<Vec<String>>,
    ) -> Result<ListCurrentOrdersResponse> {
        let request = ListCurrentOrdersRequest {
            bet_ids,
            market_ids,
            order_projection: None,
            customer_order_refs: None,
            customer_strategy_refs: None,
            date_range: None,
            order_by: None,
            sort_dir: None,
            from_record: None,
            record_count: None,
        };
        self.make_betting_request("listCurrentOrders", request)
            .await
    }

    async fn list_cleared_orders(
        &self,
        bet_ids: Option<Vec<String>>,
    ) -> Result<ListClearedOrdersResponse> {
        let statuses = ["LAPSED", "SETTLED", "CANCELLED", "VOIDED"];
        let mut all_orders = Vec::with_capacity(100); // Pre-allocate with reasonable capacity
        let mut more_available = false;

        for status in statuses {
            let request = ListClearedOrdersRequest {
                bet_status: Some(status.to_string()),
                event_type_ids: None,
                event_ids: None,
                market_ids: None,
                runner_ids: None,
                bet_ids: bet_ids.as_ref().cloned(),
                customer_order_refs: None,
                customer_strategy_refs: None,
                side: None,
                settled_date_range: None,
                group_by: None,
                include_item_description: None,
                locale: None,
                from_record: None,
                record_count: None,
            };

            let response: ListClearedOrdersResponse = self
                .make_betting_request("listClearedOrders", request)
                .await?;

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
        let response: AccountFundsResponse = self
            .make_account_request("getAccountFunds", request)
            .await?;

        Ok(GetAccountFundsResponse {
            available_to_bet_balance: response.available_to_bet_balance,
            exposure: response.exposure,
            retained_commission: response.retained_commission,
            exposure_limit: response.exposure_limit,
            discount_rate: Some(response.discount_rate),
            points_balance: response.points_balance as i32,
            wallet: Some(response.wallet),
        })
    }

    pub async fn get_order_status(
        &self,
        bet_ids: Vec<String>,
    ) -> Result<HashMap<String, OrderStatusResponse>> {
        let mut results = HashMap::with_capacity(bet_ids.len());
        let mut remaining_bet_ids = bet_ids;

        // First check current orders
        let current_orders = self
            .list_current_orders(Some(remaining_bet_ids.clone()), None)
            .await?;

        for order in current_orders.current_orders {
            let bet_id = order.bet_id.clone();
            results.insert(
                bet_id.clone(),
                OrderStatusResponse {
                    bet_id,
                    market_id: order.market_id,
                    selection_id: order.selection_id,
                    side: order.side,
                    order_status: format!("{:?}", order.status),
                    placed_date: order.placed_date,
                    matched_date: None,
                    average_price_matched: order.average_price_matched,
                    size_matched: order.size_matched,
                    size_remaining: order.size_remaining,
                    size_lapsed: order.size_lapsed,
                    size_cancelled: order.size_cancelled,
                    size_voided: order.size_voided,
                    profit: None,
                },
            );
            // Remove found bet_id from remaining list
            remaining_bet_ids.retain(|id| id != &order.bet_id);
        }

        // If there are remaining bet_ids, check cleared orders
        if !remaining_bet_ids.is_empty() {
            let cleared_orders = self.list_cleared_orders(Some(remaining_bet_ids)).await?;

            for order in cleared_orders.cleared_orders {
                if let Some(bet_id) = order.bet_id.clone() {
                results.insert(
                    bet_id.clone(),
                    OrderStatusResponse {
                        bet_id: bet_id.clone(),
                        market_id: order.market_id.unwrap_or_default(),
                        selection_id: order.selection_id.unwrap_or(0),
                        side: order.side.unwrap_or(Side::Back),
                        order_status: "SETTLED".to_string(),
                        placed_date: order.placed_date,
                        matched_date: order.settled_date,
                        average_price_matched: order.price_matched,
                        size_matched: order.size_settled,
                        size_remaining: None,
                        size_lapsed: None,
                        size_cancelled: order.size_cancelled,
                        size_voided: None,
                        profit: order.profit,
                    },
                );
                }
            }
        }

        Ok(results)
    }

    pub async fn subscribe_to_orders(&mut self) -> Result<()> {
        info!("Subscribing to orders stream");
        let streamer = self
            .streamer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Streamer not initialized. Please login first"))?;
        let order_sub_msg = BetfairStreamer::create_order_subscription_message();
        streamer.send_message(order_sub_msg).await?;
        Ok(())
    }
}
