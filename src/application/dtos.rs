use std::collections::HashMap;

use crate::domain::entity::ContactMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactMessageDTO {
    pub id: String,
    pub category: String,
    pub email: String,
    pub name: String,
    pub message: String,
    pub data: Option<HashMap<String, String>>,
}

impl From<ContactMessage> for ContactMessageDTO {
    fn from(contact_message: ContactMessage) -> Self {
        Self {
            id: contact_message.id.to_string(),
            category: contact_message.category.to_string(),
            email: contact_message.email,
            name: contact_message.name,
            message: contact_message.message,
            data: contact_message.data,
        }
    }
}
