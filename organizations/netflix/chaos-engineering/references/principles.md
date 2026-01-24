# Principles of Chaos Engineering

## The Official Principles

From [principlesofchaos.org](https://principlesofchaos.org/):

### 1. Build a Hypothesis around Steady State Behavior

Focus on the measurable output of a system, rather than internal attributes. Measurements of that output over a short period constitute a proxy for the system's steady state.

**Examples of steady state metrics:**
- Overall throughput
- Error rates
- Latency percentiles
- Business metrics (orders per second, streams started)

### 2. Vary Real-World Events

Chaos variables reflect real-world events:
- Hardware failures
- Network partitions
- Traffic spikes
- Resource exhaustion
- Clock skew
- Byzantine failures

Prioritize events by potential impact or estimated frequency.

### 3. Run Experiments in Production

Systems behave differently in production than in test environments. The only way to guarantee we can handle production conditions is to test in production.

**Safeguards for production chaos:**
- Start small (canary)
- Have abort mechanisms
- Run during business hours (humans available)
- Monitor closely

### 4. Automate Experiments to Run Continuously

Running experiments manually is labor-intensive and unsustainable. Automate both execution and analysis.

### 5. Minimize Blast Radius

Start with the smallest scope that provides meaningful results:
1. Single instance
2. Single service
3. Single availability zone
4. Single region

Expand only after building confidence.

## Advanced Principles

### Principle of Observable Effects

If you can't measure it, you can't chaos engineer it. Ensure sufficient observability before experimenting.

### Principle of Known Unknowns

Chaos engineering is most valuable for finding **unknown unknowns**—failure modes you haven't considered.

### Principle of Continuous Verification

The system changes constantly. What was resilient yesterday may be fragile today. Chaos must be continuous.

## Chaos Engineering vs. Testing

| Aspect | Testing | Chaos Engineering |
|--------|---------|-------------------|
| **Goal** | Verify known behaviors | Discover unknown behaviors |
| **Environment** | Test/staging | Production |
| **Scope** | Component | System |
| **Frequency** | On change | Continuous |
| **Output** | Pass/fail | Learning |

## Chaos Maturity Model

### Level 1: Ad Hoc
- Manual experiments
- Reactive (after incidents)
- No documentation

### Level 2: Planned
- Scheduled game days
- Basic tooling
- Some documentation

### Level 3: Continuous
- Automated experiments
- Production chaos
- Systematic approach

### Level 4: Optimized
- Continuous in production
- Full automation
- Drives architecture decisions

## Netflix-Specific Insights

### The Simian Army Lineage

```
Chaos Monkey (2011)
    ↓
Simian Army (2012)
    ├── Chaos Gorilla (AZ failures)
    ├── Chaos Kong (Region failures)
    ├── Latency Monkey (Delays)
    ├── Conformity Monkey (Best practices)
    └── Janitor Monkey (Cleanup)
    ↓
FIT (2014) - Failure Injection Testing
    ↓
ChAP (2017) - Chaos Automation Platform
```

### Key Netflix Learnings

1. **Chaos in production is essential** - Staging doesn't reveal production issues
2. **Business metrics matter most** - Users don't care about CPU; they care about streams
3. **Automation is required** - Manual chaos doesn't scale
4. **Culture > Tools** - Teams must embrace failure as learning
