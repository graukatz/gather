use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::fs;

const DB_URL: &str = "data";

pub async fn ensure_database() -> SqlitePool {
    if let Err(e) = fs::create_dir_all(DB_URL).await {
        tracing::error!("Failed creating data directory: {e}");
        panic!("Failed creating data directory")
    } else {
        tracing::info!("Created data directory")
    }

    let database_url = "sqlite:data/gather.db";

    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        tracing::info!("Creating database {}", database_url);

        if let Err(e) = Sqlite::create_database(database_url).await {
            tracing::error!("Failed to create database: {e}");
            panic!("Failed to create database");
        }
    }

    let pool = match SqlitePoolOptions::new().connect(database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to connect to database: {e}");
            panic!("Failed to connect to database");
        }
    };

    tracing::info!("Database connected");

    if let Err(e) = sqlx::migrate!().run(&pool).await {
        tracing::error!("Failed running migrations: {e}");
        panic!("Failed running migrations");
    }

    tracing::info!("Migrations completed");

    pool
}