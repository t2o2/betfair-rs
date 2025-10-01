use crate::config::Config;
use crate::dto::rpc::{InteractiveLoginResponse, LoginResponse};
use crate::dto::*;
use crate::rate_limiter::BetfairRateLimiter;
use crate::retry::RetryPolicy;
use anyhow::Result;
use reqwest::{header::HeaderMap, Client};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::debug;

const LOGIN_URL: &str = "https://identitysso-cert.betfair.com/api/certlogin";
const INTERACTIVE_LOGIN_URL: &str = "https://identitysso.betfair.com/api/login";
const BETTING_URL: &str = "https://api.betfair.com/exchange/betting/json-rpc/v1";
const ACCOUNT_URL: &str = "https://api.betfair.com/exchange/account/json-rpc/v1";

fn load_pem_identity(pem_path: &str) -> Result<reqwest::Identity> {
    let pem_contents = std::fs::read(pem_path)
        .map_err(|e| anyhow::anyhow!("Failed to read PEM file {pem_path}: {e}"))?;

    reqwest::Identity::from_pem(&pem_contents)
        .map_err(|e| anyhow::anyhow!("Failed to parse PEM identity: {e}"))
}

/// REST API client for all Betfair operations
pub struct RestClient {
    client: Client,
    config: Arc<Config>,
    session_token: Option<String>,
    retry_policy: RetryPolicy,
    rate_limiter: BetfairRateLimiter,
}

impl RestClient {
    /// Create a new API client
    pub fn new(config: Config) -> Self {
        let client = Client::new();

        Self {
            client,
            config: Arc::new(config),
            session_token: None,
            retry_policy: RetryPolicy::default(),
            rate_limiter: BetfairRateLimiter::new(),
        }
    }

    /// Login to Betfair using certificate authentication and obtain session token
    pub async fn login(&mut self) -> Result<LoginResponse> {
        crate::ensure_crypto_provider();

        let api_key = self.config.betfair.api_key.clone();
        let username = self.config.betfair.username.clone();
        let password = self.config.betfair.password.clone();
        let pem_path = self.config.betfair.pem_path.clone();

        let response = self
            .retry_policy
            .retry(|| {
                let api_key = api_key.clone();
                let username = username.clone();
                let password = password.clone();
                let pem_path = pem_path.clone();
                async move {
                    let identity = load_pem_identity(&pem_path)?;

                    let mut headers = HeaderMap::new();
                    headers.insert("X-Application", api_key.parse()?);
                    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);

                    let client = Client::builder()
                        .use_rustls_tls()
                        .identity(identity)
                        .build()?;
                    let form = [
                        ("username", username.as_str()),
                        ("password", password.as_str()),
                    ];

                    let http_response = client
                        .post(LOGIN_URL)
                        .headers(headers)
                        .header("X-Application", format!("app_{}", rand::random::<u128>()))
                        .form(&form)
                        .send()
                        .await?;

                    let response_text = http_response.text().await?;
                    tracing::info!("Login response: {}", response_text);

                    let response: LoginResponse = serde_json::from_str(&response_text)?;

                    Ok(response)
                }
            })
            .await?;

        if response.login_status == "SUCCESS" {
            self.session_token = Some(response.session_token.clone());
        }

        Ok(response)
    }

    /// Login to Betfair using interactive (username/password) authentication
    pub async fn login_interactive(
        &mut self,
        username: String,
        password: String,
    ) -> Result<InteractiveLoginResponse> {
        let api_key = self.config.betfair.api_key.clone();
        let client = self.client.clone();
        let retry_policy = self.retry_policy.clone();

        let response = retry_policy
            .retry(|| {
                let api_key = api_key.clone();
                let username = username.clone();
                let password = password.clone();
                let client = client.clone();

                async move {
                    let mut headers = HeaderMap::new();
                    headers.insert("X-Application", api_key.parse()?);
                    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);
                    headers.insert("Accept", "application/json".parse()?);

                    let form = [
                        ("username", username.as_str()),
                        ("password", password.as_str()),
                    ];

                    let response = client
                        .post(INTERACTIVE_LOGIN_URL)
                        .headers(headers)
                        .form(&form)
                        .send()
                        .await?;

                    let status = response.status();
                    let body = response.text().await?;

                    debug!("Interactive login response status: {}", status);
                    debug!("Interactive login response body: {}", body);

                    if !status.is_success() {
                        return Err(anyhow::anyhow!(
                            "Interactive login failed with status {}: {}",
                            status,
                            body.trim()
                        ));
                    }

                    let login_response: InteractiveLoginResponse = serde_json::from_str(&body)?;

                    Ok(login_response)
                }
            })
            .await?;

        // Extract the session token from various possible fields
        let session_token = self.extract_session_token(&response)?;

        // Check login status
        let status = self.get_login_status(&response);
        if !status.is_empty() && status.to_uppercase() != "SUCCESS" {
            let error_msg = self.get_error_message(&response, &status);
            return Err(anyhow::anyhow!(
                "Interactive login {status}: {error_msg}"
            ));
        }

        if session_token.is_empty() {
            return Err(anyhow::anyhow!(
                "Interactive login response did not include a session token"
            ));
        }

        self.session_token = Some(session_token);
        Ok(response)
    }

    fn extract_session_token(&self, response: &InteractiveLoginResponse) -> Result<String> {
        // Try sessionToken first, then token
        if let Some(token) = &response.session_token {
            if !token.is_empty() {
                return Ok(token.clone());
            }
        }

        if let Some(token) = &response.token {
            if !token.is_empty() {
                return Ok(token.clone());
            }
        }

        Ok(String::new())
    }

    fn get_login_status(&self, response: &InteractiveLoginResponse) -> String {
        // Return the first non-empty status field
        response
            .login_status
            .as_ref()
            .or(response.status.as_ref())
            .or(response.status_code.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    fn get_error_message(&self, response: &InteractiveLoginResponse, status: &str) -> String {
        // Return the first available error message
        response
            .error
            .as_ref()
            .or(response.error_details.as_ref())
            .cloned()
            .unwrap_or_else(|| format!("Login failed with status: {status}"))
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

                    let response = client
                        .post(&url_str)
                        .headers(headers)
                        .json(&jsonrpc_request)
                        .send()
                        .await?;

                    let status = response.status();
                    debug!("API response status: {}", status);

                    let response_text = response.text().await?;
                    debug!("API response: {}", response_text);

                    if !status.is_success() {
                        return Err(anyhow::anyhow!(
                            "API request failed with status {status}: {response_text}"
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
        let price_decimal = Decimal::from_f64(price).unwrap_or(Decimal::ZERO);
        let size_decimal = Decimal::from_f64(size).unwrap_or(Decimal::ZERO);

        let request = PlaceOrdersRequest {
            market_id,
            instructions: vec![PlaceInstruction {
                order_type: OrderType::Limit,
                selection_id,
                handicap: Some(Decimal::ZERO),
                side,
                limit_order: Some(LimitOrder {
                    size: size_decimal,
                    price: price_decimal,
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
            },
        )
        .await
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
    /// ```no_run
    /// # use betfair_rs::{RestClient, Config};
    /// # use betfair_rs::dto::MarketFilter;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = Config::new()?;
    /// # let client = RestClient::new(config);
    /// // Get all events for Soccer
    /// let filter = MarketFilter {
    ///     event_type_ids: Some(vec!["1".to_string()]),
    ///     ..Default::default()
    /// };
    /// let events = client.list_events(Some(filter)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_events(&self, filter: Option<MarketFilter>) -> Result<Vec<EventResult>> {
        self.rate_limiter.acquire_for_navigation().await?;
        self.make_json_rpc_request(
            BETTING_URL,
            "SportsAPING/v1.0/listEvents",
            ListEventsRequest {
                filter: filter.unwrap_or_default(),
                locale: Some("en".to_string()),
            },
        )
        .await
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
    /// ```no_run
    /// # use betfair_rs::{RestClient, Config};
    /// # use betfair_rs::dto::MarketFilter;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = Config::new()?;
    /// # let client = RestClient::new(config);
    /// // Get all competitions for Tennis in USA
    /// let filter = MarketFilter {
    ///     event_type_ids: Some(vec!["2".to_string()]),
    ///     market_countries: Some(vec!["US".to_string()]),
    ///     ..Default::default()
    /// };
    /// let competitions = client.list_competitions(Some(filter)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_competitions(
        &self,
        filter: Option<MarketFilter>,
    ) -> Result<Vec<CompetitionResult>> {
        self.rate_limiter.acquire_for_navigation().await?;
        self.make_json_rpc_request(
            BETTING_URL,
            "SportsAPING/v1.0/listCompetitions",
            ListCompetitionsRequest {
                filter: filter.unwrap_or_default(),
                locale: Some("en".to_string()),
            },
        )
        .await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BetfairConfig;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn create_test_config() -> Config {
        Config {
            betfair: BetfairConfig {
                username: "test_user".to_string(),
                password: "test_pass".to_string(),
                api_key: "test_key".to_string(),
                pem_path: "/tmp/test.pem".to_string(),
            },
        }
    }

    #[test]
    fn test_api_client_creation() {
        let config = create_test_config();
        let client = RestClient::new(config);

        assert!(client.session_token.is_none());
        assert!(client.get_session_token().is_none());
    }

    #[test]
    fn test_set_and_get_session_token() {
        let config = create_test_config();
        let mut client = RestClient::new(config);

        let token = "test_session_token".to_string();
        client.set_session_token(token.clone());

        assert_eq!(client.get_session_token(), Some(token));
    }

    #[test]
    fn test_client_has_session_token_field() {
        let config = create_test_config();
        let client = RestClient::new(config);

        assert!(client.session_token.is_none());
    }

    #[test]
    fn test_client_has_api_key_in_config() {
        let config = create_test_config();
        let client = RestClient::new(config);

        assert_eq!(client.config.betfair.api_key, "test_key");
    }

    #[test]
    fn test_extract_session_token_from_session_token_field() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: Some("test_session_token".to_string()),
            token: None,
            login_status: Some("SUCCESS".to_string()),
            status: None,
            status_code: None,
            error: None,
            error_details: None,
        };

        let token = client.extract_session_token(&response).unwrap();
        assert_eq!(token, "test_session_token");
    }

    #[test]
    fn test_extract_session_token_from_token_field() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: None,
            token: Some("test_token".to_string()),
            login_status: Some("SUCCESS".to_string()),
            status: None,
            status_code: None,
            error: None,
            error_details: None,
        };

        let token = client.extract_session_token(&response).unwrap();
        assert_eq!(token, "test_token");
    }

    #[test]
    fn test_extract_session_token_prefers_session_token_over_token() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: Some("session_token_value".to_string()),
            token: Some("token_value".to_string()),
            login_status: Some("SUCCESS".to_string()),
            status: None,
            status_code: None,
            error: None,
            error_details: None,
        };

        let token = client.extract_session_token(&response).unwrap();
        assert_eq!(token, "session_token_value");
    }

    #[test]
    fn test_extract_session_token_empty_when_no_tokens() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: None,
            token: None,
            login_status: Some("SUCCESS".to_string()),
            status: None,
            status_code: None,
            error: None,
            error_details: None,
        };

        let token = client.extract_session_token(&response).unwrap();
        assert_eq!(token, "");
    }

    #[test]
    fn test_get_login_status_from_login_status_field() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: Some("token".to_string()),
            token: None,
            login_status: Some("SUCCESS".to_string()),
            status: Some("OTHER".to_string()),
            status_code: Some("CODE".to_string()),
            error: None,
            error_details: None,
        };

        let status = client.get_login_status(&response);
        assert_eq!(status, "SUCCESS");
    }

    #[test]
    fn test_get_login_status_falls_back_to_status() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: Some("token".to_string()),
            token: None,
            login_status: None,
            status: Some("FAILED".to_string()),
            status_code: Some("CODE".to_string()),
            error: None,
            error_details: None,
        };

        let status = client.get_login_status(&response);
        assert_eq!(status, "FAILED");
    }

    #[test]
    fn test_get_login_status_falls_back_to_status_code() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: Some("token".to_string()),
            token: None,
            login_status: None,
            status: None,
            status_code: Some("ERROR_CODE".to_string()),
            error: None,
            error_details: None,
        };

        let status = client.get_login_status(&response);
        assert_eq!(status, "ERROR_CODE");
    }

    #[test]
    fn test_get_error_message_from_error_field() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: Some("token".to_string()),
            token: None,
            login_status: Some("FAILED".to_string()),
            status: None,
            status_code: None,
            error: Some("Authentication failed".to_string()),
            error_details: Some("Detailed error".to_string()),
        };

        let error_msg = client.get_error_message(&response, "FAILED");
        assert_eq!(error_msg, "Authentication failed");
    }

    #[test]
    fn test_get_error_message_falls_back_to_error_details() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: Some("token".to_string()),
            token: None,
            login_status: Some("FAILED".to_string()),
            status: None,
            status_code: None,
            error: None,
            error_details: Some("Detailed error message".to_string()),
        };

        let error_msg = client.get_error_message(&response, "FAILED");
        assert_eq!(error_msg, "Detailed error message");
    }

    #[test]
    fn test_get_error_message_default_when_no_error_fields() {
        let config = create_test_config();
        let client = RestClient::new(config);

        let response = InteractiveLoginResponse {
            session_token: Some("token".to_string()),
            token: None,
            login_status: Some("FAILED".to_string()),
            status: None,
            status_code: None,
            error: None,
            error_details: None,
        };

        let error_msg = client.get_error_message(&response, "FAILED");
        assert_eq!(error_msg, "Login failed with status: FAILED");
    }

    #[test]
    fn test_market_filter_builder() {
        let filter = MarketFilter {
            event_type_ids: Some(vec!["1".to_string()]),
            competition_ids: Some(vec!["10".to_string()]),
            market_ids: Some(vec!["1.123456".to_string()]),
            in_play_only: Some(true),
            market_betting_types: Some(vec!["ODDS".to_string()]),
            market_countries: Some(vec!["GB".to_string()]),
            market_type_codes: Some(vec!["MATCH_ODDS".to_string()]),
            ..Default::default()
        };

        assert_eq!(filter.event_type_ids.unwrap()[0], "1");
        assert_eq!(filter.competition_ids.unwrap()[0], "10");
        assert_eq!(filter.market_ids.unwrap()[0], "1.123456");
        assert!(filter.in_play_only.unwrap());
    }

    #[test]
    fn test_place_orders_request_builder() {
        let request = PlaceOrdersRequest {
            market_id: "1.123456".to_string(),
            instructions: vec![PlaceInstruction {
                order_type: OrderType::Limit,
                selection_id: 12345,
                handicap: Some(Decimal::ZERO),
                side: Side::Back,
                limit_order: Some(LimitOrder {
                    size: dec!(10.0),
                    price: dec!(2.0),
                    persistence_type: PersistenceType::Lapse,
                    time_in_force: None,
                    min_fill_size: None,
                    bet_target_type: None,
                    bet_target_size: None,
                }),
                limit_on_close_order: None,
                market_on_close_order: None,
                customer_order_ref: Some("test_ref".to_string()),
            }],
            customer_ref: None,
            market_version: None,
            customer_strategy_ref: None,
            async_: None,
        };

        assert_eq!(request.market_id, "1.123456");
        assert_eq!(request.instructions[0].selection_id, 12345);
        assert_eq!(request.instructions[0].side, Side::Back);
        assert_eq!(
            request.instructions[0].limit_order.as_ref().unwrap().size,
            dec!(10.0)
        );
        assert_eq!(
            request.instructions[0].limit_order.as_ref().unwrap().price,
            dec!(2.0)
        );
    }

    #[test]
    fn test_cancel_orders_request_builder() {
        let request = CancelOrdersRequest {
            market_id: "1.123456".to_string(),
            instructions: vec![CancelInstruction {
                bet_id: "12345".to_string(),
                size_reduction: Some(dec!(5.0)),
            }],
            customer_ref: None,
        };

        assert_eq!(request.market_id, "1.123456");
        assert_eq!(request.instructions[0].bet_id, "12345");
        assert_eq!(request.instructions[0].size_reduction.unwrap(), dec!(5.0));
    }

    #[test]
    fn test_list_market_catalogue_request() {
        let request = ListMarketCatalogueRequest {
            filter: MarketFilter {
                event_type_ids: Some(vec!["1".to_string()]),
                ..Default::default()
            },
            market_projection: Some(vec![MarketProjection::Competition, MarketProjection::Event]),
            sort: Some(MarketSort::FirstToStart),
            max_results: Some(10i32),
            locale: Some("en".to_string()),
        };

        assert!(request.filter.event_type_ids.is_some());
        assert_eq!(request.max_results, Some(10));
        assert_eq!(request.locale, Some("en".to_string()));
    }

    #[test]
    fn test_list_market_book_request() {
        let request = ListMarketBookRequest {
            market_ids: vec!["1.123456".to_string()],
            price_projection: Some(PriceProjectionDto {
                price_data: Some(vec![PriceData::ExBestOffers]),
                ex_best_offers_overrides: None,
                virtualise: Some(false),
                rollover_stakes: Some(false),
            }),
            order_projection: None,
            match_projection: None,
            include_overall_position: Some(false),
            partition_matched_by_strategy_ref: Some(false),
            customer_strategy_refs: None,
            currency_code: Some("GBP".to_string()),
            locale: Some("en".to_string()),
            matched_since: None,
            bet_ids: None,
        };

        assert_eq!(request.market_ids[0], "1.123456");
        assert!(request.price_projection.is_some());
        assert_eq!(request.currency_code, Some("GBP".to_string()));
    }

    #[test]
    fn test_list_current_orders_request() {
        let request = ListCurrentOrdersRequest {
            bet_ids: Some(vec!["123".to_string(), "456".to_string()]),
            market_ids: Some(vec!["1.123456".to_string()]),
            order_projection: Some("ALL".to_string()),
            customer_order_refs: None,
            customer_strategy_refs: None,
            date_range: None,
            order_by: Some("BY_BET".to_string()),
            sort_dir: Some("EARLIEST_TO_LATEST".to_string()),
            from_record: Some(0),
            record_count: Some(100),
        };

        assert_eq!(request.bet_ids.unwrap().len(), 2);
        assert_eq!(request.order_by, Some("BY_BET".to_string()));
        assert_eq!(request.record_count, Some(100));
    }

    #[test]
    fn test_get_account_funds_request() {
        let request = GetAccountFundsRequest {
            wallet: Some(Wallet::Uk),
        };

        assert!(request.wallet.is_some());
    }
}
