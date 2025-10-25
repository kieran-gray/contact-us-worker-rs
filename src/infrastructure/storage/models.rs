use crate::domain::{
    entity::ContactMessage, enums::ContactMessageCategory, exceptions::RepositoryError,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactMessageRow {
    pub id: String,
    pub category: String,
    pub email: String,
    pub name: String,
    pub message: String,
    pub data: String,
}

impl ContactMessageRow {
    pub fn from_contact_message(contact: &ContactMessage) -> Result<Self, RepositoryError> {
        let data_json = match &contact.data {
            Some(data_map) => serde_json::to_string(data_map).map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to serialize data: {e}"))
            })?,
            None => "null".to_string(),
        };

        Ok(Self {
            id: contact.id.to_string(),
            category: contact.category.to_string(),
            email: contact.email.clone(),
            name: contact.name.clone(),
            message: contact.message.clone(),
            data: data_json,
        })
    }

    pub fn to_contact_message(&self) -> Result<ContactMessage, RepositoryError> {
        let id = Uuid::parse_str(&self.id)
            .map_err(|e| RepositoryError::DatabaseError(format!("Invalid UUID: {e}")))?;

        let category = ContactMessageCategory::from_str(&self.category).map_err(|e| {
            RepositoryError::DatabaseError(format!("Invalid category '{}': {}", self.category, e))
        })?;

        let data = if self.data == "null" {
            None
        } else {
            Some(serde_json::from_str(&self.data).map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to parse data JSON: {e}"))
            })?)
        };

        Ok(ContactMessage {
            id,
            category,
            email: self.email.clone(),
            name: self.name.clone(),
            message: self.message.clone(),
            data,
        })
    }
}
