mod client;
mod server;

use std::net::{SocketAddr, TcpListener};
use std::str::FromStr;
use std::time::Duration;

pub use client::*;
pub use server::*;

use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::Resource;
use tracing::log::log;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_tracing() -> anyhow::Result<()> {
    let export_endpoint = std::env::var("PROMETHEUS_EXPORT_URL")
        .ok()
        .unwrap_or("0.0.0.0:9000".to_owned());
    
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(SocketAddr::from_str(&export_endpoint)?)
        .install()?;

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(crate::export_config()),
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
                .with_export_config(crate::export_config()),
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

fn export_config() -> ExportConfig {
    let protocol = match std::env::var("OTEL_EXPORT_PROTOCOL")
        .ok()
        .unwrap_or_default()
        .as_str()
    {
        "http/proto" => opentelemetry_otlp::Protocol::HttpBinary,
        "http/json" => opentelemetry_otlp::Protocol::HttpJson,
        "grpc" | _ => opentelemetry_otlp::Protocol::Grpc,
    };

    let endpoint = std::env::var("OTEL_EXPORTER_OTEL_URL")
        .ok()
        .unwrap_or("http://localhost:4317".to_owned());

    log!(
        tracing::log::Level::Info,
        "Using OTEL_EXPORT_PROTOCOL: {:?}",
        protocol
    );
    log!(
        tracing::log::Level::Info,
        "Using OTEL_EXPORTER_OTEL_URL: {endpoint}"
    );

    ExportConfig {
        endpoint,
        timeout: std::time::Duration::from_secs(3),
        protocol,
    }
}

// pub fn metering_provider() -> &'static SdkMeterProvider {
//     static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();
//
//     METER_PROVIDER.get_or_init(|| {
//         let exporter = opentelemetry_otlp::new_exporter()
//             .tonic()
//             .with_export_config(export_config());
//
//         let meter_provider = opentelemetry_otlp::new_pipeline()
//             .metrics(opentelemetry_sdk::runtime::Tokio)
//             .with_exporter(exporter)
//             .with_resource(Resource::new([KeyValue::new(
//                 opentelemetry_semantic_conventions::resource::SERVICE_NAME,
//                 "rust-basic-app",
//             )]))
//             .build()
//             .expect("failed to build meter provider");
//
//         opentelemetry::global::set_meter_provider(meter_provider.clone());
//         meter_provider
//     })
// }
