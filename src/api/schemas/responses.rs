use serde::{Deserialize, Serialize};
use worker::Response;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub status: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Self {
        Self {
            status: 200,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn failure(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            data: None,
        }
    }

    pub fn to_response(self) -> worker::Result<Response> {
        let status = self.status;
        let mut response = Response::from_json(&self)?;
        response
            .headers_mut()
            .set("Content-Type", "application/json")?;
        Ok(response.with_status(status))
    }
}
