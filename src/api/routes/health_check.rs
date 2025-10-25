use worker::{Request, Response, RouteContext};

use crate::setup::config::Config;

pub async fn health_check_handler(
    _req: Request,
    _ctx: RouteContext<Config>,
) -> worker::Result<Response> {
    Response::from_json(&true)
}
