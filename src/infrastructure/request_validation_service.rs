use std::sync::Arc;

use crate::application::{
    exceptions::AppError, request_validation_service::RequestValidationServiceTrait,
};
use async_trait::async_trait;
use serde::Deserialize;
use worker::{Fetch, Headers, Method, Request, RequestInit, console_error, console_log};

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
        siteverify_url: String,
        secret_key: String,
    ) -> Arc<dyn RequestValidationServiceTrait> {
        Arc::new(Self {
            siteverify_url,
            secret_key,
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
        let body_string = serde_json::to_string(&body).map_err(|_| AppError::InternalError)?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_body(Some(body_string.into()));

        let headers = Headers::new();
        headers
            .set("Content-Type", "application/json")
            .map_err(|_| AppError::InternalError)?;
        init.with_headers(headers);

        let request = Request::new_with_init(&self.siteverify_url, &init)
            .map_err(|_| AppError::InternalError)?;

        let mut response = Fetch::Request(request).send().await.map_err(|e| {
            console_error!("Cloudflare request failed: {:?}", e);
            AppError::InternalError
        })?;

        let turnstile_response: TurnstileResponse = response.json().await.map_err(|e| {
            console_error!("Failed to parse Turnstile response: {:?}", e);
            AppError::InternalError
        })?;

        console_log!("Turnstile response: {:?}", turnstile_response);

        if turnstile_response.success {
            Ok(())
        } else {
            if let Some(error_codes) = &turnstile_response.error_codes {
                console_log!("Turnstile validation failed with errors: {:?}", error_codes);

                for error_code in error_codes {
                    match error_code.as_str() {
                        "invalid-input-secret" => {
                            console_error!("Invalid secret key configured");
                            return Err(AppError::InternalError);
                        }
                        "invalid-input-response" => {
                            console_log!("Invalid or expired token");
                        }
                        "timeout-or-duplicate" => {
                            console_log!("Token timeout or duplicate submission");
                        }
                        _ => {
                            console_log!("Unknown error code: {}", error_code);
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
