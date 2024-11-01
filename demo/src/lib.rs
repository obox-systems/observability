mod client;
mod server;

pub use client::*;
pub use server::*;

use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::Resource;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_tracing() -> anyhow::Result<()> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTEL_URL")
        .ok()
        .unwrap_or("http://localhost:4317".to_owned());

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&endpoint),
        )
        .with_trace_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "rust-basic-app",
            )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let tracing_telemetry = tracing_opentelemetry::layer().with_tracer(tracer.tracer("opentelemetry"));

    let logging = opentelemetry_otlp::new_pipeline().logging().with_exporter(opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&endpoint)).with_resource(Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "rust-basic-app",
    )])).install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let logging_telemetry = OpenTelemetryTracingBridge::new(&logging);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_telemetry)
        .with(logging_telemetry)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    Ok(())
}
