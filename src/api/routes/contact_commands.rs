use crate::{
    api::{schemas::requests::CreateContactMessageRequest, utils::handlers::PublicContext},
    application::exceptions::AppError,
};
use tracing::{error, info};
use worker::{Request, Response};

pub async fn create_contact_message_handler(
    mut req: Request,
    ctx: PublicContext,
) -> worker::Result<Response> {
    let payload: CreateContactMessageRequest = match req.json().await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to parse request body: {:?}", e);
            let response = Response::from(AppError::ValidationError(
                "Failed to parse request body".into(),
            ));
            return Ok(ctx.with_cors(response));
        }
    };

    match ctx
        .app_state
        .contact_message_service
        .create_message(
            payload.token,
            ctx.connecting_ip.clone(),
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
            Ok(ctx.with_cors(response))
        }
        Err(e) => {
            error!("Failed to create message: {:?}", e);
            let response = Response::from(e);
            Ok(ctx.with_cors(response))
        }
    }
}
