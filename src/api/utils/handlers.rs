use tracing::{error, info};
use worker::{Request, Response, RouteContext};

use crate::{
    application::exceptions::AppError,
    setup::{
        app_state::{AuthenticatedState, CoreState},
        config::Config,
    },
};

use crate::api::cors::CorsContext;

pub struct AuthenticatedContext {
    pub app_state: AuthenticatedState,
    pub cors_context: CorsContext,
    pub user_id: String,
}

pub struct PublicContext {
    pub app_state: CoreState,
    pub cors_context: CorsContext,
    pub connecting_ip: String,
}

impl AuthenticatedContext {
    pub async fn from_request(req: &Request, ctx: &RouteContext<Config>) -> Result<Self, Response> {
        let cors_context = CorsContext::new(ctx.data.allowed_origins.clone(), req);
        cors_context.validate(req)?;

        let app_state = match AuthenticatedState::from_env(&ctx.env) {
            Ok(state) => state,
            Err(e) => {
                error!(error = ?e, "Failed to create app state");
                let response =
                    Response::from(AppError::InternalError("Failed to create app state".into()));
                return Err(cors_context.add_to_response(response));
            }
        };

        let authorization = match req.headers().get("Authorization").ok().flatten() {
            Some(auth_header) => auth_header,
            None => {
                let response = Response::from(AppError::Unauthorised("Not Authenticated".into()));
                return Err(cors_context.add_to_response(response));
            }
        };

        let user_id = match app_state.auth_service.verify(&authorization).await {
            Ok(user_id) => user_id,
            Err(e) => {
                info!(error = ?e, "User verification failed");
                let response =
                    Response::from(AppError::Unauthorised("User verification failed".into()));
                return Err(cors_context.add_to_response(response));
            }
        };

        Ok(Self {
            app_state,
            cors_context,
            user_id,
        })
    }

    pub fn with_cors(&self, response: Response) -> Response {
        self.cors_context.add_to_response(response)
    }
}

impl PublicContext {
    pub async fn from_request(req: &Request, ctx: &RouteContext<Config>) -> Result<Self, Response> {
        let cors_context = CorsContext::new(ctx.data.allowed_origins.clone(), req);
        cors_context.validate(req)?;

        let connecting_ip = req
            .headers()
            .get("CF-Connecting-IP")
            .ok()
            .flatten()
            .unwrap_or_else(|| "0.0.0.0".to_string());

        let app_state = match CoreState::from_env(&ctx.env) {
            Ok(state) => state,
            Err(e) => {
                error!(error = ?e, "Failed to create app state");
                let response =
                    Response::from(AppError::InternalError("Failed to create app state".into()));
                return Err(cors_context.add_to_response(response));
            }
        };

        Ok(Self {
            app_state,
            cors_context,
            connecting_ip,
        })
    }

    pub fn with_cors(&self, response: Response) -> Response {
        self.cors_context.add_to_response(response)
    }
}

pub fn create_options_handler(req: Request, ctx: RouteContext<Config>) -> worker::Result<Response> {
    let cors_context = CorsContext::new(ctx.data.allowed_origins, &req);

    match cors_context.validate(&req) {
        Ok(_) => cors_context.preflight_response(),
        Err(response) => Ok(response),
    }
}
