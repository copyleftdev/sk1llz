# Error Budget Deep Dive

## The Mathematics of Error Budgets

### Basic Calculation

```
Error Budget = 1 - SLO

Example:
  SLO = 99.9% (three nines)
  Error Budget = 1 - 0.999 = 0.001 = 0.1%
```

### Time Translation

| SLO | Error Budget | Per Year | Per Month | Per Week |
|-----|--------------|----------|-----------|----------|
| 99% | 1% | 3.65 days | 7.3 hours | 1.68 hours |
| 99.9% | 0.1% | 8.76 hours | 43.8 min | 10.1 min |
| 99.95% | 0.05% | 4.38 hours | 21.9 min | 5.04 min |
| 99.99% | 0.01% | 52.6 min | 4.38 min | 1.01 min |
| 99.999% | 0.001% | 5.26 min | 26.3 sec | 6.05 sec |

### Burn Rate

```
Burn Rate = (Error Rate) / (Error Budget Rate)

Where:
  Error Budget Rate = Error Budget / Window Duration

Example:
  SLO: 99.9% over 30 days
  Error Budget: 43.8 minutes
  Error Budget Rate: 43.8 / (30 * 24 * 60) = 0.001% per minute

  If current error rate is 0.01% per minute:
  Burn Rate = 0.01 / 0.001 = 10x

  At 10x burn rate, budget exhausted in 3 days instead of 30
```

## Error Budget Policies

### Healthy Budget (>50% remaining)
- Ship new features
- Run experiments
- Take calculated risks
- Conduct chaos engineering

### Caution Zone (25-50% remaining)
- Review recent changes
- Increase monitoring
- Slow feature velocity
- Prioritize stability fixes

### Critical (<25% remaining)
- Freeze non-critical deployments
- All hands on reliability
- Incident review for all errors
- Aggressive rollback policy

### Exhausted (0% remaining)
- Complete feature freeze
- Only emergency fixes
- Post-mortem for every incident
- Executive review required

## Multi-Window Error Budgets

Google recommends tracking budgets across multiple windows:

```
Short window (1 hour):  Catch acute issues quickly
Medium window (1 day):  Track daily trends
Long window (30 days):  Strategic planning
```

### Alert Thresholds

| Window | Budget Consumption | Alert Level |
|--------|-------------------|-------------|
| 1 hour | >2% | Page |
| 6 hours | >5% | Page |
| 1 day | >10% | Ticket |
| 3 days | >20% | Ticket |

## Error Budget and Deployment

### Before Deployment
```
remaining_budget = current_budget - expected_error_rate * deployment_time
if remaining_budget < threshold:
    delay_deployment()
```

### Canary Analysis
```
canary_error_rate = errors_in_canary / requests_to_canary
baseline_error_rate = errors_in_baseline / requests_to_baseline

if canary_error_rate > baseline_error_rate * 1.1:
    rollback_canary()
```

## Sample SLO Document

```yaml
service: payment-api
team: payments
 
slis:
  - name: availability
    description: Proportion of successful requests
    good_events: "http_status < 500"
    total_events: "all requests"
    
  - name: latency
    description: Proportion of fast requests
    good_events: "latency_ms < 200"
    total_events: "all requests"

slos:
  - sli: availability
    target: 0.999
    window: 30d
    
  - sli: latency
    target: 0.99
    window: 30d

error_budget_policy:
  healthy:
    budget_remaining: ">50%"
    actions:
      - "Normal feature development"
      - "Chaos experiments allowed"
      
  warning:
    budget_remaining: "25-50%"
    actions:
      - "Review recent deployments"
      - "Increase monitoring"
      
  critical:
    budget_remaining: "<25%"
    actions:
      - "Feature freeze"
      - "Focus on reliability"
      
  exhausted:
    budget_remaining: "0%"
    actions:
      - "Complete deployment freeze"
      - "Executive escalation"
```
