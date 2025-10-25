use crate::api::utils::handlers::AuthenticatedContext;
use tracing::error;
use worker::{Request, Response};

pub async fn get_contact_messages_handler(
    _req: Request,
    ctx: AuthenticatedContext,
) -> worker::Result<Response> {
    match ctx
        .app_state
        .contact_message_query_service
        .get_messages()
        .await
    {
        Ok(contact_messages) => {
            let response = Response::from_json(&contact_messages)?;
            Ok(ctx.with_cors(response))
        }
        Err(e) => {
            error!("Failed to get messages: {:?}", e);
            let response = Response::from(e);
            Ok(ctx.with_cors(response))
        }
    }
}
