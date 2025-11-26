pub mod types;
pub mod users;

use sqlx::PgPool;
pub use users::*;

pub async fn init_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    users::init_database(pool).await?;
    Ok(())
}