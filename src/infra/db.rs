use std::env;

use sqlx::{pgPool, postgres::PgPoolOptions};

use tracing::info;

pub asyn fn init_db()-> anyhow::Result<PgPool> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;

    info!("Connected to database");

    Ok(pool)
}