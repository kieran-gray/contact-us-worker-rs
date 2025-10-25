pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod setup;

use tracing::{Instrument, error, info, info_span};
use tracing_subscriber::{
    EnvFilter,
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{MakeConsoleWriter, performance_layer};
use uuid::Uuid;

use serde_json::json;

use worker::*;

use crate::setup::config::Config;

#[event(start)]
fn start() {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(true)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter);

    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer.with_filter(EnvFilter::new(log_level)))
        .with(perf_layer)
        .init();
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let request_id = Uuid::now_v7();

    async move {
        info!(method = %req.method(), path = %req.path(), "START");

        let config = match Config::from_env(&env) {
            Ok(config) => config,
            Err(e) => {
                error!(error = ?e, "Failed to load config");
                let json = json!({"message": "Failed to load config".to_string()});
                return Ok(Response::from_json(&json)?.with_status(500));
            }
        };

        let router = api::router::create_router(config);
        let result = router.run(req, env).await;

        match &result {
            Ok(res) => info!(status = res.status_code(), "SUCCESS"),
            Err(e) => error!(error = ?e, "FAILURE"),
        }

        result
    }
    .instrument(info_span!("request", request_id = %request_id))
    .await
}
