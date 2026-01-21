# Prometheus Metrics API Documentation

## Overview

Code-RAG exposes Prometheus metrics when running in **server mode** with **telemetry enabled**. The metrics are automatically collected and exposed via the OpenTelemetry Prometheus exporter.

## Configuration

### Files Involved

1. **[src/telemetry.rs](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs)** - Telemetry initialization and Prometheus exporter setup
2. **[docker-compose.telemetry.yml](file:///i:/01-Master_Code/Test-Labs/code-rag/docker-compose.telemetry.yml)** - Docker services for observability stack
3. **[prometheus.yml](file:///i:/01-Master_Code/Test-Labs/code-rag/prometheus.yml)** - Prometheus scrape configuration

### Enable Telemetry

In your `code-rag.toml`:

```toml
# Enable OpenTelemetry tracing and metrics
telemetry_enabled = true

# OTLP endpoint for traces (Jaeger)
telemetry_endpoint = "http://localhost:4317"
```

## Metrics Endpoint

### **GET /metrics**

**Currently Not Directly Exposed** - The metrics are registered with the Prometheus default registry and need to be scraped.

> **Note**: The current implementation initializes the Prometheus exporter but doesn't expose a dedicated `/metrics` HTTP endpoint in the server router. The metrics are available through the Prometheus registry but need to be exposed via an endpoint.

### Available Metrics

#### **app_memory_usage_bytes**

- **Type**: Gauge (Observable)
- **Description**: Current RAM usage of the application process in bytes
- **Unit**: bytes
- **Labels**: None
- **Update Frequency**: On each observation callback (dynamic)

**Implementation Location**: [src/telemetry.rs:106-121](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L106-L121)

```rust
let memory_gauge = meter
    .u64_observable_gauge("app_memory_usage_bytes")
    .with_description("Current RAM usage of the application process")
    .init();
```

## Observability Stack

### Docker Compose Services

Start the full observability stack:

```bash
docker-compose -f docker-compose.telemetry.yml up -d
```

This starts:

#### **Jaeger** (Distributed Tracing)
- **Web UI**: http://localhost:16686
- **OTLP gRPC**: Port 4317
- **OTLP HTTP**: Port 4318

#### **Prometheus** (Metrics)
- **Web UI**: http://localhost:9090
- **Scrape Target**: `host.docker.internal:3000` (code-rag server)
- **Scrape Interval**: 5 seconds

#### **Grafana** (Visualization)
- **Web UI**: http://localhost:3000
- **Default Credentials**: admin/admin

## Prometheus Scrape Configuration

**File**: `prometheus.yml`

```yaml
global:
  scrape_interval: 5s

scrape_configs:
  - job_name: 'code-rag-server'
    static_configs:
      - targets: ['host.docker.internal:3000']
```

## Implementation Details

### Telemetry Initialization Flow

1. **Server Mode Detection** ([src/telemetry.rs:41-44](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L41-L44))
   - CLI mode uses Chrome tracing
   - **Server mode** uses OTLP + Prometheus

2. **Prometheus Exporter Setup** ([src/telemetry.rs:97-103](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L97-L103))
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

3. **Memory Gauge Registration** ([src/telemetry.rs:105-121](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L105-L121))
   - Uses `sysinfo` crate to read process memory
   - Registered as observable gauge with callback
   - Automatically updates on scrape

## Adding the /metrics Endpoint

### Current Gap

The server router in [src/server.rs](file:///i:/01-Master_Code/Test-Labs/code-rag/src/server.rs) does not currently expose a `/metrics` endpoint. 

### Recommended Implementation

To expose Prometheus metrics, add this handler to `src/server.rs`:

```rust
use prometheus::{Encoder, TextEncoder};

/// Prometheus metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {}", e)
        ).into_response();
    }
    
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        buffer
    ).into_response()
}
```

Add route to router ([src/server.rs:124-138](file:///i:/01-Master_Code/Test-Labs/code-rag/src/server.rs#L124-L138)):

```rust
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(status_handler))
        .route("/metrics", get(metrics_handler))  // ← Add this
        .route("/search", post(search_handler_default))
        .route("/v1/{workspace}/search", post(search_handler_workspace))
        // ... rest of config
}
```

## Testing Metrics

### 1. Start Code-RAG Server with Telemetry

```powershell
# Using the test config
code-rag start --config code-rag.test.toml
```

### 2. Start Observability Stack

```bash
docker-compose -f docker-compose.telemetry.yml up -d
```

### 3. Access Prometheus UI

Navigate to: http://localhost:9090

**Query Memory Usage**:
```promql
app_memory_usage_bytes
```

### 4. Verify Scrape Targets

Go to: http://localhost:9090/targets

You should see `code-rag-server` in the targets list.

## Future Metrics

Consider adding these metrics for better observability:

### Search Performance
- `search_duration_seconds{workspace="name"}` - Histogram
- `search_requests_total{workspace="name",status="success|error"}` - Counter
- `active_searches` - Gauge

### Indexing
- `indexed_files_total{workspace="name"}` - Counter
- `index_duration_seconds{workspace="name"}` - Histogram
- `index_size_bytes{workspace="name"}` - Gauge

### Storage
- `database_size_bytes{workspace="name"}` - Gauge
- `chunk_count{workspace="name"}` - Gauge

### Embeddings
- `embedding_generation_duration_seconds` - Histogram
- `embedding_batch_size` - Histogram

## References

- **Prometheus Documentation**: https://prometheus.io/docs/
- **OpenTelemetry Rust**: https://github.com/open-telemetry/opentelemetry-rust
- **Code-RAG Telemetry**: [src/telemetry.rs](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs)
- **Server Configuration**: [src/server.rs](file:///i:/01-Master_Code/Test-Labs/code-rag/src/server.rs)
