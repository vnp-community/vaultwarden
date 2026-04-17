use crate::error::Error;
use reqwest::{Client, StatusCode};

pub struct ItsmClient {
    client: Client,
}

impl ItsmClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn validate_ticket(&self, ticket_number: &str) -> Result<bool, Error> {
        if !crate::CONFIG.itsm_enabled() || !crate::CONFIG.itsm_ticket_validation() {
            return Ok(true);
        }

        let itsm_type = crate::CONFIG.itsm_type();
        match itsm_type.as_str() {
            "servicenow" => self.validate_servicenow_ticket(ticket_number).await,
            "mock" => Ok(ticket_number.starts_with("INC")), // Quick unit-testing validation
            _ => Ok(true),
        }
    }

    pub async fn validate_servicenow_ticket(&self, ticket_number: &str) -> Result<bool, Error> {
        let instance = crate::CONFIG.itsm_servicenow_instance();
        let user = crate::CONFIG.itsm_servicenow_user();
        let password = crate::CONFIG.itsm_servicenow_password();

        if instance.is_empty() || user.is_empty() {
            return Ok(true); 
        }

        let url = format!("{}/api/now/table/incident?sysparm_query=number={}&sysparm_limit=1", instance, ticket_number);
        let res = self.client.get(&url)
            .basic_auth(user, Some(password))
            .send()
            .await?;

        if res.status() == StatusCode::OK {
            let json: serde_json::Value = res.json().await?;
            if let Some(results) = json.get("result").and_then(|v| v.as_array()) {
                if !results.is_empty() {
                    let incident = &results[0];
                    if let Some(state) = incident.get("state").and_then(|s| s.as_str()) {
                        // 6=Resolved, 7=Closed, 8=Canceled
                        if state != "6" && state != "7" && state != "8" {
                            return Ok(true);
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }
}
