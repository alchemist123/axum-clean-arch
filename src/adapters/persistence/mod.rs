use sqlx::PgPool;

use crate::adapters::http::app_error_impl::AppError;

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