use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    application::{dtos::ContactMessageDTO, exceptions::AppError},
    domain::{
        entity::ContactMessage,
        repository::ContactMessageRepository as ContactMessageRepositoryInterface,
    },
};

#[async_trait(?Send)]
pub trait ContactMessageQueryServiceTrait {
    async fn get_messages(&self) -> Result<Vec<ContactMessageDTO>, AppError>;
}

pub struct ContactMessageQueryService {
    pub repo: Arc<dyn ContactMessageRepositoryInterface + Send + Sync>,
}

impl ContactMessageQueryService {
    pub fn create(contact_repo: Arc<dyn ContactMessageRepositoryInterface>) -> Arc<Self> {
        Arc::new(Self { repo: contact_repo })
    }
}

#[async_trait(?Send)]
impl ContactMessageQueryServiceTrait for ContactMessageQueryService {
    async fn get_messages(&self) -> Result<Vec<ContactMessageDTO>, AppError> {
        let contact_messages: Vec<ContactMessage> = self
            .repo
            .get()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let contact_message_dtos: Vec<ContactMessageDTO> = contact_messages
            .into_iter()
            .map(ContactMessageDTO::from)
            .collect();
        Ok(contact_message_dtos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{enums::ContactMessageCategory, exceptions::RepositoryError};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockContactMessageRepository {
        contact_messages: Arc<Mutex<Vec<ContactMessage>>>,
        should_save_fail: Arc<Mutex<bool>>,
        should_get_fail: Arc<Mutex<bool>>,
    }

    impl MockContactMessageRepository {
        fn new() -> Self {
            Self {
                contact_messages: Arc::new(Mutex::new(Vec::new())),
                should_save_fail: Arc::new(Mutex::new(false)),
                should_get_fail: Arc::new(Mutex::new(false)),
            }
        }

        fn set_get_should_fail(&self, should_fail: bool) {
            *self.should_get_fail.lock().unwrap() = should_fail;
        }
    }

    #[async_trait(?Send)]
    impl ContactMessageRepositoryInterface for MockContactMessageRepository {
        async fn save(&self, contact: &ContactMessage) -> Result<bool, RepositoryError> {
            if *self.should_save_fail.lock().unwrap() {
                return Err(RepositoryError::DatabaseError(
                    "Mock database error on save".into(),
                ));
            }

            let mut contact_messages = self.contact_messages.lock().unwrap();

            if let Some(existing_index) = contact_messages.iter().position(|c| c.id == contact.id) {
                contact_messages[existing_index] = contact.clone();
            } else {
                contact_messages.push(contact.clone());
            }

            Ok(true)
        }

        async fn get(&self) -> Result<Vec<ContactMessage>, RepositoryError> {
            if *self.should_get_fail.lock().unwrap() {
                return Err(RepositoryError::DatabaseError(
                    "Mock database error on get".into(),
                ));
            }

            Ok(self.contact_messages.lock().unwrap().to_owned())
        }
    }

    fn create_query_service() -> (
        Arc<ContactMessageQueryService>,
        Arc<MockContactMessageRepository>,
    ) {
        let mock_repo = Arc::new(MockContactMessageRepository::new());
        let service = ContactMessageQueryService::create(mock_repo.clone());
        (service, mock_repo)
    }

    #[tokio::test]
    async fn test_get_messages_empty() {
        let (query_service, _mock_repo) = create_query_service();

        let result = query_service.get_messages().await;

        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_get_messages_success() {
        let (query_service, mock_repo) = create_query_service();

        let message1 = ContactMessage::create(
            ContactMessageCategory::ERROR,
            "user1@example.com".to_string(),
            "User One".to_string(),
            "First message".to_string(),
            None,
        )
        .unwrap();

        let mut data = HashMap::new();
        data.insert("rating".to_string(), "5".to_string());

        let message2 = ContactMessage::create(
            ContactMessageCategory::IDEA,
            "user2@example.com".to_string(),
            "User Two".to_string(),
            "Second message".to_string(),
            Some(data.clone()),
        )
        .unwrap();

        mock_repo.save(&message1).await.unwrap();
        mock_repo.save(&message2).await.unwrap();

        let result = query_service.get_messages().await;

        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 2);
    }

    #[tokio::test]
    async fn test_get_messages_dto_conversion() {
        let (query_service, mock_repo) = create_query_service();

        let mut data = HashMap::new();
        data.insert("source".to_string(), "website".to_string());
        data.insert("rating".to_string(), "4".to_string());

        let message = ContactMessage::create(
            ContactMessageCategory::TESTIMONIAL,
            "test@example.com".to_string(),
            "Test User".to_string(),
            "Great product!".to_string(),
            Some(data.clone()),
        )
        .unwrap();

        let message_id = message.id.to_string();
        mock_repo.save(&message).await.unwrap();

        let result = query_service.get_messages().await;

        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);

        let dto = &messages[0];
        assert_eq!(dto.id, message_id);
        assert_eq!(dto.category, "TESTIMONIAL");
        assert_eq!(dto.email, "test@example.com");
        assert_eq!(dto.name, "Test User");
        assert_eq!(dto.message, "Great product!");
        assert_eq!(dto.data, Some(data));
    }

    #[tokio::test]
    async fn test_get_messages_database_error() {
        let (query_service, mock_repo) = create_query_service();

        mock_repo.set_get_should_fail(true);

        let result = query_service.get_messages().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::DatabaseError(_) => {}
            _ => panic!("Expected DatabaseError"),
        }
    }

    #[tokio::test]
    async fn test_get_messages_multiple_categories() {
        let (query_service, mock_repo) = create_query_service();

        let categories = vec![
            ContactMessageCategory::ERROR,
            ContactMessageCategory::IDEA,
            ContactMessageCategory::TESTIMONIAL,
            ContactMessageCategory::OTHER,
        ];

        for (i, category) in categories.iter().enumerate() {
            let message = ContactMessage::create(
                category.clone(),
                format!("user{}@example.com", i),
                format!("User {}", i),
                format!("Message {}", i),
                None,
            )
            .unwrap();
            mock_repo.save(&message).await.unwrap();
        }

        let result = query_service.get_messages().await;

        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 4);

        assert_eq!(messages[0].category, "ERROR");
        assert_eq!(messages[1].category, "IDEA");
        assert_eq!(messages[2].category, "TESTIMONIAL");
        assert_eq!(messages[3].category, "OTHER");
    }
}
