use worker::{Request, Response, RouteContext};

use crate::setup::app_state::AppState;

use crate::api::cors::CorsContext;

pub fn create_options_handler(
    req: Request,
    ctx: RouteContext<AppState>,
) -> worker::Result<Response> {
    let cors_context = CorsContext::new(ctx.data.config.allowed_origins, &req);

    match cors_context.validate(&req) {
        Ok(_) => cors_context.preflight_response(),
        Err(response) => Ok(response),
    }
}
