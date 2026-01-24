# Gremlin Attack Catalog

## Resource Attacks

### CPU

Consume CPU resources to test performance under load.

```yaml
attack: cpu
parameters:
  cores: 2           # Number of cores to consume
  percent: 80        # Percentage of each core
  duration: 300      # Seconds
```

**Use cases:**
- Test autoscaling triggers
- Validate resource limits
- Measure degraded performance

### Memory

Consume memory to test behavior under memory pressure.

```yaml
attack: memory
parameters:
  amount: 75         # Percent or absolute (e.g., "2GB")
  duration: 300
```

**Use cases:**
- Test OOM killer behavior
- Validate memory limits
- Check graceful degradation

### Disk

Fill disk space to test storage handling.

```yaml
attack: disk
parameters:
  dir: /tmp
  percent: 90
  duration: 300
```

**Use cases:**
- Test log rotation
- Validate disk alerts
- Check write failure handling

### IO

Stress disk I/O operations.

```yaml
attack: io
parameters:
  mode: readwrite    # read, write, or readwrite
  workers: 4
  block_size: 64k
  duration: 300
```

**Use cases:**
- Test database under I/O stress
- Validate I/O-bound workloads
- Check timeout configurations

## Network Attacks

### Latency

Add network latency to connections.

```yaml
attack: latency
parameters:
  ms: 200            # Latency in milliseconds
  jitter: 50         # Optional variance
  hosts:             # Target hosts
    - "api.example.com"
  ports:             # Target ports
    - 443
  duration: 300
```

**Use cases:**
- Test timeout configurations
- Validate circuit breakers
- Simulate cross-region latency

### Packet Loss

Drop a percentage of network packets.

```yaml
attack: packet_loss
parameters:
  percent: 10        # Percent of packets to drop
  hosts:
    - "database.internal"
  duration: 300
```

**Use cases:**
- Test retry logic
- Validate idempotency
- Check connection recovery

### DNS

Block or delay DNS resolution.

```yaml
attack: dns
parameters:
  mode: block        # block or delay
  delay_ms: 1000     # For delay mode
  hostnames:
    - "api.example.com"
  duration: 300
```

**Use cases:**
- Test DNS failover
- Validate caching
- Check timeout handling

### Blackhole

Drop all traffic to specific hosts/ports.

```yaml
attack: blackhole
parameters:
  hosts:
    - "10.0.0.0/8"
  ports:
    - 5432
  protocol: tcp
  duration: 300
```

**Use cases:**
- Test circuit breakers
- Validate failover
- Simulate network partitions

## State Attacks

### Process Killer

Terminate specific processes.

```yaml
attack: process_killer
parameters:
  process: "myapp"
  signal: SIGKILL    # or SIGTERM
  interval: 60       # Kill every N seconds
  duration: 300
```

**Use cases:**
- Test process supervision
- Validate restart behavior
- Check data persistence

### Time Travel

Shift system time forward or backward.

```yaml
attack: time_travel
parameters:
  offset: "1h"       # Time offset
  direction: future  # future or past
  duration: 300
```

**Use cases:**
- Test certificate expiration
- Validate TTL handling
- Check scheduled job behavior

### Shutdown

Gracefully or forcefully shut down a host.

```yaml
attack: shutdown
parameters:
  delay: 60          # Seconds before shutdown
  reboot: true       # Reboot after shutdown
```

**Use cases:**
- Test failover
- Validate data persistence
- Check cluster recovery

## Application Attacks

### HTTP

Inject failures into HTTP endpoints.

```yaml
attack: http
parameters:
  endpoint: "/api/v1/users"
  error_rate: 0.3    # 30% of requests fail
  error_code: 503
  latency_ms: 500
  duration: 300
```

**Use cases:**
- Test client error handling
- Validate retry logic
- Check fallback behavior

### Certificate Expiry

Simulate certificate problems.

```yaml
attack: certificate
parameters:
  mode: expire       # expire, revoke, or mismatch
  target: "api.example.com"
  duration: 300
```

**Use cases:**
- Test certificate rotation
- Validate TLS error handling
- Check alerting systems

## Container/Kubernetes Attacks

### Container Kill

Terminate containers.

```yaml
attack: container_kill
target:
  type: kubernetes
  labels:
    app: myapp
parameters:
  count: 1
  interval: 60
```

### Pod Network Partition

Isolate pods from each other.

```yaml
attack: pod_network_partition
target:
  namespace: production
  labels:
    app: myapp
parameters:
  partition_from:
    labels:
      app: database
  duration: 300
```

## Safety Controls

### Halt Conditions

```yaml
halt_conditions:
  - type: metric_threshold
    metric: error_rate
    threshold: 0.5
    operator: greater_than
  
  - type: health_check
    endpoint: https://api.example.com/health
    expected_status: 200
    interval: 10
  
  - type: time_window
    allowed_days: [monday, tuesday, wednesday, thursday, friday]
    allowed_hours: "09:00-17:00"
    timezone: "America/New_York"
```

### Blast Radius Limits

```yaml
blast_radius:
  max_hosts: 5
  max_containers: 10
  max_percent_of_service: 25
  excluded_tags:
    - production-critical
    - database-primary
```

## Attack Composition

### Sequential Attacks

```yaml
scenario: "Cascading Failure Test"
steps:
  - attack: latency
    target: database
    parameters:
      ms: 500
    duration: 120
    
  - wait: 30
  
  - attack: packet_loss
    target: cache
    parameters:
      percent: 20
    duration: 120
```

### Parallel Attacks

```yaml
scenario: "Multi-Service Stress"
parallel:
  - attack: cpu
    target: web-servers
    parameters:
      percent: 70
  
  - attack: memory
    target: api-servers
    parameters:
      percent: 60

duration: 300
```

## Metrics to Watch

| Attack Type | Key Metrics |
|-------------|-------------|
| CPU | Request latency, queue depth, autoscale events |
| Memory | OOM kills, swap usage, GC frequency |
| Network Latency | P99 latency, timeout rate, retry rate |
| Packet Loss | Error rate, retry rate, circuit breaker trips |
| Process Kill | Recovery time, data loss, request failures |
