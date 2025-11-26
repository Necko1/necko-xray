use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub mod postgres;

pub async fn create_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}