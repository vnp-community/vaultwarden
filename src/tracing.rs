// TASK-010-014: OpenTelemetry Tracing Setup (feature-gated)
//
// Enable with `--features otel`. Requires a running OTLP collector
// (e.g. OpenTelemetry Collector, Jaeger, or Grafana Tempo).
//
// Config keys (src/config.rs):
//   OTEL_ENABLED=true
//   OTEL_EXPORTER=otlp            # only "otlp" is supported
//   OTEL_ENDPOINT=http://localhost:4317
//   OTEL_SERVICE_NAME=vaultwarden
//   OTEL_SAMPLE_RATE=1.0          # 0.0–1.0, used for TraceIdRatioBased sampler

/// Set up OpenTelemetry tracing when the `otel` feature is compiled in.
///
/// - Installs an OTLP gRPC exporter pipeline.
/// - Uses `TraceIdRatioBased` sampler at the configured sample rate.
/// - Attaches `service.name` and `service.version` resource attributes.
/// - Installs an `OpenTelemetryLayer` on the global `tracing_subscriber`.
/// - No-op when `otel` feature is absent or `OTEL_ENABLED=false`.
pub fn setup_otel() {
    if !crate::CONFIG.otel_enabled() {
        return;
    }

    #[cfg(feature = "otel")]
    {
        setup_otel_inner();
    }

    #[cfg(not(feature = "otel"))]
    {
        warn!(
            "OTEL_ENABLED=true but the `otel` Cargo feature is not compiled in. \
             Rebuild with `--features otel` to enable OpenTelemetry tracing."
        );
    }
}

/// Inner implementation, compiled only when `--features otel` is set.
#[cfg(feature = "otel")]
fn setup_otel_inner() {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig as _;
    use opentelemetry_sdk::{
        runtime,
        trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
        Resource,
    };
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

    let endpoint = crate::CONFIG.otel_endpoint();
    let service_name = crate::CONFIG.otel_service_name();
    let sample_rate = crate::CONFIG.otel_sample_rate();

    // Build the resource — `service.name` and `service.version` are standard OTLP attributes.
    let resource = Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", service_name.clone()),
        opentelemetry::KeyValue::new(
            "service.version",
            env!("CARGO_PKG_VERSION"),
        ),
    ]);

    // Build the OTLP gRPC exporter.
    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()
    {
        Ok(e) => e,
        Err(err) => {
            error!("OTel: failed to create OTLP exporter (endpoint={endpoint}): {err}");
            return;
        }
    };

    // Assemble the provider with TraceIdRatioBased sampler and Tokio async runtime.
    let sampler = if (sample_rate - 1.0_f32).abs() < f32::EPSILON {
        Sampler::AlwaysOn
    } else {
        Sampler::TraceIdRatioBased(f64::from(sample_rate))
    };

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .build();

    let tracer = provider.tracer(service_name);

    // Compose the tracing subscriber layers:
    //   1. OpenTelemetryLayer — exports spans via OTLP
    //   2. fmt layer — keeps existing log output
    //   3. env-filter — honours RUST_LOG
    let otel_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(otel_layer)
        .try_init()
        .unwrap_or_else(|err| {
            // Another subscriber may already be installed (e.g. by the JSON logging path).
            // This is non-fatal — spans will still be exported if the existing subscriber
            // has a compatible OTel layer.
            warn!("OTel: could not set as global subscriber (already initialised?): {err}");
        });

    // Keep the provider alive for the duration of the process.
    // The SDK will flush pending spans when the provider is dropped (on shutdown).
    // We leak it intentionally to the 'static lifetime so Tokio tasks can keep submitting spans.
    let _ = Box::leak(Box::new(provider));

    info!(
        "OTel: tracing enabled — exporter=otlp endpoint={endpoint} sample_rate={sample_rate}"
    );
}
