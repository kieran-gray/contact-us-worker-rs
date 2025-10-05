use crate::{
    api::{
        cors::CorsHeaders,
        schemas::{requests::CreateContactMessageRequest, responses::ApiResponse},
    },
    application::exceptions::AppError,
    setup::{app_state::AppState, config::Config},
};
use worker::{Request, Response, RouteContext, console_error, console_log};

pub async fn create_contact_message_handler(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let origin: Option<String> = req.headers().get("Origin").ok().flatten();

    let config = Config::from_env(&ctx.env).map_err(worker::Error::RustError)?;
    let cors = CorsHeaders::new(config.allowed_origins);

    if !cors.is_allowed(&origin) {
        console_error!("Blocked unauthorised origin: {:?}", origin);
        return ApiResponse::<()>::failure(403, "Forbidden").to_response();
    }

    let payload: CreateContactMessageRequest = match req.json().await {
        Ok(p) => p,
        Err(e) => {
            console_error!("Failed to parse request body: {:?}", e);
            let response = ApiResponse::<()>::failure(400, "Invalid request body").to_response()?;
            return cors.add_to_response(response, origin);
        }
    };

    let app_state = match AppState::from_env(&ctx.env) {
        Ok(state) => state,
        Err(e) => {
            console_error!("Failed to create app state: {:?}", e);
            let response =
                ApiResponse::<()>::failure(500, "Internal Server Error").to_response()?;
            return cors.add_to_response(response, origin);
        }
    };

    let client_ip = req
        .headers()
        .get("CF-Connecting-IP")
        .ok()
        .flatten()
        .unwrap_or_else(|| "0.0.0.0".to_string());

    if let Err(e) = app_state
        .request_validation_service
        .verify(payload.token, client_ip)
        .await
    {
        console_error!("Turnstile validation failed: {:?}", e);
        let response =
            ApiResponse::<()>::failure(401, "Request validation failed").to_response()?;
        return cors.add_to_response(response, origin);
    }

    match app_state
        .contact_message_service
        .create_message(
            payload.category,
            payload.email,
            payload.name,
            payload.message,
            payload.data,
        )
        .await
    {
        Ok(_) => {
            console_log!("Contact-us message created successfully.");
            let response = ApiResponse::success(true).to_response()?;
            cors.add_to_response(response, origin)
        }
        Err(e) => {
            console_error!("Failed to create message: {:?}", e);
            let response = match e {
                AppError::ValidationError(msg) => {
                    ApiResponse::<()>::failure(400, msg).to_response()?
                }
                _ => ApiResponse::<()>::failure(500, "Failed to save message").to_response()?,
            };
            cors.add_to_response(response, origin)
        }
    }
}
