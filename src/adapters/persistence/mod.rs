use sqlx::pool;

use crate::app_error::AppError

#[derive(Clone)]
pub struct PostgresPresistence {
    pool: PgPool,
}

impl PostgresPresistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::Database(error.to_string())
    }
}