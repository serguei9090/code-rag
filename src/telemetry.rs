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
    // Always apply log level from config, even if RUST_LOG is set
    // This ensures consistent behavior regardless of environment
    let filter = format!(
        "code_rag={},tokenizers=error,tantivy=warn,h2=error,tower=error,hyper=warn,reqwest=warn",
        config.log_level
    );
    std::env::set_var("RUST_LOG", filter);

    if !config.telemetry_enabled {
        // Initialize basic logging with EnvFilter
        let env_filter = tracing_subscriber::EnvFilter::from_default_env();

        // For MCP, we MUST NOT print logs to stdout as it corrupts the JSON-RPC stream
        if config.enable_mcp {
            // Redirect logs to stderr only
            let subscriber = Registry::default()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr));
            let _ = subscriber.try_init();
        } else {
            // Normal CLI/Server mode - stdout is fine
            let subscriber = Registry::default()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer());
            let _ = subscriber.try_init();
        }

        return Ok(TelemetryGuard {
            _chrome_guard: None,
        });
    }

    match mode {
        AppMode::Cli => init_cli_telemetry(config),
        AppMode::Server => init_server_telemetry(&config.telemetry_endpoint, config),
    }
}

fn init_cli_telemetry(config: &AppConfig) -> Result<TelemetryGuard> {
    let (chrome_layer, guard) = ChromeLayerBuilder::new().build();

    // Explicitly build the filter to ensure specific crate levels are respected
    // even in CLI mode (e.g., suppressing tokenizers trace logs)
    let filter_str = format!(
        "code_rag={},tokenizers=error,tantivy=warn,h2=error,tower=error,hyper=warn,reqwest=warn",
        config.log_level
    );
    let filter_layer = tracing_subscriber::EnvFilter::try_new(&filter_str)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    // Redirect 'log' events to 'tracing' to capture dependencies using the log crate
    let _ = tracing_log::LogTracer::init();

    // We must use specific types or branch entirely to avoid type mismatch
    if config.enable_mcp {
        // MCP Mode: Chrome Layer + Stderr Logging + Filter
        let registry = Registry::default()
            .with(filter_layer)
            .with(chrome_layer)
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr));
        let _ = registry.try_init();
    } else {
        // Normal Mode: Chrome Layer + Default (Stdout) Logging + Filter
        let registry = Registry::default()
            .with(filter_layer)
            .with(chrome_layer)
            .with(tracing_subscriber::fmt::layer());
        let _ = registry.try_init();
    }

    Ok(TelemetryGuard {
        _chrome_guard: Some(guard),
    })
}

fn init_server_telemetry(endpoint: &str, _config: &AppConfig) -> Result<TelemetryGuard> {
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
    // Explicitly build the filter to ensure it's applied correctly
    let filter_str = format!(
        "code_rag={},tokenizers=error,tantivy=warn,h2=error,tower=error,hyper=warn,reqwest=warn",
        _config.log_level
    );
    let filter_layer = tracing_subscriber::EnvFilter::try_new(&filter_str)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    // Redirect 'log' events to 'tracing'
    let _ = tracing_log::LogTracer::init();

    let subscriber = Registry::default()
        .with(filter_layer)
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer());

    // Ignore error if already set
    let _ = subscriber.try_init();

    Ok(TelemetryGuard {
        _chrome_guard: None,
    })
}
