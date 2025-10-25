use std::sync::Arc;

use super::models::ContactMessageRow;
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
}

#[async_trait(?Send)]
impl ContactMessageRepositoryInterface for ContactMessageRepository {
    async fn save(&self, contact: &ContactMessage) -> Result<bool, RepositoryError> {
        let row = ContactMessageRow::from_contact_message(contact)?;
        let created_at = Utc::now().timestamp() as f64;

        let statement = self.db.prepare(
            "INSERT INTO contact_messages (id, category, email, name, message, data, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        );

        let result = statement
            .bind(&[
                row.id.into(),
                row.category.into(),
                row.email.into(),
                row.name.into(),
                row.message.into(),
                row.data.into(),
                created_at.into(),
            ])
            .map_err(|e| RepositoryError::DatabaseError(format!("Failed to bind parameters: {e}")))?
            .run()
            .await
            .map_err(|e| RepositoryError::DatabaseError(format!("Failed to execute query: {e}")))?;

        Ok(result.success())
    }

    async fn get(&self) -> Result<Vec<ContactMessage>, RepositoryError> {
        let statement = self
            .db
            .prepare("SELECT id, category, email, name, message, data FROM contact_messages");

        let result = statement
            .all()
            .await
            .map_err(|e| RepositoryError::DatabaseError(format!("Failed to execute query: {e}")))?;

        let rows: Vec<ContactMessageRow> = result.results().map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to deserialize rows: {e}"))
        })?;

        rows.into_iter()
            .map(|row| row.to_contact_message())
            .collect()
    }
}
