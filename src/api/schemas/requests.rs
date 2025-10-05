use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Deserialize, Serialize)]
pub struct CreateContactMessageRequest {
    pub token: String,
    pub category: String,
    pub email: String,
    pub name: String,
    pub message: String,
    pub data: Option<HashMap<String, String>>,
}
