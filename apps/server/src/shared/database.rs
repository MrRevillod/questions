use serde::Deserialize;
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use std::{path::Path, sync::Arc, time::Duration};
use sword::prelude::*;

#[injectable(provider)]
pub struct Database {
    pool: Arc<PgPool>,
}

#[config(key = "postgres-db")]
#[derive(Clone, Deserialize)]
pub struct DatabaseConfig {
    pub uri: String,
    pub migrations_path: String,
    pub min_connections: u8,
    pub max_connections: u8,
    pub acquire_timeout_ms: u64,
}

impl Database {
    pub async fn new(db_conf: DatabaseConfig) -> Self {
        let pool = PgPoolOptions::new()
            .min_connections(db_conf.min_connections.into())
            .max_connections(db_conf.max_connections.into())
            .acquire_timeout(Duration::from_millis(db_conf.acquire_timeout_ms))
            .connect(&db_conf.uri)
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to connect to PostgreSQL database: {}", err);
            })
            .expect("Failed to create database connection pool");

        let migrator = Migrator::new(Path::new(&db_conf.migrations_path))
            .await
            .expect("Failed to initialize migrator");

        migrator
            .run(&pool)
            .await
            .expect("Failed to run database migrations");

        Self {
            pool: Arc::new(pool),
        }
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}
