use worker::Env;

pub struct Config {
    pub siteverify_url: String,
    pub secret_key: String,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env(env: &Env) -> Result<Self, String> {
        let siteverify_url = env
            .var("TURNSTILE_SITEVERIFY_URL")
            .map_err(|_| "Missing siteverify URL".to_string())?
            .to_string();

        let secret_key = env
            .secret("TURNSTILE_SECRET_KEY")
            .map_err(|_| "Missing secret key".to_string())?
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
            siteverify_url,
            secret_key,
            allowed_origins,
        })
    }
}
