use std::time::Duration;

use axum::{http::StatusCode, routing::get, Router};
use tracing::Level;

pub async fn run_sample_server() -> anyhow::Result<()> {
    let app = Router::new().route("/", get(handler));

    tracing::info!("running server at localhost:5000");
    axum::serve(tokio::net::TcpListener::bind("0.0.0.0:5000").await?, app).await?;

    Ok(())
}

#[tracing::instrument]
async fn handler() -> Result<(), StatusCode> {
    tokio::time::sleep(Duration::from_millis(
        (rand::random::<u8>() as u32 * 4).into(),
    ))
    .await;

    match rand::random::<u8>() {
        number @ 0..50 => {
            tracing::info!("Handler is OK. number: {number}");

            Ok(())
        }
        number @ 50..100 => {
            tracing::info!("Handler is OK. number: {number}");

            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            tracing::event!( name: "seconds from UNIX_EPOCH", Level::INFO, time = %time);

            Ok(())
        }
        number @ 100..150 => {
            tracing::warn!("Handler is WARN, but STATUS CODE - OK. number: {number}");

            Ok(())
        }
        number @ 150..200 => {
            tracing::warn!("Handler is WARN. number: {number}");

            Err(StatusCode::FORBIDDEN)
        }
        number => {
            tracing::error!("Handler is ERR. number: {number}");

            tokio::time::sleep(Duration::from_secs(1)).await;

            tracing::event!(Level::INFO, "Slept 1s");

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
