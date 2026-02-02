use std::sync::Arc;
use uuid::Uuid;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};
use std::env;

use crate::{
    app_error::AppError,
    domain::{
        repository::TeamRepository,
        team::TeamStatus,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLoginResponse {
    pub token: String,
}

pub struct AdminUseCase {
    team_repository: Arc<dyn TeamRepository + Send + Sync>,
}

impl AdminUseCase {
    pub fn new(team_repository: Arc<dyn TeamRepository + Send + Sync>) -> Self {
        Self { team_repository }
    }

    pub async fn login(&self, request: AdminLoginRequest) -> Result<AdminLoginResponse, AppError> {
        let admin_user = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let admin_pass = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());

        if request.username != admin_user || request.password != admin_pass {
            return Err(AppError::InvalidCredentials);
        }

        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: request.username,
            exp: expiration as usize,
        };

        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(AdminLoginResponse { token })
    }

    pub async fn update_team_status(&self, id: Uuid, status: TeamStatus, remarks: Option<String>) -> Result<(), AppError> {
        self.team_repository
            .as_ref()
            .update_team_status(id, status.into(), remarks)
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn delete_team(&self, id: Uuid) -> Result<(), AppError> {
        self.team_repository
            .as_ref()
            .delete_team(id)
            .await
            .map_err(|e| AppError::Database(e))
    }
}
