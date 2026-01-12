#[derive(Debug)]
pub enum AppError {
    Database(String),
    InvalidCredentials,
    InternalError(String),
}

