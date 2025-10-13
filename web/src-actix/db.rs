use anyhow::Result;
use log::LevelFilter;
use sqlx::{
    ConnectOptions,
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions},
};
use std::str::FromStr;

pub async fn init_database() -> Result<SqlitePool> {
    let db_url = "sqlite://minipx.db";

    let connect_options = SqliteConnectOptions::from_str(db_url)?.create_if_missing(true).log_statements(LevelFilter::Debug);

    let pool = SqlitePoolOptions::new().max_connections(5).connect_with(connect_options).await?;

    // Run migrations
    sqlx::query(include_str!("../migrations/001_initial_schema.sql")).execute(&pool).await?;

    Ok(pool)
}
