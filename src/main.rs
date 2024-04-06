use axum::{routing::post, serve, Router};
use dotenvy_macro::dotenv;
use log::{info, LevelFilter};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};
use std::{env::args, sync::Arc, time::Duration};
use tokio::{main, net::TcpListener};

mod error;
use error::*;

mod options;

mod geocode;
use geocode::*;

#[main]
async fn main() -> Result<(), AppError> {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()?;

    info!("Connecting to postgres");

    let options = PgConnectOptions::new()
        .username(dotenv!("PGUSER"))
        .password(dotenv!("PGPASSWORD"))
        .database(dotenv!("PGDATABASE"))
        .log_slow_statements(LevelFilter::Info, Duration::from_secs(5));

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
