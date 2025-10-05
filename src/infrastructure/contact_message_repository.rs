use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::{
    entity::ContactMessage, exceptions::RepositoryError,
    repository::ContactMessageRepository as ContactMessageRepositoryInterface,
};
use async_trait::async_trait;
use chrono::Utc;
use worker::D1Database;

pub struct ContactMessageRepository {
    db: D1Database,
}

impl ContactMessageRepository {
    pub fn create(db: D1Database) -> Arc<dyn ContactMessageRepositoryInterface> {
        Arc::new(Self { db })
    }

    fn serialize_data(data: &Option<HashMap<String, String>>) -> Result<String, RepositoryError> {
        match data {
            Some(m) => serde_json::to_string(m).map_err(|e| {
                RepositoryError::DatabaseError(format!("JSON serialization failed: {e}"))
            }),
            None => Ok("null".to_string()),
        }
    }
}

#[async_trait(?Send)]
impl ContactMessageRepositoryInterface for ContactMessageRepository {
    async fn save(&self, contact: &ContactMessage) -> Result<bool, RepositoryError> {
        let data_json = Self::serialize_data(&contact.data)?;
        let created_at = Utc::now().timestamp() as f64;

        let statement = self.db.prepare(
            "INSERT INTO contact_messages (id, category, email, name, message, data, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        );

        let result = statement
            .bind(&[
                contact.id.clone().into(),
                contact.category.to_string().into(),
                contact.email.clone().into(),
                contact.name.clone().into(),
                contact.message.clone().into(),
                data_json.into(),
                created_at.into(),
            ])
            .map_err(|e| RepositoryError::DatabaseError(format!("Failed to bind parameters: {e}")))?
            .run()
            .await
            .map_err(|e| RepositoryError::DatabaseError(format!("Failed to execute query: {e}")))?;

        Ok(result.success())
    }
}
