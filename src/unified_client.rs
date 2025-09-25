use crate::api_client::BetfairApiClient;
use crate::config::Config;
use crate::dto::rpc::{InteractiveLoginResponse, LoginResponse};
use crate::dto::*;
use crate::orderbook::Orderbook;
use crate::streaming_client::StreamingClient;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Type alias for the shared orderbook state
pub type SharedOrderbooks = Arc<RwLock<HashMap<String, HashMap<String, Orderbook>>>>;

/// Unified client combining REST API and streaming capabilities
pub struct UnifiedBetfairClient {
    api_client: BetfairApiClient,
    streaming_client: Option<StreamingClient>,
    config: Config,
}

impl UnifiedBetfairClient {
    /// Create a new unified client
    pub fn new(config: Config) -> Self {
        let api_client = BetfairApiClient::new(config.clone());
        Self {
            api_client,
            streaming_client: None,
            config,
        }
    }

    /// Login to Betfair using certificate authentication and obtain session token
    pub async fn login(&mut self) -> Result<LoginResponse> {
        // Login via API client
        let response = self.api_client.login().await?;

        // If login successful and we want streaming, initialize streaming client
        if response.login_status == "SUCCESS" {
            let streaming = StreamingClient::with_session_token(
                self.config.betfair.api_key.clone(),
                response.session_token.clone(),
            );
            self.streaming_client = Some(streaming);
        }

        Ok(response)
    }

    /// Login to Betfair using interactive authentication and obtain session token
    pub async fn login_interactive(
        &mut self,
        username: String,
        password: String,
    ) -> Result<InteractiveLoginResponse> {
        // Login via API client
        let response = self
            .api_client
            .login_interactive(username, password)
            .await?;

        // Extract session token and check if login was successful
        if let Some(session_token) = self.api_client.get_session_token() {
            let streaming = StreamingClient::with_session_token(
                self.config.betfair.api_key.clone(),
                session_token,
            );
            self.streaming_client = Some(streaming);
        }

        Ok(response)
    }

    /// Get current session token
    pub fn get_session_token(&self) -> Option<String> {
        self.api_client.get_session_token()
    }

    /// Set session token (useful for restoring sessions)
    pub fn set_session_token(&mut self, token: String) {
        self.api_client.set_session_token(token.clone());

        // Update streaming client if it exists
        if let Some(streaming) = &mut self.streaming_client {
            streaming.set_session_token(token.clone());
        } else {
            // Create streaming client with the token
            self.streaming_client = Some(StreamingClient::with_session_token(
                self.config.betfair.api_key.clone(),
                token,
            ));
        }
    }

    // ========== REST API Methods (delegated to BetfairApiClient) ==========

    /// List sports (event types)
    pub async fn list_sports(&self, filter: Option<MarketFilter>) -> Result<Vec<EventTypeResult>> {
        self.api_client.list_sports(filter).await
    }

    /// List competitions
    pub async fn list_competitions(
        &self,
        filter: Option<MarketFilter>,
    ) -> Result<Vec<CompetitionResult>> {
        self.api_client.list_competitions(filter).await
    }

    /// List events
    pub async fn list_events(&self, filter: Option<MarketFilter>) -> Result<Vec<EventResult>> {
        self.api_client.list_events(filter).await
    }

    /// List market catalogue
    pub async fn list_market_catalogue(
        &self,
        request: ListMarketCatalogueRequest,
    ) -> Result<Vec<MarketCatalogue>> {
        self.api_client.list_market_catalogue(request).await
    }

    /// List market book
    pub async fn list_market_book(
        &self,
        request: ListMarketBookRequest,
    ) -> Result<Vec<MarketBook>> {
        self.api_client.list_market_book(request).await
    }

    /// Get odds for a specific market with full price ladder
    pub async fn get_odds(&self, market_id: String) -> Result<Vec<MarketBook>> {
        self.api_client.get_odds(market_id).await
    }

    /// List runners for a specific market
    pub async fn list_runners(&self, market_id: &str) -> Result<Vec<MarketCatalogue>> {
        self.api_client.list_runners(market_id).await
    }

    /// Place orders
    pub async fn place_orders(&self, request: PlaceOrdersRequest) -> Result<PlaceOrdersResponse> {
        self.api_client.place_orders(request).await
    }

    /// Cancel orders
    pub async fn cancel_orders(
        &self,
        request: CancelOrdersRequest,
    ) -> Result<CancelOrdersResponse> {
        self.api_client.cancel_orders(request).await
    }

    /// List current orders
    pub async fn list_current_orders(
        &self,
        request: ListCurrentOrdersRequest,
    ) -> Result<ListCurrentOrdersResponse> {
        self.api_client.list_current_orders(request).await
    }

    /// List cleared orders
    pub async fn list_cleared_orders(
        &self,
        request: ListClearedOrdersRequest,
    ) -> Result<ListClearedOrdersResponse> {
        self.api_client.list_cleared_orders(request).await
    }

    /// Get account funds
    pub async fn get_account_funds(
        &self,
        request: GetAccountFundsRequest,
    ) -> Result<GetAccountFundsResponse> {
        self.api_client.get_account_funds(request).await
    }

    /// Get account details
    pub async fn get_account_details(&self) -> Result<GetAccountDetailsResponse> {
        self.api_client.get_account_details().await
    }

    // ========== Streaming Methods ==========

    /// Start the streaming client
    pub async fn start_streaming(&mut self) -> Result<()> {
        let streaming = self.streaming_client.as_mut().ok_or_else(|| {
            anyhow::anyhow!("Streaming client not initialized. Call login() first.")
        })?;

        streaming.start().await
    }

    /// Subscribe to a market for streaming updates
    pub async fn subscribe_to_market(&self, market_id: String, levels: usize) -> Result<()> {
        let streaming = self.streaming_client.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Streaming client not initialized. Call login() first.")
        })?;

        streaming.subscribe_to_market(market_id, levels).await
    }

    /// Unsubscribe from a market
    pub async fn unsubscribe_from_market(&self, market_id: String) -> Result<()> {
        let streaming = self.streaming_client.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Streaming client not initialized. Call login() first.")
        })?;

        streaming.unsubscribe_from_market(market_id).await
    }

    /// Get streaming orderbooks
    pub fn get_streaming_orderbooks(&self) -> Option<SharedOrderbooks> {
        self.streaming_client.as_ref().map(|s| s.get_orderbooks())
    }

    /// Get last update time for a market
    pub fn get_market_last_update_time(&self, market_id: &str) -> Option<Instant> {
        self.streaming_client
            .as_ref()
            .and_then(|s| s.get_last_update_time(market_id))
    }

    /// Check if streaming is connected
    pub fn is_streaming_connected(&self) -> bool {
        self.streaming_client
            .as_ref()
            .map(|s| s.is_connected())
            .unwrap_or(false)
    }

    /// Stop streaming
    pub async fn stop_streaming(&mut self) -> Result<()> {
        if let Some(streaming) = &mut self.streaming_client {
            streaming.stop().await?;
        }
        Ok(())
    }

    /// Set a custom orderbook callback that will be called immediately when new data arrives
    pub fn set_orderbook_callback<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(String, HashMap<String, Orderbook>) + Send + Sync + 'static,
    {
        let streaming = self.streaming_client.as_mut().ok_or_else(|| {
            anyhow::anyhow!("Streaming client not initialized. Call login() first.")
        })?;

        streaming.set_orderbook_callback(callback);
        Ok(())
    }

    // ========== Convenience Methods ==========

    /// Place an order and subscribe to updates for the market
    pub async fn place_order_with_updates(
        &mut self,
        request: PlaceOrdersRequest,
        levels: usize,
    ) -> Result<PlaceOrdersResponse> {
        // Place the order
        let response = self.api_client.place_orders(request.clone()).await?;

        // If successful, subscribe to market updates
        if response.status == "SUCCESS" {
            if let Err(e) = self
                .subscribe_to_market(request.market_id.clone(), levels)
                .await
            {
                // Log but don't fail the order placement
                tracing::warn!("Failed to subscribe to market updates: {}", e);
            }
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BetfairConfig;
    use crate::dto::{LimitOrder, OrderType, PersistenceType, PlaceInstruction, Side};

    fn create_test_config() -> Config {
        Config {
            betfair: BetfairConfig {
                username: "test_user".to_string(),
                password: "test_pass".to_string(),
                api_key: "test_api_key".to_string(),
                pfx_path: "/tmp/test.pfx".to_string(),
                pfx_password: "test_pfx_pass".to_string(),
                proxy_url: None,
            },
        }
    }

    #[test]
    fn test_unified_client_creation() {
        let config = create_test_config();
        let client = UnifiedBetfairClient::new(config);

        assert!(client.get_session_token().is_none());
        assert!(!client.is_streaming_connected());
    }

    #[test]
    fn test_get_and_set_session_token() {
        let config = create_test_config();
        let mut client = UnifiedBetfairClient::new(config);

        assert!(client.get_session_token().is_none());

        client.set_session_token("test_token".to_string());
        assert_eq!(client.get_session_token(), Some("test_token".to_string()));
    }

    #[test]
    fn test_get_streaming_orderbooks() {
        let config = create_test_config();
        let client = UnifiedBetfairClient::new(config);

        let ob1 = client.get_streaming_orderbooks();
        let ob2 = client.get_streaming_orderbooks();

        assert!(ob1.is_none());
        assert!(ob2.is_none());
    }

    #[test]
    fn test_get_streaming_orderbooks_empty() {
        let config = create_test_config();
        let client = UnifiedBetfairClient::new(config);

        let orderbooks = client.get_streaming_orderbooks();
        assert!(orderbooks.is_none());
    }

    #[test]
    fn test_market_last_update_time_not_available() {
        let config = create_test_config();
        let client = UnifiedBetfairClient::new(config);

        let time = client.get_market_last_update_time("1.123456");
        assert!(time.is_none());
    }

    #[test]
    fn test_streaming_not_connected_initially() {
        let config = create_test_config();
        let client = UnifiedBetfairClient::new(config);

        assert!(!client.is_streaming_connected());
    }

    #[test]
    fn test_market_filter_methods() {
        let filter = MarketFilter {
            event_type_ids: Some(vec!["1".to_string()]),
            competition_ids: Some(vec!["10".to_string()]),
            market_ids: Some(vec!["1.123456".to_string()]),
            in_play_only: Some(true),
            market_betting_types: Some(vec!["ODDS".to_string()]),
            ..Default::default()
        };

        assert!(filter.event_type_ids.is_some());
        assert!(filter.competition_ids.is_some());
        assert!(filter.market_ids.is_some());
        assert_eq!(filter.in_play_only, Some(true));
        assert_eq!(filter.market_betting_types.unwrap()[0], "ODDS");
    }

    #[test]
    fn test_place_order_request_creation() {
        let request = PlaceOrdersRequest {
            market_id: "1.123456".to_string(),
            instructions: vec![PlaceInstruction {
                order_type: OrderType::Limit,
                selection_id: 12345,
                handicap: Some(0.0),
                side: Side::Back,
                limit_order: Some(LimitOrder {
                    size: 10.0,
                    price: 2.0,
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
        assert_eq!(request.instructions.len(), 1);
        assert_eq!(request.instructions[0].selection_id, 12345);
        assert_eq!(request.instructions[0].side, Side::Back);
    }

    #[test]
    fn test_place_order_with_updates_request() {
        let request = PlaceOrdersRequest {
            market_id: "1.123456".to_string(),
            instructions: vec![
                PlaceInstruction {
                    order_type: OrderType::Limit,
                    selection_id: 12345,
                    handicap: Some(0.0),
                    side: Side::Back,
                    limit_order: Some(LimitOrder {
                        size: 10.0,
                        price: 2.0,
                        persistence_type: PersistenceType::Lapse,
                        time_in_force: None,
                        min_fill_size: None,
                        bet_target_type: None,
                        bet_target_size: None,
                    }),
                    limit_on_close_order: None,
                    market_on_close_order: None,
                    customer_order_ref: Some("order1".to_string()),
                },
                PlaceInstruction {
                    order_type: OrderType::Limit,
                    selection_id: 54321,
                    handicap: Some(0.0),
                    side: Side::Lay,
                    limit_order: Some(LimitOrder {
                        size: 20.0,
                        price: 3.0,
                        persistence_type: PersistenceType::Persist,
                        time_in_force: None,
                        min_fill_size: None,
                        bet_target_type: None,
                        bet_target_size: None,
                    }),
                    limit_on_close_order: None,
                    market_on_close_order: None,
                    customer_order_ref: Some("order2".to_string()),
                },
            ],
            customer_ref: Some("batch_ref".to_string()),
            market_version: None,
            customer_strategy_ref: None,
            async_: None,
        };

        assert_eq!(request.market_id, "1.123456");
        assert_eq!(request.instructions.len(), 2);
        assert_eq!(request.instructions[0].selection_id, 12345);
        assert_eq!(request.instructions[0].side, Side::Back);
        assert_eq!(request.instructions[1].selection_id, 54321);
        assert_eq!(request.instructions[1].side, Side::Lay);
        assert_eq!(request.customer_ref, Some("batch_ref".to_string()));
    }

    #[test]
    fn test_unified_client_streaming_orderbooks_initialization() {
        let config = create_test_config();
        let client = UnifiedBetfairClient::new(config);

        let orderbooks = client.get_streaming_orderbooks();
        assert!(orderbooks.is_none());
    }
}
