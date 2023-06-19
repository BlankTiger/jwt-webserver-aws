use crate::db_actions;
use crate::models::Keys;
use color_eyre::Result;
use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::env;
use tracing::info;
use tracing_subscriber::EnvFilter;

pub static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

pub static PEPPER: Lazy<String> =
    Lazy::new(|| std::env::var("PEPPER").expect("PEPPER must be set"));

pub async fn setup() -> Result<()> {
    dotenv()?;
    setup_error_handling()?;
    setup_tracing();
    info!("Environment variables should be accessible now!");
    setup_database().await?;

    Ok(())
}

fn setup_error_handling() -> Result<()> {
    if env::var("RUST_LIB_BACKTRACE").is_err() {
        env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    color_eyre::install()?;
    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

async fn setup_database() -> Result<()> {
    let pool = db_actions::get_pool().await?;
    sqlx::migrate!().run(&pool).await?;

    Ok(())
}
