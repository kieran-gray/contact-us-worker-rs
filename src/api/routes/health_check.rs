use crate::api::schemas::responses::ApiResponse;
use worker::{Request, Response, RouteContext};

pub async fn health_check_handler(
    _req: Request,
    _ctx: RouteContext<()>,
) -> worker::Result<Response> {
    ApiResponse::success(true).to_response()
}
