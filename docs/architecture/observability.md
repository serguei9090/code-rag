# Observability Architecture

## Overview

This document describes the internal design of the telemetry and observability system in `code-rag`.

## System Components

```mermaid
graph TB
    A[Application Code] --> B[Telemetry Module]
    B --> C{Mode?}
    C -->|CLI| D[Chrome Tracing]
    C -->|Server| E[OTLP Exporter]
    C -->|Server| F[Prometheus Exporter]
    E --> G[Jaeger]
    F --> H[Prometheus Registry]
    H --> I[/metrics Endpoint]
    
    J[System Info] --> K[Memory Gauge]
    K --> H
```

## Telemetry Initialization Flow

### Entry Point

**File**: [src/main.rs](file:///i:/01-Master_Code/Test-Labs/code-rag/src/main.rs)

The telemetry system is initialized early in `main()`:

```rust
let _guard = init_telemetry(app_mode, &config)
    .context("Failed to initialize telemetry")?;
```

**Key Points**:
- Guard pattern ensures cleanup on drop
- Mode detection (`Cli` vs `Server`) happens before initialization
- Config determines if telemetry is enabled

---

### Mode Detection

**File**: [src/telemetry.rs:41-44](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L41-L44)

```rust
match mode {
    AppMode::Cli => init_cli_telemetry(config),
    AppMode::Server => init_server_telemetry(&config.telemetry_endpoint),
}
```

Two distinct initialization paths based on application mode.

---

## CLI Mode Telemetry

**File**: [src/telemetry.rs:47-74](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L47-L74)

### Chrome Tracing

Uses `tracing-chrome` to generate `trace-*.json` files compatible with `chrome://tracing`.

**Benefits**:
- Low overhead
- No external dependencies
- Detailed flamegraphs for performance analysis

**Output Location**: `trace-<timestamp>.json` in working directory

### MCP Consideration

When `config.enable_mcp = true`, logs are redirected to stderr to avoid corrupting stdout JSON-RPC stream:

```rust
if config.enable_mcp {
    // Redirect logs to stderr only
    let registry = Registry::default()
        .with(chrome_layer)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr));
    registry.try_init();
}
```

---

## Server Mode Telemetry

**File**: [src/telemetry.rs:77-133](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L77-L133)

### 1. OTLP Tracer (Jaeger Integration)

**Purpose**: Export distributed traces to Jaeger for request tracing

**Implementation**:
```rust
let tracer = opentelemetry_otlp::new_pipeline()
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
```

**Key Characteristics**:
- Uses gRPC transport via Tonic
- Batched export for performance
- Service name: `code-rag-server`
- Async export using Tokio runtime

---

### 2. Prometheus Exporter

**Purpose**: Expose metrics for Prometheus scraping

**Implementation**:
```rust
let registry = prometheus::default_registry();
let exporter = opentelemetry_prometheus::exporter()
    .with_registry(registry.clone())
    .build()?;

let provider = SdkMeterProvider::builder()
    .with_reader(exporter)
    .build();
global::set_meter_provider(provider.clone());
```

**Architecture**:
- Uses default Prometheus registry
- OpenTelemetry provides abstraction layer
- Metrics are pushed to registry for scraping

---

### 3. Memory Gauge

**Purpose**: Track application memory usage over time

**Implementation**: [src/telemetry.rs:105-121](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L105-L121)

```rust
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
```

**Design**:
- **Observable Gauge**: Updated via callback, not manually pushed
- **Callback-based**: Prometheus pulls trigger the callback
- **System Info Crate**: Uses `sysinfo` for cross-platform memory reading
- **Arc<Mutex<System>>**: Thread-safe access to system info
- **No Labels**: Currently tracks total process memory only

**Update Frequency**: On each Prometheus scrape (default: 5s)

---

## Tracing Subscriber Stack

**File**: [src/telemetry.rs:124-129](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L124-L129)

```rust
let subscriber = Registry::default()
    .with(telemetry)          // OpenTelemetry layer for Jaeger
    .with(tracing_subscriber::fmt::layer());  // Console logging

subscriber.try_init();
```

**Layers**:
1. **OpenTelemetry Layer**: Exports spans to Jaeger
2. **fmt Layer**: Prints logs to stdout/stderr

**Benefits**:
- Multiple subscribers can consume same events
- Decoupled concerns (logging vs tracing vs metrics)

---

## Resource Cleanup

**File**: [src/telemetry.rs:17-19](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L17-L19)

```rust
pub struct TelemetryGuard {
    _chrome_guard: Option<tracing_chrome::FlushGuard>,
}
```

The `TelemetryGuard` holds the Chrome tracing flush guard. When dropped:
- Chrome trace file is flushed and finalized
- OpenTelemetry traces are flushed to Jaeger
- Clean shutdown of exporters

**Usage**: Stored in `_guard` variable in `main()`, dropped at program exit.

---

## Performance Considerations

### Overhead

- **Disabled Telemetry**: Zero runtime cost (compile-time opt-out)
- **Chrome Tracing**: Minimal overhead (<1% in most workloads)
- **OTLP Export**: Async + batched, negligible impact on hot path
- **Prometheus Metrics**: Callback-based, only triggered on scrape

### Memory Usage

The memory gauge itself uses:
- `Arc<Mutex<System>>`: ~KB of overhead
- Callback registration: Negligible
- **Does not allocate per scrape**

---

## Future Enhancements

### Planned Metrics

1. **Search Performance**
   - `search_duration_seconds{workspace="name"}` - Histogram
   - `search_requests_total{workspace="name",status="success|error"}` - Counter

2. **Indexing**
   - `indexed_files_total{workspace="name"}` - Counter
   - `index_duration_seconds{workspace="name"}` - Histogram

3. **Storage**
   - `database_size_bytes{workspace="name"}` - Gauge
   - `chunk_count{workspace="name"}` - Gauge

### Missing API Endpoint

**Current Gap**: The `/metrics` endpoint is not exposed in `server.rs` router.

**Proposed Solution**: Add Prometheus HTTP handler to serve metrics:

```rust
async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    
    (StatusCode::OK, buffer).into_response()
}
```

Add route: `.route("/metrics", get(metrics_handler))`

---

## Dependencies

- `opentelemetry` - Core abstractions
- `opentelemetry_otlp` - OTLP exporter for Jaeger
- `opentelemetry_prometheus` - Prometheus exporter
- `opentelemetry_sdk` - SDK implementation
- `tracing-opentelemetry` - Bridge between `tracing` and OpenTelemetry
- `tracing-chrome` - Chrome trace format exporter
- `sysinfo` - Cross-platform system information
- `prometheus` - Prometheus client library

---

## Related Documentation

- [Telemetry Features Guide](../features/telemetry.md)
- [Configuration Reference](../configuration/telemetry_config.md)
- [Troubleshooting](../troubleshooting/prometheus_metrics.md)
