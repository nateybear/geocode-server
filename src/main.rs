use axum::{routing::post, serve, Router};
use dotenvy_macro::dotenv;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};
use std::{env::args, sync::Arc, time::Duration};
use tokio::{main, net::TcpListener};
use tracing::info;
use tracing_subscriber::{filter::EnvFilter, fmt};

mod error;
use error::*;

mod options;

mod geocode;
use geocode::*;

#[main]
async fn main() -> Result<(), AppError> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .with_thread_ids(true)
        .init();

    info!("Connecting to postgres");

    let options = PgConnectOptions::new()
        .username(dotenv!("PGUSER"))
        .password(dotenv!("PGPASSWORD"))
        .database(dotenv!("PGDATABASE"))
        .log_slow_statements(log::LevelFilter::Info, Duration::from_secs(5));

    info!("Connecting to postgres with options: {:?}", options);

    let pool_size = args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(4);

    info!("Creating a pool of {} connections", pool_size);
    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .acquire_timeout(Duration::from_secs(60 * 60 * 24)) // your batch request will die after 24 hours
        .connect_with(options)
        .await?;

    let app = Router::new()
        .route("/geocode", post(geocode))
        .route("/geocode/batch", post(geocode_batch))
        .with_state(Arc::new(pool));

    info!("Binding to port 3000 and listening");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, app).await.unwrap();

    Ok(())
}
