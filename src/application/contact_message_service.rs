use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    application::exceptions::AppError,
    domain::{
        entity::ContactMessage, enums::ContactMessageCategory,
        repository::ContactMessageRepository as ContactMessageRepositoryInterface,
    },
};

pub struct ContactMessageService {
    pub repo: Arc<dyn ContactMessageRepositoryInterface + Send + Sync>,
}

impl ContactMessageService {
    pub fn create(contact_repo: Arc<dyn ContactMessageRepositoryInterface>) -> Self {
        Self { repo: contact_repo }
    }

    pub async fn create_message(
        &self,
        category: String,
        email: String,
        name: String,
        message: String,
        data: Option<HashMap<String, String>>,
    ) -> Result<(), AppError> {
        let parsed_category = ContactMessageCategory::from_str(&category);
        if parsed_category.is_err() {
            return Err(AppError::ValidationError(format!(
                "Category '{category}' is invalid"
            )));
        }
        let category = parsed_category.unwrap();
        let contact_message = ContactMessage::create(category, email, name, message, data)
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        self.repo
            .save(&contact_message)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::exceptions::RepositoryError;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockContactMessageRepository {
        contact_messages: Arc<Mutex<Vec<ContactMessage>>>,
        should_save_fail: Arc<Mutex<bool>>,
    }

    impl MockContactMessageRepository {
        fn new() -> Self {
            Self {
                contact_messages: Arc::new(Mutex::new(Vec::new())),
                should_save_fail: Arc::new(Mutex::new(false)),
            }
        }

        fn set_save_should_fail(&self, should_fail: bool) {
            *self.should_save_fail.lock().unwrap() = should_fail;
        }

        fn get_all_contact_messages(&self) -> Vec<ContactMessage> {
            self.contact_messages.lock().unwrap().clone()
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
    }

    fn create_service() -> (ContactMessageService, Arc<MockContactMessageRepository>) {
        let mock_repo = Arc::new(MockContactMessageRepository::new());
        let service = ContactMessageService::create(mock_repo.clone());
        (service, mock_repo)
    }

    #[tokio::test]
    async fn test_create_message_success() {
        let (service, mock_repo) = create_service();

        let result = service
            .create_message(
                "ERROR".to_string(),
                "test@example.com".to_string(),
                "John Doe".to_string(),
                "Test message".to_string(),
                None,
            )
            .await;

        assert!(result.is_ok());

        let saved_contact_messages = mock_repo.get_all_contact_messages();
        assert_eq!(saved_contact_messages.len(), 1);

        let saved_contact = &saved_contact_messages[0];
        assert_eq!(saved_contact.category, ContactMessageCategory::ERROR);
        assert_eq!(saved_contact.email, "test@example.com");
        assert_eq!(saved_contact.name, "John Doe");
        assert_eq!(saved_contact.message, "Test message");
        assert_eq!(saved_contact.data, None);
    }

    #[tokio::test]
    async fn test_create_message_with_data() {
        let (service, mock_repo) = create_service();

        let mut data = HashMap::new();
        data.insert("rating".to_string(), "5".to_string());
        data.insert("testimonial".to_string(), "I love quest-lock".to_string());

        let result = service
            .create_message(
                "IDEA".to_string(),
                "user@example.com".to_string(),
                "Jane Smith".to_string(),
                "Feature request".to_string(),
                Some(data.clone()),
            )
            .await;

        assert!(result.is_ok());

        let saved_contact_messages = mock_repo.get_all_contact_messages();
        assert_eq!(saved_contact_messages.len(), 1);

        let saved_contact = &saved_contact_messages[0];
        assert_eq!(saved_contact.category, ContactMessageCategory::IDEA);
        assert_eq!(saved_contact.email, "user@example.com");
        assert_eq!(saved_contact.name, "Jane Smith");
        assert_eq!(saved_contact.message, "Feature request");
        assert_eq!(saved_contact.data, Some(data));
    }

    #[tokio::test]
    async fn test_create_message_invalid_category() {
        let (service, _mock_repo) = create_service();

        let result = service
            .create_message(
                "INVALID_CATEGORY".to_string(),
                "test@example.com".to_string(),
                "John Doe".to_string(),
                "Test message".to_string(),
                None,
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationError(_) => {}
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_create_message_database_error() {
        let (service, mock_repo) = create_service();

        mock_repo.set_save_should_fail(true);

        let result = service
            .create_message(
                "OTHER".to_string(),
                "test@example.com".to_string(),
                "John Doe".to_string(),
                "Test message".to_string(),
                None,
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::DatabaseError(_) => {}
            _ => panic!("Expected DatabaseError"),
        }

        let saved_contact_messages = mock_repo.get_all_contact_messages();
        assert_eq!(saved_contact_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_all_valid_categories() {
        let (service, mock_repo) = create_service();

        let categories = vec!["ERROR", "IDEA", "TESTIMONIAL", "OTHER"];

        for (i, category) in categories.iter().enumerate() {
            let result = service
                .create_message(
                    category.to_string(),
                    format!("test{}@example.com", i),
                    format!("User {}", i),
                    format!("This is test message number {}", i),
                    None,
                )
                .await;

            assert!(result.is_ok(), "Failed for category: {}", category);
        }

        let saved_contact_messages = mock_repo.get_all_contact_messages();
        assert_eq!(saved_contact_messages.len(), 4);

        assert_eq!(
            saved_contact_messages[0].category,
            ContactMessageCategory::ERROR
        );
        assert_eq!(
            saved_contact_messages[1].category,
            ContactMessageCategory::IDEA
        );
        assert_eq!(
            saved_contact_messages[2].category,
            ContactMessageCategory::TESTIMONIAL
        );
        assert_eq!(
            saved_contact_messages[3].category,
            ContactMessageCategory::OTHER
        );
    }

    #[tokio::test]
    async fn test_case_insensitive_categories() {
        let (service, mock_repo) = create_service();

        let result = service
            .create_message(
                "error".to_string(),
                "test@example.com".to_string(),
                "John Doe".to_string(),
                "Test message".to_string(),
                None,
            )
            .await;

        assert!(result.is_ok());

        let saved_contact_messages = mock_repo.get_all_contact_messages();
        assert_eq!(saved_contact_messages.len(), 1);
        assert_eq!(
            saved_contact_messages[0].category,
            ContactMessageCategory::ERROR
        );
    }
}
