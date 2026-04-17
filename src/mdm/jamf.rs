use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

struct TokenCache {
    token: String,
    expires_at: DateTime<Utc>,
}

pub struct JamfClient {
    client: Client,
    base_url: String,
    client_id: String,
    client_secret: String,
    token_cache: Arc<RwLock<Option<TokenCache>>>,
}

impl JamfClient {
    pub fn new(base_url: String, client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            client_id,
            client_secret,
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_access_token(&self) -> Result<String, String> {
        let now = Utc::now();
        
        {
            let cache_read = self.token_cache.read().await;
            if let Some(cache) = cache_read.as_ref() {
                // Return cached token if it has more than 60 seconds of validity left
                if cache.expires_at > now + Duration::seconds(60) {
                    return Ok(cache.token.clone());
                }
            }
        }

        let url = format!("{}/api/oauth/token", self.base_url);
        
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("grant_type", "client_credentials"),
        ];

        let res = self.client.post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        
        let token = data["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Missing access_token in response".to_string())?;
            
        let expires_in_seconds = data["expires_in"].as_i64().unwrap_or(1799); // typical Jamf is 30 mins
        
        let mut cache_write = self.token_cache.write().await;
        *cache_write = Some(TokenCache {
            token: token.clone(),
            expires_at: now + Duration::seconds(expires_in_seconds),
        });

        Ok(token)
    }

    pub async fn check_device_compliance(&self, jamf_id: &str) -> Result<bool, String> {
        let token = self.get_access_token().await?;
        let url = format!("{}/api/v1/computers-inventory/{}", self.base_url, jamf_id);

        let res = self.client.get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        
        // This relies on Jamf Pro's extension attributes or local status.
        // For scaffold, we assume missing or not deployed means non-compliant
        let is_managed = data["general"]["remoteManagement"]["managed"].as_bool().unwrap_or(false);
        Ok(is_managed)
    }
}
