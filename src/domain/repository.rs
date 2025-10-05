use super::entity::ContactMessage;
use super::exceptions::RepositoryError;

use async_trait::async_trait;

#[async_trait(?Send)]
/// Trait representing repository-level operations for Contact Message entities.
/// Provides methods for saving, retrieving, updating, and deleting Contact Messages in the database.
pub trait ContactMessageRepository: Send + Sync {
    async fn save(&self, contact: &ContactMessage) -> Result<bool, RepositoryError>;
}
