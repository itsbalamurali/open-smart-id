pub mod entities;
pub mod migrations;

use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    migrations::Migrator::up(&db, None).await?;
    Ok(db)
}
