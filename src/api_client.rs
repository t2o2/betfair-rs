use crate::config::Config;
use crate::dto::*;
use crate::dto::rpc::LoginResponse;
use crate::rate_limiter::BetfairRateLimiter;
use crate::retry::RetryPolicy;
use anyhow::Result;
use reqwest::{header::HeaderMap, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::debug;

const LOGIN_URL: &str = "https://identitysso-cert.betfair.com/api/certlogin";
const BETTING_URL: &str = "https://api.betfair.com/exchange/betting/json-rpc/v1";
const ACCOUNT_URL: &str = "https://api.betfair.com/exchange/account/json-rpc/v1";

/// Unified API client for all Betfair operations
pub struct BetfairApiClient {
    client: Client,
    config: Arc<Config>,
    session_token: Option<String>,
    retry_policy: RetryPolicy,
    rate_limiter: BetfairRateLimiter,
}

impl BetfairApiClient {
    /// Create a new API client
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config: Arc::new(config),
            session_token: None,
            retry_policy: RetryPolicy::default(),
            rate_limiter: BetfairRateLimiter::new(),
        }
    }

    /// Login to Betfair and obtain session token
    pub async fn login(&mut self) -> Result<LoginResponse> {
        let api_key = self.config.betfair.api_key.clone();
        let username = self.config.betfair.username.clone();
        let password = self.config.betfair.password.clone();
        let pfx_path = self.config.betfair.pfx_path.clone();
        let pfx_password = self.config.betfair.pfx_password.clone();

        let response = self
            .retry_policy
            .retry(|| {
                let api_key = api_key.clone();
                let username = username.clone();
                let password = password.clone();
                let pfx_path = pfx_path.clone();
                let pfx_password = pfx_password.clone();

                async move {
                    // Read and create identity inside the retry closure
                    let pem_contents = std::fs::read(&pfx_path)?;
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
                        .header("X-Application", format!("app_{}", rand::random::<u128>()))
                        .form(&form)
                        .send()?
                        .json()?;

                    Ok(response)
                }
            })
            .await?;

        if response.login_status == "SUCCESS" {
            self.session_token = Some(response.session_token.clone());
        }

        Ok(response)
    }

    /// Get current session token
    pub fn get_session_token(&self) -> Option<String> {
        self.session_token.clone()
    }

    /// Set session token (useful for restoring sessions)
    pub fn set_session_token(&mut self, token: String) {
        self.session_token = Some(token);
    }

    /// Generic method to make JSON-RPC API requests
    async fn make_json_rpc_request<T, U>(&self, url: &str, method: &str, params: T) -> Result<U>
    where
        T: Serialize + Clone,
        U: DeserializeOwned,
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

                    debug!("API request: {}", serde_json::to_string(&jsonrpc_request)?);

                    let mut response = client
                        .post(&url_str)
                        .headers(headers)
                        .json(&jsonrpc_request)
                        .send()?;

                    let status = response.status();
                    debug!("API response status: {}", status);

                    let response_text = response.text()?;
                    debug!("API response: {}", response_text);

                    if !status.is_success() {
                        return Err(anyhow::anyhow!(
                            "API request failed with status {}: {}",
                            status,
                            response_text
                        ));
                    }

                    let json_response: JsonRpcResponse<U> = serde_json::from_str(&response_text)?;
                    json_response.result.ok_or_else(|| {
                        anyhow::anyhow!("No result in response: {:?}", json_response.error)
                    })
                }
            })
            .await
    }

    // ========================================================================
    // Market Operations
    // ========================================================================

    /// List market catalogue
    pub async fn list_market_catalogue(
        &self,
        request: ListMarketCatalogueRequest,
    ) -> Result<Vec<MarketCatalogue>> {
        self.rate_limiter.acquire_for_navigation().await?;
        self.make_json_rpc_request(BETTING_URL, "SportsAPING/v1.0/listMarketCatalogue", request)
            .await
    }

    /// List market book
    pub async fn list_market_book(
        &self,
        request: ListMarketBookRequest,
    ) -> Result<Vec<MarketBook>> {
        self.rate_limiter.acquire_for_navigation().await?;
        self.make_json_rpc_request(BETTING_URL, "SportsAPING/v1.0/listMarketBook", request)
            .await
    }

    // ========================================================================
    // Order Operations
    // ========================================================================

    /// Place orders
    pub async fn place_orders(&self, request: PlaceOrdersRequest) -> Result<PlaceOrdersResponse> {
        self.rate_limiter.acquire_for_transaction().await?;
        self.make_json_rpc_request(BETTING_URL, "SportsAPING/v1.0/placeOrders", request)
            .await
    }

    /// Cancel orders
    pub async fn cancel_orders(
        &self,
        request: CancelOrdersRequest,
    ) -> Result<CancelOrdersResponse> {
        self.rate_limiter.acquire_for_transaction().await?;
        self.make_json_rpc_request(BETTING_URL, "SportsAPING/v1.0/cancelOrders", request)
            .await
    }

    /// List current orders
    pub async fn list_current_orders(
        &self,
        request: ListCurrentOrdersRequest,
    ) -> Result<ListCurrentOrdersResponse> {
        self.rate_limiter.acquire_for_data().await?;
        self.make_json_rpc_request(BETTING_URL, "SportsAPING/v1.0/listCurrentOrders", request)
            .await
    }

    /// List cleared orders
    pub async fn list_cleared_orders(
        &self,
        request: ListClearedOrdersRequest,
    ) -> Result<ListClearedOrdersResponse> {
        self.rate_limiter.acquire_for_data().await?;
        self.make_json_rpc_request(BETTING_URL, "SportsAPING/v1.0/listClearedOrders", request)
            .await
    }

    // ========================================================================
    // Account Operations
    // ========================================================================

    /// Get account funds
    pub async fn get_account_funds(
        &self,
        request: GetAccountFundsRequest,
    ) -> Result<GetAccountFundsResponse> {
        self.rate_limiter.acquire_for_data().await?;
        self.make_json_rpc_request(ACCOUNT_URL, "AccountAPING/v1.0/getAccountFunds", request)
            .await
    }

    /// Get account details
    pub async fn get_account_details(&self) -> Result<GetAccountDetailsResponse> {
        self.rate_limiter.acquire_for_data().await?;
        self.make_json_rpc_request(
            ACCOUNT_URL,
            "AccountAPING/v1.0/getAccountDetails",
            GetAccountDetailsRequest {},
        )
        .await
    }

    /// Transfer funds between wallets
    pub async fn transfer_funds(
        &self,
        request: TransferFundsRequest,
    ) -> Result<TransferFundsResponse> {
        self.rate_limiter.acquire_for_transaction().await?;
        self.make_json_rpc_request(ACCOUNT_URL, "AccountAPING/v1.0/transferFunds", request)
            .await
    }

    // ========================================================================
    // Helper Methods for Common Operations
    // ========================================================================

    /// Place a simple back or lay order
    pub async fn place_simple_order(
        &self,
        market_id: String,
        selection_id: i64,
        side: Side,
        price: f64,
        size: f64,
    ) -> Result<PlaceOrdersResponse> {
        let request = PlaceOrdersRequest {
            market_id,
            instructions: vec![PlaceInstruction {
                order_type: OrderType::Limit,
                selection_id,
                handicap: Some(0.0),
                side,
                limit_order: Some(LimitOrder {
                    size,
                    price,
                    persistence_type: PersistenceType::Lapse,
                    time_in_force: None,
                    min_fill_size: None,
                    bet_target_type: None,
                    bet_target_size: None,
                }),
                limit_on_close_order: None,
                market_on_close_order: None,
                customer_order_ref: None,
            }],
            customer_ref: None,
            market_version: None,
            customer_strategy_ref: None,
            async_: None,
        };

        self.place_orders(request).await
    }

    /// Cancel a bet by ID
    pub async fn cancel_bet(
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

        self.cancel_orders(request).await
    }

    /// Get orders by bet IDs
    pub async fn get_orders_by_bet_ids(
        &self,
        bet_ids: Vec<String>,
    ) -> Result<ListCurrentOrdersResponse> {
        let request = ListCurrentOrdersRequest {
            bet_ids: Some(bet_ids),
            market_ids: None,
            order_projection: None,
            customer_order_refs: None,
            customer_strategy_refs: None,
            date_range: None,
            order_by: None,
            sort_dir: None,
            from_record: None,
            record_count: None,
        };

        self.list_current_orders(request).await
    }

    /// Get orders by market IDs
    pub async fn get_orders_by_market_ids(
        &self,
        market_ids: Vec<String>,
    ) -> Result<ListCurrentOrdersResponse> {
        let request = ListCurrentOrdersRequest {
            bet_ids: None,
            market_ids: Some(market_ids),
            order_projection: None,
            customer_order_refs: None,
            customer_strategy_refs: None,
            date_range: None,
            order_by: None,
            sort_dir: None,
            from_record: None,
            record_count: None,
        };

        self.list_current_orders(request).await
    }

    /// Get market prices
    pub async fn get_market_prices(&self, market_ids: Vec<String>) -> Result<Vec<MarketBook>> {
        let request = ListMarketBookRequest {
            market_ids,
            price_projection: Some(PriceProjectionDto {
                price_data: Some(vec![PriceData::ExBestOffers]),
                ex_best_offers_overrides: None,
                virtualise: None,
                rollover_stakes: None,
            }),
            order_projection: None,
            match_projection: None,
            include_overall_position: None,
            partition_matched_by_strategy_ref: None,
            customer_strategy_refs: None,
            currency_code: None,
            locale: None,
            matched_since: None,
            bet_ids: None,
        };

        self.list_market_book(request).await
    }

    /// Get odds for a specific market with full price ladder
    pub async fn get_odds(&self, market_id: String) -> Result<Vec<MarketBook>> {
        let request = ListMarketBookRequest {
            market_ids: vec![market_id],
            price_projection: Some(PriceProjectionDto {
                price_data: Some(vec![
                    PriceData::ExBestOffers,
                    PriceData::ExAllOffers,
                    PriceData::ExTraded,
                ]),
                ex_best_offers_overrides: Some(ExBestOffersOverrides {
                    best_prices_depth: Some(3),
                    rollup_model: Some("STAKE".to_string()),
                    rollup_limit: None,
                    rollup_liability_threshold: None,
                    rollup_liability_factor: None,
                }),
                virtualise: Some(true),
                rollover_stakes: Some(false),
            }),
            order_projection: None,
            match_projection: None,
            include_overall_position: None,
            partition_matched_by_strategy_ref: None,
            customer_strategy_refs: None,
            currency_code: None,
            locale: None,
            matched_since: None,
            bet_ids: None,
        };

        self.list_market_book(request).await
    }

    /// Search markets by text
    pub async fn search_markets(
        &self,
        text_query: String,
        max_results: Option<i32>,
    ) -> Result<Vec<MarketCatalogue>> {
        let request = ListMarketCatalogueRequest {
            filter: MarketFilter {
                text_query: Some(text_query),
                exchange_ids: None,
                event_type_ids: None,
                event_ids: None,
                competition_ids: None,
                market_ids: None,
                venues: None,
                bsp_only: None,
                turn_in_play_enabled: None,
                in_play_only: None,
                market_betting_types: None,
                market_countries: None,
                market_type_codes: None,
                market_start_time: None,
                with_orders: None,
            },
            market_projection: Some(vec![
                MarketProjection::Competition,
                MarketProjection::Event,
                MarketProjection::MarketStartTime,
                MarketProjection::MarketDescription,
            ]),
            sort: Some(MarketSort::FirstToStart),
            max_results,
            locale: None,
        };

        self.list_market_catalogue(request).await
    }

    /// List all available sports (event types)
    /// 
    /// # Arguments
    /// * `filter` - Optional market filter. If None, returns all sports.
    pub async fn list_sports(&self, filter: Option<MarketFilter>) -> Result<Vec<EventTypeResult>> {
        self.rate_limiter.acquire_for_navigation().await?;
        self.make_json_rpc_request(
            BETTING_URL, 
            "SportsAPING/v1.0/listEventTypes",
            ListEventTypesRequest {
                filter: filter.unwrap_or_default(),
                locale: Some("en".to_string()),
            }
        ).await
    }

    /// List events with optional filtering
    /// 
    /// # Arguments
    /// * `filter` - Optional market filter. Common filters:
    ///   - `event_type_ids`: Filter by sport IDs
    ///   - `competition_ids`: Filter by competition IDs  
    ///   - `market_countries`: Filter by country codes
    ///   - `in_play_only`: Only in-play events
    /// 
    /// # Example
    /// ```
    /// // Get all events for Soccer
    /// let filter = MarketFilter {
    ///     event_type_ids: Some(vec!["1".to_string()]),
    ///     ..Default::default()
    /// };
    /// let events = client.list_events(Some(filter)).await?;
    /// ```
    pub async fn list_events(&self, filter: Option<MarketFilter>) -> Result<Vec<EventResult>> {
        self.rate_limiter.acquire_for_navigation().await?;
        self.make_json_rpc_request(
            BETTING_URL,
            "SportsAPING/v1.0/listEvents", 
            ListEventsRequest {
                filter: filter.unwrap_or_default(),
                locale: Some("en".to_string()),
            }
        ).await
    }

    /// List competitions with optional filtering
    /// 
    /// # Arguments
    /// * `filter` - Optional market filter. Common filters:
    ///   - `event_type_ids`: Filter by sport IDs
    ///   - `market_countries`: Filter by country codes
    ///   - `competition_ids`: Specific competition IDs
    /// 
    /// # Example
    /// ```
    /// // Get all competitions for Tennis in USA
    /// let filter = MarketFilter {
    ///     event_type_ids: Some(vec!["2".to_string()]),
    ///     market_countries: Some(vec!["US".to_string()]),
    ///     ..Default::default()
    /// };
    /// let competitions = client.list_competitions(Some(filter)).await?;
    /// ```
    pub async fn list_competitions(&self, filter: Option<MarketFilter>) -> Result<Vec<CompetitionResult>> {
        self.rate_limiter.acquire_for_navigation().await?;
        self.make_json_rpc_request(
            BETTING_URL,
            "SportsAPING/v1.0/listCompetitions",
            ListCompetitionsRequest {
                filter: filter.unwrap_or_default(),
                locale: Some("en".to_string()),
            }
        ).await
    }

    /// List runners for a specific market
    /// 
    /// # Arguments
    /// * `market_id` - The market ID to get runners for
    /// 
    /// # Returns
    /// Returns the market catalogue with runner information including names, IDs, and metadata
    pub async fn list_runners(&self, market_id: &str) -> Result<Vec<MarketCatalogue>> {
        self.rate_limiter.acquire_for_navigation().await?;
        let request = ListMarketCatalogueRequest {
            filter: MarketFilter {
                market_ids: Some(vec![market_id.to_string()]),
                ..Default::default()
            },
            market_projection: Some(vec![
                MarketProjection::RunnerDescription,
                MarketProjection::RunnerMetadata,
                MarketProjection::Event,
                MarketProjection::MarketDescription,
            ]),
            sort: None,
            max_results: Some(1),
            locale: Some("en".to_string()),
        };
        
        self.list_market_catalogue(request).await
    }
}
