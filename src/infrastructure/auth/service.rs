use async_trait::async_trait;
use tracing::{debug, error, info, warn};
use worker::{Fetch, Method, Request, RequestInit};

use super::jwks::{JWKS, Jwk, token_kid, validate};
use crate::infrastructure::storage::cache::CacheTrait;
use std::sync::Arc;

#[derive(Debug)]
pub enum AuthError {
    InvalidToken(String),
    Unauthorised(String),
    InternalError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {msg}"),
            AuthError::Unauthorised(msg) => write!(f, "Unauthorised: {msg}"),
            AuthError::InternalError(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for AuthError {}

#[async_trait(?Send)]
pub trait AuthServiceTrait {
    async fn verify(&self, token: &str) -> Result<String, AuthError>;
}

pub struct AuthService<C: CacheTrait> {
    pub auth_issuer_url: String,
    pub auth_jwks_path: String,
    pub cache: Arc<C>,
}

impl<C: CacheTrait> AuthService<C> {
    pub fn create(auth_issuer_url: String, auth_jwks_path: String, cache: C) -> Self {
        Self {
            auth_issuer_url,
            auth_jwks_path,
            cache: Arc::new(cache),
        }
    }

    pub async fn fetch_jwks(&self) -> Result<JWKS, AuthError> {
        let jwks_url = format!("{}{}", self.auth_issuer_url, self.auth_jwks_path);

        match self.cache.get::<JWKS>(jwks_url.clone()).await {
            Ok(Some(jwks)) => {
                info!(cache_status = "hit", "Retrieved JWKS from cache");
                return Ok(jwks);
            }
            Ok(None) => {
                debug!(
                    cache_status = "miss",
                    "Cache miss, fetching from auth provider"
                );
            }
            Err(err) => {
                warn!(error = %err, cache_status = "failed", "Cache retrieval failed")
            }
        }

        let mut init = RequestInit::new();
        init.with_method(Method::Get);

        let request = Request::new_with_init(&jwks_url, &init)
            .map_err(|e| AuthError::InternalError(format!("Failed to create JWKS request: {e}")))?;

        let mut response = Fetch::Request(request).send().await.map_err(|e| {
            error!(
                error = ?e,
                issuer_url = %self.auth_issuer_url,
                "JWKS request failed"
            );
            AuthError::InternalError(format!("JWKS request failed: {e}"))
        })?;

        let jwks_response: JWKS = response.json().await.map_err(|e| {
            error!(error = ?e, "Failed to parse JWKS response");
            AuthError::InternalError(format!("Failed to parse JWKS response: {e}"))
        })?;

        let _ = self
            .cache
            .set(jwks_url.clone(), jwks_response.clone())
            .await;

        Ok(jwks_response)
    }

    pub async fn get_jwk(&self, token: &str) -> Result<Jwk, AuthError> {
        let jwks = self.fetch_jwks().await?;
        let kid = match token_kid(token) {
            Ok(Some(kid)) => kid,
            _ => {
                error!(error = "missing_kid", "Unable to extract kid from token");
                return Err(AuthError::InternalError(
                    "Unable to extract kid from token".into(),
                ));
            }
        };

        match jwks.find(&kid) {
            Some(jwk) => Ok(jwk.to_owned()),
            None => {
                error!(kid = %kid, "JWK not found for kid");
                Err(AuthError::InternalError(format!(
                    "JWK not found for kid: {kid}"
                )))
            }
        }
    }
}

#[async_trait(?Send)]
impl<C: CacheTrait> AuthServiceTrait for AuthService<C> {
    async fn verify(&self, token: &str) -> Result<String, AuthError> {
        let token = token.strip_prefix("Bearer ").unwrap_or(token);

        let jwk = self.get_jwk(token).await?;

        match validate(token, &jwk, Some(&self.auth_issuer_url), None, true) {
            Ok(claims) => {
                let sub = claims.get("sub").and_then(|v| v.as_str()).ok_or_else(|| {
                    error!(error = "missing_subject_claim", "Token validation failed");
                    AuthError::Unauthorised("Invalid subject claim".to_string())
                })?;
                info!(user_id = %sub, "Authentication successful");
                Ok(sub.to_string())
            }
            Err(e) => {
                warn!(
                    error = ?e,
                    error_type = "token_validation_failed",
                    "Authentication failed"
                );
                Err(AuthError::Unauthorised(e.to_string()))
            }
        }
    }
}
