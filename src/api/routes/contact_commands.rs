use crate::{
    api::{cors::CorsContext, schemas::requests::CreateContactMessageRequest},
    application::exceptions::AppError,
    setup::app_state::AppState,
};
use tracing::{error, info};
use worker::{Request, Response, RouteContext};

pub async fn create_contact_message_handler(
    mut req: Request,
    ctx: RouteContext<AppState>,
    cors_context: CorsContext,
) -> worker::Result<Response> {
    let payload: CreateContactMessageRequest = match req.json().await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to parse request body: {:?}", e);
            let response = Response::from(AppError::ValidationError(
                "Failed to parse request body".into(),
            ));
            return Ok(cors_context.add_to_response(response));
        }
    };

    let ip_address = req
        .headers()
        .get("CF-Connecting-IP")
        .ok()
        .flatten()
        .unwrap_or_else(|| "0.0.0.0".to_string());

    match ctx
        .data
        .contact_message_service
        .create_message(
            payload.token,
            ip_address,
            payload.category,
            payload.email,
            payload.name,
            payload.message,
            payload.data,
        )
        .await
    {
        Ok(_) => {
            info!("Contact-us message created successfully.");
            let response = Response::from_json(&true)?;
            Ok(cors_context.add_to_response(response))
        }
        Err(e) => {
            error!("Failed to create message: {:?}", e);
            let response = Response::from(e);
            Ok(cors_context.add_to_response(response))
        }
    }
}
