# Prometheus Metrics Troubleshooting

## Common Issues and Solutions

### 1. Metrics Endpoint Accessibility

#### Testing the Endpoint
```bash
curl http://localhost:3000/metrics
```

**Expected Output** (Prometheus text format):
```
# HELP app_memory_usage_bytes Current RAM usage of the application process
# TYPE app_memory_usage_bytes gauge
app_memory_usage_bytes 45678912
```

#### Troubleshooting

**Issue**: Endpoint returns 404

**Possible Causes**:

**A. Old Executable**: Your binary needs to be rebuilt
```powershell
cargo build --release --features cuda
```

**B. Server Not Running**
```bash
code-rag start --config code-rag.test.toml
# Or
code-rag serve
```

**C. Wrong Port**

Check your config:
```toml
server_port = 3000  # Must match curl request
```

---

### 2. Prometheus Cannot Scrape Targets

#### Symptom
In Prometheus UI (http://localhost:9090/targets), status shows "down" or "connection refused"

#### Diagnosis
```bash
# Check if code-rag server is running
curl http://localhost:3000/health

# Check Docker network connectivity
docker exec prometheus wget -O- http://host.docker.internal:3000/health
```

#### Possible Causes

**A. Server Not Running**
```bash
# Start code-rag server
code-rag start --config code-rag.toml
```

**B. Wrong Port in prometheus.yml**

Check `prometheus.yml`:
```yaml
scrape_configs:
  - job_name: 'code-rag-server'
    static_configs:
      - targets: ['host.docker.internal:3000']  # Must match server port
```

If server runs on different port (e.g., 8080):
```yaml
- targets: ['host.docker.internal:8080']
```

**C. Firewall Blocking Connection**

Windows:
```powershell
# Allow port 3000 (if Windows Firewall is active)
New-NetFirewallRule -DisplayName "code-rag" -Direction Inbound -LocalPort 3000 -Protocol TCP -Action Allow
```

Linux:
```bash
# Check firewall status
sudo ufw status

# Allow port if needed
sudo ufw allow 3000/tcp
```

**D. Docker Network Issues on Windows**

Use `host.docker.internal` instead of `localhost` in `prometheus.yml` when running on Windows/macOS.

Linux alternative:
```yaml
- targets: ['172.17.0.1:3000']  # Docker bridge gateway
```

---

### 3. Jaeger Not Receiving Traces

#### Symptom
Jaeger UI shows no traces for service `code-rag-server`

#### Diagnosis
```bash
# Check Jaeger collector logs
docker logs jaeger

# Should see: "OTLP receiver started"

# Test OTLP endpoint
curl http://localhost:4317
```

#### Possible Causes

**A. Telemetry Not Enabled**

In `code-rag.toml`:
```toml
telemetry_enabled = true  # Must be true
telemetry_endpoint = "http://localhost:4317"
```

**B. Wrong Endpoint Configuration**

From **inside Docker**, use container name:
```toml
telemetry_endpoint = "http://jaeger:4317"
```

From **host machine**, use localhost:
```toml
telemetry_endpoint = "http://localhost:4317"
```

**C. OTLP Not Enabled in Jaeger**

Verify in `docker-compose.telemetry.yml`:
```yaml
jaeger:
  environment:
    - COLLECTOR_OTLP_ENABLED=true  # Must be set
```

**D. No Activity Yet**

Traces are only created when requests are made. Generate activity:
```bash
# Make search request
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{"query":"test","limit":5}'
```

Then refresh Jaeger UI.

---

### 4. Memory Gauge Showing Zero or Not Updating

#### Symptom
Prometheus query `app_memory_usage_bytes` returns 0 or stale value

#### Diagnosis

**A. Check Gauge Registration**

Look for errors in server startup logs:
```
Failed to register memory gauge: <error>
```

**B. System Info Failure**

The gauge uses `sysinfo` crate. Check logs for:
```
Failed to refresh process info
```

**C. Observable Callback Not Firing**

Verify Prometheus is actually scraping (check `/targets` in Prometheus UI).

Callback only fires when Prometheus scrapes the endpoint.

---

### 5. Grafana Cannot Connect to Prometheus

#### Symptom
Grafana shows "Bad Gateway" when adding Prometheus data source

#### Solution

**A. Use Docker Network DNS**

In Grafana data source settings, use:
```
URL: http://prometheus:9090
```

NOT:
```
URL: http://localhost:9090  # Won't work from inside Docker
```

**B. Verify Network**

All services must be on same Docker network (`observability`):
```bash
docker network inspect observability
# Should list: jaeger, prometheus, grafana
```

**C. Restart Grafana**
```bash
docker-compose -f docker-compose.telemetry.yml restart grafana
```

---

### 6. High Memory Usage During Telemetry

#### Symptom
`app_memory_usage_bytes` shows excessive growth

#### Diagnosis

**A. Check Trace Batching**

OTLP exporter batches traces. Large batches can buffer memory.

**Workaround**: Flush more frequently (requires code change to `telemetry.rs`)

**B. Chrome Tracing in CLI Mode**

Chrome traces are kept in memory until process exit.

**Solution**: Disable telemetry for long-running CLI operations:
```toml
telemetry_enabled = false
```

---

### 7. Docker Compose Fails to Start

#### Symptom
```bash
docker-compose -f docker-compose.telemetry.yml up -d
# Error: Cannot load prometheus.yml
```

#### Solution

**A. Verify File Exists**
```bash
# Check prometheus.yml exists in project root
ls prometheus.yml
```

**B. Check Working Directory**

Run docker-compose from project root:
```bash
cd /path/to/code-rag
docker-compose -f docker-compose.telemetry.yml up -d
```

**C. Check File Permissions** (Linux/macOS)
```bash
chmod 644 prometheus.yml
```

---

## Verification Checklist

After troubleshooting, verify full stack:

```bash
# 1. Check all Docker containers running
docker-compose -f docker-compose.telemetry.yml ps
# Should show: jaeger (Up), prometheus (Up), grafana (Up)

# 2. Check code-rag server health
curl http://localhost:3000/health
# Should return: 200 OK

# 3. Check Prometheus targets
# Navigate to: http://localhost:9090/targets
# Should show: code-rag-server (UP)

# 4. Check Jaeger UI
# Navigate to: http://localhost:16686
# Select service: code-rag-server, click "Find Traces"

# 5. Generate test activity
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{"query":"test","limit":5}'

# 6. Verify trace appears in Jaeger (refresh UI)
```

---

## Getting Help

If issues persist:

1. **Check Logs**:
   ```bash
   # Code-rag server logs
   # (should show telemetry initialization)
   
   # Docker container logs
   docker logs jaeger
   docker logs prometheus
   docker logs grafana
   ```

2. **Capture Diagnostics**:
   ```bash
   # Network connectivity
   docker exec prometheus wget -O- http://host.docker.internal:3000/health
   
   # Prometheus config validation
   docker exec prometheus promtool check config /etc/prometheus/prometheus.yml
   ```

3. **Review Documentation**:
   - [Telemetry Features Guide](../features/telemetry.md)
   - [Configuration Reference](../configuration/telemetry_config.md)
   - [Architecture Details](../architecture/observability.md)

4. **File an Issue**: Include code-rag version, OS, and full error messages
