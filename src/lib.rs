pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod setup;

use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_log!("Worker handling request: {} {}", req.method(), req.path());

    let router = api::router::create_router();
    router.run(req, env).await
}
