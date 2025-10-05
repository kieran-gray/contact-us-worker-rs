use std::sync::Arc;

use worker::{Env, console_error};

use crate::{
    application::{
        contact_message_service::ContactMessageService,
        request_validation_service::RequestValidationServiceTrait,
    },
    infrastructure::{
        contact_message_repository::ContactMessageRepository,
        request_validation_service::CloudflareRequestValidationService,
    },
    setup::config::Config,
};

pub struct AppState {
    pub contact_message_service: ContactMessageService,
    pub request_validation_service: Arc<dyn RequestValidationServiceTrait>,
}

impl AppState {
    pub fn from_env(env: &Env) -> Result<Self, String> {
        let config = Config::from_env(env)?;

        let db = env.d1("DB").map_err(|e| {
            console_error!("Failed to get D1 binding: {:?}", e);
            "Database unavailable".to_string()
        })?;

        let request_validation_service =
            CloudflareRequestValidationService::create(config.siteverify_url, config.secret_key);

        let contact_message_repository = ContactMessageRepository::create(db);
        let contact_message_service = ContactMessageService::create(contact_message_repository);

        Ok(Self {
            contact_message_service,
            request_validation_service,
        })
    }
}
