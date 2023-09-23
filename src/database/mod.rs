use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, DatabaseConnection, DbErr};
use tracing::{debug, log::LevelFilter};

use crate::settings::database;

pub mod entity;

pub async fn init(db_config: &database::Settings) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(db_config.get_url());
    opt.sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    debug!("Connecting to database: {}", db_config.get_url());
    let connection = sea_orm::Database::connect(opt).await?;

    Migrator::up(&connection, None).await?;
    debug!("Database migration completed");

    Ok(connection)
}
