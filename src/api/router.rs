use worker::*;

use crate::api::routes::{
    contact_commands::create_contact_message_handler, health_check::health_check_handler,
};
use crate::{api::cors::CorsHeaders, setup::config::Config};

pub fn create_router() -> Router<'static, ()> {
    let router = Router::new();
    router
        .get_async("/api/v1/health-check/", health_check_handler)
        .post_async("/api/v1/contact-us/", create_contact_message_handler)
        .options("/api/v1/contact-us/", |req, ctx| {
            let config = Config::from_env(&ctx.env)?;
            let cors = CorsHeaders::new(config.allowed_origins);
            let origin = req.headers().get("Origin").ok().flatten();

            cors.preflight_response(origin)
        })
}
