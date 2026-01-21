# Telemetry and Observability

## Overview

`code-rag` provides comprehensive observability through OpenTelemetry integration, enabling you to monitor performance, trace requests, and analyze system metrics in real-time.

## Features

- **Distributed Tracing**: Track requests through the system with Jaeger
- **Metrics Collection**: Monitor memory usage and performance with Prometheus
- **Visualization**: Create custom dashboards with Grafana
- **Zero Cloud Dependencies**: All telemetry data stays local

## Quick Start

### 1. Enable Telemetry

In your `code-rag.toml`:

```toml
# Enable OpenTelemetry tracing and metrics
telemetry_enabled = true

# OTLP endpoint for Jaeger traces
telemetry_endpoint = "http://localhost:4317"
```

### 2. Start Observability Stack

Launch Jaeger, Prometheus, and Grafana using Docker Compose:

```bash
docker-compose -f docker-compose.telemetry.yml up -d
```

This starts three services:
- **Jaeger** (http://localhost:16686) - Distributed tracing UI
- **Prometheus** (http://localhost:9090) - Metrics database
- **Grafana** (http://localhost:3000) - Visualization dashboards

### 3. Run Code-RAG Server

Start the server with telemetry enabled:

```bash
code-rag start --config code-rag.toml
```

Or in server-only mode:

```bash
code-rag serve --config code-rag.toml
```

## Accessing Telemetry Data

### Jaeger (Distributed Tracing)

**URL**: http://localhost:16686

**Use Cases**:
- Track search request latency end-to-end
- Identify bottlenecks in indexing pipeline
- Debug slow queries with detailed span timings
- Analyze concurrent request patterns

**Example**: Select service `code-rag-server` and click "Find Traces" to see all captured requests.

### Prometheus (Metrics)

**URL**: http://localhost:9090

**Available Metrics**:
- `app_memory_usage_bytes` - Current process memory consumption

**Example Queries**:
```promql
# Current memory usage
app_memory_usage_bytes

# Memory usage over time (rate)
rate(app_memory_usage_bytes[5m])
```

**Scrape Configuration**: Prometheus scrapes metrics every 5 seconds from `code-rag` server on port 3000.

### Grafana (Visualization)

**URL**: http://localhost:3000  
**Default Credentials**: admin / admin

**Setup**:
1. Add Prometheus as data source: http://prometheus:9090
2. Import or create dashboards
3. Visualize metrics with graphs, gauges, and alerts

**Recommended Dashboards**:
- Memory usage trends
- Search request rates
- Indexing throughput

## Common Use Cases

### Monitoring Memory Usage

Track memory consumption during large indexing operations:

1. Start telemetry stack
2. Navigate to Prometheus UI
3. Query: `app_memory_usage_bytes`
4. Watch memory grow as files are indexed

### Debugging Slow Searches

Identify why a search is slow using distributed tracing:

1. Open Jaeger UI
2. Select service: `code-rag-server`
3. Find your search request trace
4. Examine span durations to find bottlenecks

### Performance Baseline

Establish performance baselines for regression testing:

1. Run searches with telemetry enabled
2. Collect timing data from Jaeger
3. Export metrics from Prometheus
4. Compare against future builds

## Configuration Reference

See [Telemetry Configuration](../configuration/telemetry_config.md) for detailed settings.

## Architecture

See [Observability Architecture](../architecture/observability.md) for implementation details.

## Troubleshooting

See [Prometheus Metrics Troubleshooting](../troubleshooting/prometheus_metrics.md) for common issues.

## Disabling Telemetry

To run without telemetry overhead:

```toml
telemetry_enabled = false
```

This disables OpenTelemetry instrumentation but keeps basic logging active.
