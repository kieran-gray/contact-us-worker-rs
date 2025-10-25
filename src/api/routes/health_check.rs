use worker::{Request, Response, RouteContext};

use crate::setup::app_state::AppState;

pub async fn health_check_handler(
    _req: Request,
    _ctx: RouteContext<AppState>,
) -> worker::Result<Response> {
    Response::from_json(&true)
}
