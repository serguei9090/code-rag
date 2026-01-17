use anyhow::Result;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{metrics::SdkMeterProvider, runtime, trace as sdktrace, Resource};
use std::sync::{Arc, Mutex};
use sysinfo::{Pid, System};
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

use crate::config::AppConfig;

pub enum AppMode {
    Cli,
    Server,
}

pub struct TelemetryGuard {
    _chrome_guard: Option<tracing_chrome::FlushGuard>,
}

pub fn init_telemetry(mode: AppMode, config: &AppConfig) -> Result<TelemetryGuard> {
    if !config.telemetry_enabled {
        // Initialize basic logging/subscriber if needed, or just return.
        // If we don't init subscriber, regular logs might not show up if we relied on this.
        // Assuming we still want LOGS to stdout?
        // The original code replaced `init_logging` with `init_telemetry`.
        // If disabled, we should probably still enable basic fmt logging.

        // Let's setup a basic fmt subscriber for logs if telemetry is disabled.
        let subscriber = Registry::default().with(tracing_subscriber::fmt::layer());
        let _ = subscriber.try_init();

        return Ok(TelemetryGuard {
            _chrome_guard: None,
        });
    }

    match mode {
        AppMode::Cli => init_cli_telemetry(),
        AppMode::Server => init_server_telemetry(&config.telemetry_endpoint),
    }
}

fn init_cli_telemetry() -> Result<TelemetryGuard> {
    let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
    let registry = Registry::default().with(chrome_layer);

    // Attempt to init. Ignore error if already globally set (for idempotency in tests/dev)
    let _ = registry.try_init();

    Ok(TelemetryGuard {
        _chrome_guard: Some(guard),
    })
}

fn init_server_telemetry(endpoint: &str) -> Result<TelemetryGuard> {
    // 1. OTLP Tracer (Jaeger)
    // install_batch returns a Tracer. The global provider is configured implicitly by install_batch in many versions,
    // or we just use the tracer with the layer.
    let tracer =
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "code-rag-server"),
            ])))
            .install_batch(runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // 2. Monitoring (System Gauge)
    let registry = prometheus::default_registry();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()?;

    let provider = SdkMeterProvider::builder().with_reader(exporter).build();
    global::set_meter_provider(provider.clone());

    let meter = global::meter("code-rag-system");
    let memory_gauge = meter
        .u64_observable_gauge("app_memory_usage_bytes")
        .with_description("Current RAM usage of the application process")
        .init();

    let sys = Arc::new(Mutex::new(System::new_all()));
    let pid = Pid::from_u32(std::process::id());

    meter.register_callback(&[memory_gauge.as_any()], move |observer| {
        if let Ok(mut sys) = sys.lock() {
            sys.refresh_process(pid);
            if let Some(process) = sys.process(pid) {
                observer.observe_u64(&memory_gauge, process.memory(), &[]);
            }
        }
    })?;

    // Subscriber setup
    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer());

    // Ignore error if already set
    let _ = subscriber.try_init();

    Ok(TelemetryGuard {
        _chrome_guard: None,
    })
}
