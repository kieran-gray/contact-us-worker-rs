#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    NotFound(String),
    Unauthorised(String),
    InternalError,
    ValidationError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            AppError::NotFound(msg) => write!(f, "Not found: {msg}"),
            AppError::Unauthorised(msg) => write!(f, "Unauthorised: {msg}"),
            AppError::InternalError => write!(f, "Internal server error"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}
