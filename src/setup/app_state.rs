use std::sync::Arc;

use tracing::error;
use worker::Env;

use crate::{
    application::services::{
        contact_message_query_service::{
            ContactMessageQueryService, ContactMessageQueryServiceTrait,
        },
        contact_message_service::{ContactMessageService, ContactMessageServiceTrait},
    },
    infrastructure::{
        request_validation_service::CloudflareRequestValidationService,
        storage::contact_message_repository::ContactMessageRepository,
    },
    setup::{config::Config, exceptions::SetupError},
};

use crate::infrastructure::auth::service::{AuthService, AuthServiceTrait};
use crate::infrastructure::storage::cache::KVCache;

pub struct CoreState {
    pub contact_message_service: Arc<dyn ContactMessageServiceTrait>,
}

pub struct AuthenticatedState {
    pub contact_message_query_service: Arc<dyn ContactMessageQueryServiceTrait>,
    pub auth_service: Box<dyn AuthServiceTrait>,
}

impl CoreState {
    pub fn from_env(env: &Env) -> Result<Self, SetupError> {
        let config = Config::from_env(env)?;

        let db = env.d1("DB").map_err(|e| {
            error!("Failed to get D1 binding: {:?}", e);
            SetupError::MissingBinding("DB".to_string())
        })?;

        let request_validation_service =
            CloudflareRequestValidationService::create(config.siteverify_url, config.secret_key);

        let contact_message_repository = ContactMessageRepository::create(db);
        let contact_message_service = ContactMessageService::create(
            contact_message_repository.clone(),
            request_validation_service,
        );

        Ok(Self {
            contact_message_service,
        })
    }
}

impl AuthenticatedState {
    pub fn from_env(env: &Env) -> Result<Self, SetupError> {
        let config = Config::from_env(env)?;

        let db = env.d1("DB").map_err(|e| {
            error!("Failed to get D1 binding: {:?}", e);
            SetupError::MissingBinding("DB".to_string())
        })?;

        let jwks_cache = env.kv("AUTH_JWKS_CACHE").map_err(|e| {
            error!("Failed to get KV binding: {:?}", e);
            SetupError::MissingBinding("AUTH_JWKS_CACHE".to_string())
        })?;

        let jwks_cache_adapter = KVCache::create(jwks_cache, 43200);

        let auth_service = Box::new(AuthService::create(
            config.auth_issuer_url,
            config.auth_jwks_path,
            jwks_cache_adapter,
        ));

        let contact_message_repository = ContactMessageRepository::create(db);
        let contact_message_query_service =
            ContactMessageQueryService::create(contact_message_repository.clone());

        Ok(Self {
            contact_message_query_service,
            auth_service,
        })
    }
}
