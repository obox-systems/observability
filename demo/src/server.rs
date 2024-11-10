use std::time::Duration;

use crate::metering_provider;
use axum::{http::StatusCode, routing::get, Router};
use opentelemetry::metrics::MeterProvider;
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

    let meter_provider = metering_provider();

    let meter = meter_provider.meter("demo-app");
    // Create a Counter Instrument.
    let counter = meter.u64_counter("request").init();

    match rand::random::<u8>() {
        number @ 0..50 => {
            tracing::info!("Handler is OK. number: {number}");

            counter.add(1, &[opentelemetry::KeyValue::new("status", "ok")]);

            Ok(())
        }
        number @ 50..100 => {
            tracing::info!("Handler is OK. number: {number}");

            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            tracing::event!( name: "seconds from UNIX_EPOCH", Level::INFO, time = %time);

            counter.add(1, &[opentelemetry::KeyValue::new("status", "ok")]);

            Ok(())
        }
        number @ 100..150 => {
            tracing::warn!("Handler is WARN, but STATUS CODE - OK. number: {number}");

            counter.add(1, &[opentelemetry::KeyValue::new("status", "warn")]);

            Ok(())
        }
        number @ 150..200 => {
            tracing::warn!("Handler is WARN. number: {number}");

            counter.add(1, &[opentelemetry::KeyValue::new("status", "warn")]);

            Err(StatusCode::FORBIDDEN)
        }
        number => {
            tracing::error!("Handler is ERR. number: {number}");

            tokio::time::sleep(Duration::from_secs(1)).await;

            tracing::event!(Level::INFO, "Slept 1s");

            counter.add(1, &[opentelemetry::KeyValue::new("status", "err")]);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
