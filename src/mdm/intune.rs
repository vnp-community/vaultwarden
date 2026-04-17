use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

struct TokenCache {
    token: String,
    expires_at: DateTime<Utc>,
}

pub struct IntuneClient {
    client: Client,
    tenant_id: String,
    client_id: String,
    client_secret: String,
    token_cache: Arc<RwLock<Option<TokenCache>>>,
}

impl IntuneClient {
    pub fn new(tenant_id: String, client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::new(),
            tenant_id,
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

        let url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", self.tenant_id);
        
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("scope", "https://graph.microsoft.com/.default"),
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
            
        let expires_in_seconds = data["expires_in"].as_i64().unwrap_or(3599);
        
        let mut cache_write = self.token_cache.write().await;
        *cache_write = Some(TokenCache {
            token: token.clone(),
            expires_at: now + Duration::seconds(expires_in_seconds),
        });

        Ok(token)
    }

    pub async fn check_device_compliance(&self, device_id: &str) -> Result<bool, String> {
        let token = self.get_access_token().await?;
        // using azureADDeviceId filter or by ID directly. Assuming graph API call by azureADDeviceId:
        let url = format!("https://graph.microsoft.com/v1.0/deviceManagement/managedDevices?$filter=azureADDeviceId eq '{}'", device_id);

        let res = self.client.get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        
        // Find the device in the returned array
        if let Some(devices) = data["value"].as_array() {
            if let Some(device) = devices.first() {
                let compliance_state = device["complianceState"].as_str().unwrap_or("unknown");
                return Ok(compliance_state == "compliant");
            }
        }
        
        // If not found or empty array, return false
        Ok(false)
    }
}
