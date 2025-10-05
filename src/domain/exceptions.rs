#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidEmail(String),
    InvalidName(String),
    InvalidMessage(String),
    InvalidData(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidEmail(msg) => write!(f, "Invalid email: {msg}"),
            ValidationError::InvalidName(msg) => write!(f, "Invalid name: {msg}"),
            ValidationError::InvalidMessage(msg) => write!(f, "Invalid message: {msg}"),
            ValidationError::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone)]
pub enum RepositoryError {
    SaveFailed(String),
    NotFound(String),
    DatabaseError(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::SaveFailed(msg) => write!(f, "Save failed: {msg}"),
            RepositoryError::NotFound(msg) => write!(f, "Not found: {msg}"),
            RepositoryError::DatabaseError(msg) => write!(f, "Database error: {msg}"),
        }
    }
}

impl std::error::Error for RepositoryError {}
