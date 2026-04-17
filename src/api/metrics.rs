use rocket::{Route, http::Status, request::{FromRequest, Outcome, Request}};
use crate::CONFIG;

// A local guard for the Bearer token
pub struct MetricsAuth;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MetricsAuth {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if !CONFIG.metrics_enabled() {
            return Outcome::Error((Status::NotFound, ()));
        }

        let token = CONFIG.metrics_token();
        if token.is_empty() {
            // Unauthenticated metrics endpoint if unset
            return Outcome::Success(MetricsAuth);
        }

        if let Some(auth_header) = request.headers().get_one("Authorization") {
            if auth_header.starts_with("Bearer ") {
                let provided_token = &auth_header[7..];
                // In production, use constant-time comparison
                if provided_token == token {
                    return Outcome::Success(MetricsAuth);
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}

pub fn routes() -> Vec<Route> {
    routes![get_metrics]
}

#[get("/metrics")]
pub async fn get_metrics(_auth: MetricsAuth) -> Result<String, Status> {
    let registry = crate::metrics::REGISTRY.read().unwrap();
    let mut buffer = String::new();
    
    if let Err(e) = prometheus_client::encoding::text::encode(&mut buffer, &*registry) {
        error!("Failed to encode metrics: {}", e);
        return Err(Status::InternalServerError);
    }
    
    Ok(buffer)
}
