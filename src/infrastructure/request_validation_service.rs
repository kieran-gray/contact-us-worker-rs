use std::sync::Arc;

use crate::application::{
    exceptions::AppError, services::request_validation_service::RequestValidationServiceTrait,
};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::{error, info};
use worker::{Fetch, Headers, Method, Request, RequestInit};

#[derive(Deserialize, Debug)]
struct TurnstileResponse {
    success: bool,
    #[serde(rename = "error-codes")]
    error_codes: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct CloudflareRequestValidationService {
    siteverify_url: String,
    secret_key: String,
}

impl CloudflareRequestValidationService {
    pub fn create(
        siteverify_url: &str,
        secret_key: &str,
    ) -> Arc<dyn RequestValidationServiceTrait> {
        Arc::new(Self {
            siteverify_url: siteverify_url.to_string(),
            secret_key: secret_key.to_string(),
        })
    }
}

#[async_trait(?Send)]
impl RequestValidationServiceTrait for CloudflareRequestValidationService {
    async fn verify(&self, token: String, ip: String) -> Result<(), AppError> {
        let body = serde_json::json!({
            "secret": &self.secret_key,
            "response": token,
            "remoteip": ip,
        });
        let body_string =
            serde_json::to_string(&body).map_err(|err| AppError::InternalError(err.to_string()))?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_body(Some(body_string.into()));

        let headers = Headers::new();
        headers
            .set("Content-Type", "application/json")
            .map_err(|err| AppError::InternalError(err.to_string()))?;
        init.with_headers(headers);

        let request = Request::new_with_init(&self.siteverify_url, &init)
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        let mut response = Fetch::Request(request).send().await.map_err(|e| {
            error!("Cloudflare request failed: {:?}", e);
            AppError::InternalError(e.to_string())
        })?;

        let turnstile_response: TurnstileResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Turnstile response: {:?}", e);
            AppError::InternalError(e.to_string())
        })?;

        info!("Turnstile response: {:?}", turnstile_response);

        if turnstile_response.success {
            Ok(())
        } else {
            if let Some(error_codes) = &turnstile_response.error_codes {
                info!("Turnstile validation failed with errors: {:?}", error_codes);

                for error_code in error_codes {
                    match error_code.as_str() {
                        "invalid-input-secret" => {
                            error!("Invalid secret key configured");
                            return Err(AppError::InternalError(
                                "Invalid secret key configured".into(),
                            ));
                        }
                        "invalid-input-response" => {
                            info!("Invalid or expired token");
                        }
                        "timeout-or-duplicate" => {
                            info!("Token timeout or duplicate submission");
                        }
                        _ => {
                            info!("Unknown error code: {}", error_code);
                        }
                    }
                }
            }

            Err(AppError::Unauthorised(
                "Turnstile validation failed".to_string(),
            ))
        }
    }
}
