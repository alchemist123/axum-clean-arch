pub mod team_repository;

use sqlx::PgPool;

use crate::app_error::AppError;

#[derive(Clone)]
pub struct PostgresPresistence {
    #[allow(dead_code)]
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