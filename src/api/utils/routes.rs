use worker::{Request, Response, Result, RouteContext};

use crate::{
    api::utils::handlers::{AuthenticatedContext, PublicContext},
    setup::config::Config,
};

pub async fn authenticated<F, Fut>(
    handler: F,
    req: Request,
    ctx: RouteContext<Config>,
) -> Result<Response>
where
    F: Fn(Request, AuthenticatedContext) -> Fut,
    Fut: std::future::Future<Output = Result<Response>>,
{
    let auth_ctx = match AuthenticatedContext::from_request(&req, &ctx).await {
        Ok(ctx) => ctx,
        Err(response) => return Ok(response),
    };
    handler(req, auth_ctx).await
}

pub async fn public<F, Fut>(handler: F, req: Request, ctx: RouteContext<Config>) -> Result<Response>
where
    F: Fn(Request, PublicContext) -> Fut,
    Fut: std::future::Future<Output = Result<Response>>,
{
    let public_context = match PublicContext::from_request(&req, &ctx).await {
        Ok(ctx) => ctx,
        Err(response) => return Ok(response),
    };
    handler(req, public_context).await
}
