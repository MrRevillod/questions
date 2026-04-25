use serde::Deserialize;
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use std::{path::Path, sync::Arc, time::Duration};
use sword::prelude::*;

use crate::shared::AppResult;

pub type Tx<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

#[injectable(provider)]
pub struct Database {
    pool: Arc<PgPool>,
}

#[config(key = "postgres-db")]
#[derive(Clone, Deserialize)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub database: String,
    pub port: String,
    pub host: String,
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
            .connect(&Self::create_uri(&db_conf))
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

    fn create_uri(db_conf: &DatabaseConfig) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            db_conf.user, db_conf.password, db_conf.host, db_conf.port, db_conf.database
        )
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn tx(&self) -> AppResult<Tx<'_>> {
        Ok(self.pool.begin().await?)
    }
}

#[injectable]
pub struct TransactionManager {
    db: Arc<Database>,
}

impl TransactionManager {
    pub async fn begin(&self) -> AppResult<Tx<'_>> {
        self.db.tx().await
    }
}
