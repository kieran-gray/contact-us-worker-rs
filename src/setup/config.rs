use worker::Env;

use crate::setup::exceptions::SetupError;

#[derive(Clone)]
pub struct Config {
    pub auth_issuer_url: String,
    pub auth_jwks_path: String,
    pub siteverify_url: String,
    pub secret_key: String,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env(env: &Env) -> Result<Self, SetupError> {
        let auth_issuer_url = env
            .var("AUTH_ISSUER_URL")
            .map_err(|_| SetupError::MissingVariable("AUTH_ISSUER_URL".to_string()))?
            .to_string();

        let auth_jwks_path = env
            .var("AUTH_JWKS_PATH")
            .map_err(|_| SetupError::MissingVariable("AUTH_JWKS_PATH".to_string()))?
            .to_string();

        let siteverify_url = env
            .var("TURNSTILE_SITEVERIFY_URL")
            .map_err(|_| SetupError::MissingVariable("TURNSTILE_SITEVERIFY_URL".to_string()))?
            .to_string();

        let secret_key = env
            .secret("TURNSTILE_SECRET_KEY")
            .map_err(|_| SetupError::MissingVariable("TURNSTILE_SECRET_KEY".to_string()))?
            .to_string();

        let allowed_origins = env
            .var("ALLOWED_ORIGINS")
            .map(|v| {
                v.to_string()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .unwrap_or_else(|_| vec![]);

        Ok(Config {
            auth_issuer_url,
            auth_jwks_path,
            siteverify_url,
            secret_key,
            allowed_origins,
        })
    }
}
