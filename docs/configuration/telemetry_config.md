# Telemetry Configuration Reference

## Overview

This document describes all telemetry-related configuration options for `code-rag`.

## Configuration File Settings

### `telemetry_enabled`

**Type**: Boolean  
**Default**: `false`  
**Description**: Master switch for OpenTelemetry instrumentation

**Example**:
```toml
telemetry_enabled = true
```

**Behavior**:
- `true`: Initializes OpenTelemetry with tracing and metrics
- `false`: Disables telemetry, uses basic logging only

**Impact**:
- When enabled in **CLI mode**: Enables Chrome tracing to `trace-*.json` files
- When enabled in **Server mode**: Enables OTLP tracing (Jaeger) and Prometheus metrics

---

### `telemetry_endpoint`

**Type**: String (URL)  
**Default**: `"http://localhost:4317"`  
**Description**: OTLP gRPC endpoint for exporting traces to Jaeger

**Example**:
```toml
telemetry_endpoint = "http://localhost:4317"
```

**Requirements**:
- Must be a valid HTTP/HTTPS URL
- Endpoint must support OTLP gRPC protocol
- Typically points to Jaeger collector with OTLP enabled

**Common Endpoints**:
- Local Jaeger: `http://localhost:4317`
- Remote collector: `http://jaeger.example.com:4317`

---

## Environment Variables

### `CODE_RAG_TELEMETRY_ENABLED`

Override `telemetry_enabled` from environment:

```bash
export CODE_RAG_TELEMETRY_ENABLED=true
code-rag serve
```

### `CODE_RAG_TELEMETRY_ENDPOINT`

Override `telemetry_endpoint` from environment:

```bash
export CODE_RAG_TELEMETRY_ENDPOINT="http://custom-jaeger:4317"
code-rag serve
```

---

## Prometheus Scrape Configuration

### File Location

`prometheus.yml` in project root

### Default Configuration

```yaml
global:
  scrape_interval: 5s

scrape_configs:
  - job_name: 'code-rag-server'
    static_configs:
      - targets: ['host.docker.internal:3000']
```

### Key Settings

| Setting | Value | Description |
|---------|-------|-------------|
| `scrape_interval` | `5s` | How often to scrape metrics |
| `job_name` | `code-rag-server` | Service identifier in Prometheus |
| `targets` | `host.docker.internal:3000` | Server address (from Docker) |

### Custom Scrape Endpoint

If running code-rag on a different port:

```yaml
scrape_configs:
  - job_name: 'code-rag-server'
    static_configs:
      - targets: ['host.docker.internal:8080']  # Custom port
```

---

## Docker Compose Configuration

### File Location

`docker-compose.telemetry.yml` in project root

### Services

#### Jaeger (Tracing)

```yaml
jaeger:
  image: jaegertracing/all-in-one:latest
  environment:
    - COLLECTOR_OTLP_ENABLED=true
  ports:
    - "16686:16686"  # Web UI
    - "4317:4317"    # OTLP gRPC
    - "4318:4318"    # OTLP HTTP
```

**Key Environment Variables**:
- `COLLECTOR_OTLP_ENABLED=true` - Enables OpenTelemetry protocol support

#### Prometheus (Metrics)

```yaml
prometheus:
  image: prom/prometheus:latest
  volumes:
    - ./prometheus.yml:/etc/prometheus/prometheus.yml
  ports:
    - "9090:9090"
  command:
    - --config.file=/etc/prometheus/prometheus.yml
```

**Configuration Loading**:
- Volume mount loads `prometheus.yml` from project root
- Command arg specifies config file path inside container

#### Grafana (Visualization)

```yaml
grafana:
  image: grafana/grafana:latest
  ports:
    - "3000:3000"
  environment:
    - GF_SECURITY_ADMIN_PASSWORD=admin
```

**Default Credentials**: admin / admin

---

## Configuration Precedence

Settings are loaded in this order (later overrides earlier):

1. Built-in defaults
2. `code-rag.toml` file settings
3. Environment variables
4. Command-line flags (if applicable)

---

## MCP Mode Considerations

When `enable_mcp = true`, telemetry adjusts logging behavior:

- **MCP Mode + Telemetry OFF**: Logs redirected to stderr only (stdout reserved for JSON-RPC)
- **MCP Mode + Telemetry ON**: Chrome tracing + stderr logging

**Implementation**: See [src/telemetry.rs:24-34](file:///i:/01-Master_Code/Test-Labs/code-rag/src/telemetry.rs#L24-L34)

---

## Example Configurations

### Development (Telemetry Disabled)

```toml
telemetry_enabled = false
log_level = "debug"
```

### Production (Full Observability)

```toml
telemetry_enabled = true
telemetry_endpoint = "http://jaeger-collector:4317"
log_level = "info"
log_to_file = true
```

### CI/CD (Performance Profiling)

```toml
telemetry_enabled = true  # For Chrome traces
log_level = "warn"
```

---

## Related Documentation

- [Telemetry Features Guide](../features/telemetry.md)
- [Observability Architecture](../architecture/observability.md)
- [Troubleshooting Guide](../troubleshooting/prometheus_metrics.md)
