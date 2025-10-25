use crate::{api::cors::CorsContext, setup::app_state::AppState};
use tracing::{debug, error};
use worker::{Request, Response, RouteContext};

pub async fn get_contact_messages_handler(
    _req: Request,
    ctx: RouteContext<AppState>,
    cors_context: CorsContext,
    user_id: String,
) -> worker::Result<Response> {
    match ctx.data.contact_message_query_service.get_messages().await {
        Ok(contact_messages) => {
            debug!(user_id = ?user_id, "Contact Messages successfully fetched");
            let response = Response::from_json(&contact_messages)?;
            Ok(cors_context.add_to_response(response))
        }
        Err(e) => {
            error!(user_id = ?user_id, "Failed to get messages: {:?}", e);
            let response = Response::from(e);
            Ok(cors_context.add_to_response(response))
        }
    }
}
