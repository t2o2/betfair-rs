use reqwest::header::HeaderMap;
use anyhow::Result;
use reqwest::Client;
use crate::config::Config;
use std::fs;
use crate::models::LoginResponse;
use crate::streamer::BetfairStreamer;
const LOGIN_URL: &str = "https://identitysso-cert.betfair.com/api/certlogin";

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
    
        match response.sessionToken {
            Some(token) => {
                self.session_token = Some(token);
                self.streamer = Some(BetfairStreamer::new(self.config.betfair.api_key.clone(), self.session_token.clone().unwrap()));
                Ok(())
            }
            None => Err(anyhow::anyhow!("loginStatus: {}", response.loginStatus)),
        }
    }

    pub async fn get_session_token(&self) -> Option<String> {
        self.session_token.clone()
    }

    pub async fn subscribe_to_market(&mut self, market_id: String) -> Result<()> {
        let streamer = self.streamer.as_mut().unwrap();
        streamer.subscribe(market_id).await?;
        Ok(())
    }

    pub async fn start_listening(&mut self) -> Result<()> {
        let streamer = self.streamer.as_mut().unwrap();
        streamer.connect_betfair_tls_stream().await?;
        streamer.start_listening().await?;
        Ok(())
    }
}