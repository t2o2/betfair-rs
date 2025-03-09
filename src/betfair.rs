use reqwest::header::HeaderMap;
use anyhow::Result;
use reqwest::Client;
use crate::config::Config;
use std::fs;
#[allow(dead_code)]
pub struct BetfairClient {
    client: Client,
    config: Config,
    session_token: Option<String>,
}

const LOGIN_URL: &str = "https://identitysso-cert.betfair.com/api/certlogin";

impl BetfairClient {
    #[allow(dead_code)]
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
            session_token: None,
        }
    }

    #[allow(dead_code)]
    pub async fn login(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Open the encrypted PEM file
        let pem_contents = fs::read(&self.config.betfair.pfx_path)?;
    
        let identity = reqwest::Identity::from_pkcs12_der(&pem_contents, &self.config.betfair.pfx_password)?;
    
        // Create default headers
        let mut headers = HeaderMap::new();
        headers.insert("X-Application", self.config.betfair.api_key.parse()?);
        headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);
    
        // Build the client with the identity
        let client = Client::builder().identity(identity).build()?;
    
        let form = [
            ("username", self.config.betfair.username.as_str()),
            ("password", self.config.betfair.password.as_str()),
        ];
    
        let response = client
            .post(LOGIN_URL) 
            .headers(headers)
            .header(
                "X-Application",
                format!("schroedinger_{}", rand::random::<u128>()),
            )
            .form(&form)
            .send();
    
        println!("Login response: {:?}", response);
        Ok(())
    }
} 