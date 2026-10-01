use crate::configuration::Configuration;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Error as RequestError, Response,
};
use std::collections::HashMap;

// https://docs.coingecko.com/

#[derive(Clone)]
pub struct CoingeckoApi {
    client: Client,
    headers: HeaderMap,
}

const API_URL: &str = "https://api.coingecko.com/api/v3";

pub type CoingeckoRates = HashMap<String, HashMap<String, f64>>;

impl CoingeckoApi {
    pub fn new(config: &Configuration) -> Self {
        Self {
            client: Client::new(),
            headers: Self::create_headers(config),
        }
    }

    fn create_headers(config: &Configuration) -> HeaderMap {
        // Set up headers with API key "x-cg-demo-api-key"
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-cg-demo-api-key",
            HeaderValue::from_str(&config.secrets.coingecko_api_key).unwrap(),
        );
        headers
    }

    // Helper method to create a request with headers
    #[allow(non_snake_case)]
    async fn GET(&self, path: &str) -> Result<Response, RequestError> {
        self.client
            .get(format!("{}{}", API_URL, path))
            .headers(self.headers.clone())
            .send()
            .await
    }

    // Ping endpoint (GET /ping)
    pub async fn _ping(&self) -> bool {
        match self.GET("/ping").await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn get_rates<B: AsRef<str>, Q: AsRef<str>>(
        &self,
        base_ids: &[B],
        quote_ids: &[Q],
    ) -> Result<CoingeckoRates, String> {
        let path = format!(
            "/simple/price?ids={}&vs_currencies={}",
            base_ids
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<_>>()
                .join(","),
            quote_ids
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<_>>()
                .join(","),
        );

        let result = self.GET(&path).await;

        match result {
            Ok(response) if response.status().is_success() => {
                let data: HashMap<String, HashMap<String, f64>> = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?;

                Ok(data)
            }
            Ok(response) => Err(format!("Non-200 status: {}", response.status())),
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    }
}
