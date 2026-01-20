#[derive(Debug)]
pub enum AppError {
    Database(String),
    InvalidCredentials,
    InternalError(String),
    ValidationError(String),
    NotFound(String),
    Conflict(String),
}