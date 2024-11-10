mod client;
mod server;

pub use client::*;
pub use server::*;
use std::sync::OnceLock;

use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::reader::DefaultTemporalitySelector;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
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

    let tracing_telemetry =
        tracing_opentelemetry::layer().with_tracer(tracer.tracer("opentelemetry"));

    let logging = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&endpoint),
        )
        .with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "rust-basic-app",
        )]))
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let logging_telemetry = OpenTelemetryTracingBridge::new(&logging);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_telemetry)
        .with(logging_telemetry)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Ok(())
}

pub fn metering_provider() -> &'static SdkMeterProvider {
    static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();

    METER_PROVIDER.get_or_init(|| {
        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(
                std::env::var("OTEL_EXPORTER_OTEL_URL")
                    .ok()
                    .unwrap_or("http://localhost:4317".to_owned()),
            )
            .build_metrics_exporter(Box::new(DefaultTemporalitySelector::new()))
            .expect("failed to build metrics exporter");
        let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();
        let provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_resource(Resource::new([KeyValue::new(
                "service.name",
                "metrics-basic-example",
            )]))
            .build();
        opentelemetry::global::set_meter_provider(provider.clone());
        provider
    })
}
