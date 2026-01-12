use std::env;

use tokio::time::Duration;

#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
}
impl AppConfig {

    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let refresh_token_ttl_days: i64 =env::var("REFRESH_TOKEN_TTL_DAYS").expect("REFRESH_TOKEN_TTL_DAYS must be set").parse().expect("REFRESH_TOKEN_TTL_DAYS must be a valid integer");

        let access_token_ttl_sec: i64 =env::var("ACCESS_TOKEN_TTL_SEC").expect("ACCESS_TOKEN_TTL_SEC must be set").parse().expect("ACCESS_TOKEN_TTL_SEC must be a valid integer");
        Self{
            jwt_secret,
            access_token_ttl: Duration::from_secs(access_token_ttl_sec as u64),
            refresh_token_ttl: Duration::from_secs((refresh_token_ttl_days * 86400) as u64),
        }
    }
}