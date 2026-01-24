---
name: netflix-chaos-engineering
description: Apply Netflix's chaos engineering methodology to build resilient systems. Emphasizes controlled failure injection, steady-state hypothesis testing, and building confidence through experimentation. Use when you need to verify system resilience under turbulent conditions.
---

# Netflix Chaos Engineering

## Overview

Netflix invented chaos engineering in response to their 2008 migration to AWS. Facing the reality that cloud infrastructure fails unpredictably, they created Chaos Monkey—and eventually the entire Simian Army—to proactively inject failures and build confidence in system resilience.

## The Pioneers

### Casey Rosenthal (Father of Chaos Engineering)

Led Netflix's Chaos Engineering team from 2015, formalizing the discipline and co-authoring the definitive O'Reilly book. Now CEO of Verica. His key insight: chaos engineering is about **building confidence**, not breaking things.

### Nora Jones

Co-pioneered chaos engineering at Netflix, co-authored the book, and later founded Jeli to apply these principles to incident analysis. Emphasized the human factors in resilience.

## References

- **Book**: "Chaos Engineering: System Resiliency in Practice" (O'Reilly, 2020)
- **Principles**: https://principlesofchaos.org/
- **Netflix Tech Blog**: https://netflixtechblog.com/

## Core Philosophy

> "The best way to avoid failure is to fail constantly."

> "Chaos Engineering is the discipline of experimenting on a system in order to build confidence in the system's capability to withstand turbulent conditions in production."

> "We're not trying to break things. We're trying to build confidence."

Chaos engineering is NOT about breaking things randomly. It's a disciplined approach to discovering systemic weaknesses before they cause outages.

## The Principles of Chaos Engineering

```
1. Build a Hypothesis around Steady State Behavior
   Define what "normal" looks like in measurable terms
   
2. Vary Real-World Events
   Inject failures that actually happen: server crashes, network issues, etc.
   
3. Run Experiments in Production
   Staging environments hide real-world complexity
   
4. Automate Experiments to Run Continuously
   One-time tests give false confidence
   
5. Minimize Blast Radius
   Start small, expand as confidence grows
```

## The Simian Army

Netflix's suite of chaos tools:

| Tool | Purpose |
|------|---------|
| **Chaos Monkey** | Randomly terminates instances |
| **Chaos Kong** | Simulates entire region failure |
| **Latency Monkey** | Injects artificial delays |
| **Conformity Monkey** | Finds instances not following best practices |
| **Janitor Monkey** | Cleans up unused resources |
| **Security Monkey** | Finds security vulnerabilities |

## When Implementing

### Always

- Define steady-state hypothesis before experimenting
- Start with smallest blast radius possible
- Have a "stop button" to halt experiments
- Run experiments in production (with safeguards)
- Automate experiments to run continuously
- Involve the whole team, not just SRE

### Never

- Inject chaos without a hypothesis
- Start with catastrophic failures
- Run experiments without monitoring
- Chaos without stakeholder buy-in
- Treat chaos as a one-time activity
- Forget to document learnings

### Prefer

- Gradual expansion of blast radius
- Automated experiments over manual
- Production over staging (with safeguards)
- Hypothesis-driven experiments
- Business metrics over technical metrics

## Implementation Patterns

### Chaos Experiment Structure

```python
# chaos_experiment.py
# The anatomy of a chaos experiment

from dataclasses import dataclass
from typing import Callable, Optional
from datetime import datetime, timedelta
import time

@dataclass
class SteadyStateHypothesis:
    """Define what 'normal' looks like"""
    name: str
    description: str
    probe: Callable[[], float]       # Returns a metric value
    tolerance_min: float
    tolerance_max: float
    
    def is_satisfied(self) -> bool:
        value = self.probe()
        return self.tolerance_min <= value <= self.tolerance_max

@dataclass
class ChaosAction:
    """The failure to inject"""
    name: str
    description: str
    execute: Callable[[], None]      # Inject the failure
    rollback: Callable[[], None]     # Undo the failure

@dataclass
class ChaosExperiment:
    """A complete chaos experiment"""
    name: str
    description: str
    hypothesis: SteadyStateHypothesis
    action: ChaosAction
    duration_seconds: int
    
    def run(self) -> dict:
        results = {
            'experiment': self.name,
            'started_at': datetime.now().isoformat(),
            'hypothesis_before': None,
            'hypothesis_during': None,
            'hypothesis_after': None,
            'success': False
        }
        
        # 1. Verify steady state BEFORE
        print(f"Checking steady state before experiment...")
        results['hypothesis_before'] = self.hypothesis.is_satisfied()
        
        if not results['hypothesis_before']:
            print("Steady state not satisfied before experiment. Aborting.")
            return results
        
        try:
            # 2. Inject the failure
            print(f"Injecting chaos: {self.action.name}")
            self.action.execute()
            
            # 3. Monitor during experiment
            print(f"Monitoring for {self.duration_seconds} seconds...")
            time.sleep(self.duration_seconds)
            results['hypothesis_during'] = self.hypothesis.is_satisfied()
            
        finally:
            # 4. Always rollback
            print(f"Rolling back: {self.action.name}")
            self.action.rollback()
        
        # 5. Verify steady state AFTER
        print("Checking steady state after rollback...")
        time.sleep(5)  # Allow recovery
        results['hypothesis_after'] = self.hypothesis.is_satisfied()
        
        # Success = hypothesis held during and after
        results['success'] = (
            results['hypothesis_during'] and 
            results['hypothesis_after']
        )
        
        results['completed_at'] = datetime.now().isoformat()
        return results


# Example: Test resilience to instance termination
def create_instance_termination_experiment(instance_id: str):
    
    def check_error_rate():
        # Query your monitoring system
        return get_error_rate_percentage()
    
    def terminate_instance():
        # Actually terminate the instance
        ec2.terminate_instances(InstanceIds=[instance_id])
    
    def noop_rollback():
        # Auto-scaling should replace the instance
        pass
    
    hypothesis = SteadyStateHypothesis(
        name="Error rate within tolerance",
        description="Error rate should remain below 1%",
        probe=check_error_rate,
        tolerance_min=0,
        tolerance_max=1.0
    )
    
    action = ChaosAction(
        name=f"Terminate instance {instance_id}",
        description="Simulate instance failure",
        execute=terminate_instance,
        rollback=noop_rollback
    )
    
    return ChaosExperiment(
        name="Instance Termination Resilience",
        description="Verify system handles instance loss gracefully",
        hypothesis=hypothesis,
        action=action,
        duration_seconds=300
    )
```

### Chaos Monkey Implementation

```python
# chaos_monkey.py
# Simplified Chaos Monkey - random instance termination

import random
import time
from datetime import datetime
from typing import List, Optional

class ChaosMonkey:
    """
    Netflix's Chaos Monkey: randomly terminates instances
    to ensure services can handle instance failures.
    """
    
    def __init__(self, 
                 cloud_client,
                 excluded_services: List[str] = None,
                 probability: float = 0.1,
                 schedule_start_hour: int = 9,
                 schedule_end_hour: int = 15):
        """
        Args:
            cloud_client: AWS/GCP/Azure client
            excluded_services: Services to never touch
            probability: Chance of termination per run (0-1)
            schedule_start_hour: Only run after this hour
            schedule_end_hour: Stop running after this hour
        """
        self.client = cloud_client
        self.excluded = set(excluded_services or [])
        self.probability = probability
        self.start_hour = schedule_start_hour
        self.end_hour = schedule_end_hour
        self.termination_log = []
    
    def is_within_schedule(self) -> bool:
        """Only cause chaos during business hours (when humans can respond)"""
        hour = datetime.now().hour
        weekday = datetime.now().weekday()
        
        # Monday-Friday, 9am-3pm
        return weekday < 5 and self.start_hour <= hour < self.end_hour
    
    def get_eligible_instances(self) -> List[dict]:
        """Get instances that can be terminated"""
        all_instances = self.client.list_instances()
        
        eligible = []
        for instance in all_instances:
            service = instance.get('service_name', '')
            
            # Skip excluded services
            if service in self.excluded:
                continue
            
            # Skip if service has < 2 instances (no redundancy)
            service_count = sum(
                1 for i in all_instances 
                if i.get('service_name') == service
            )
            if service_count < 2:
                continue
            
            # Skip if instance is too new (let it warm up)
            age_minutes = instance.get('age_minutes', 0)
            if age_minutes < 30:
                continue
            
            eligible.append(instance)
        
        return eligible
    
    def run(self) -> Optional[dict]:
        """Execute one round of chaos"""
        
        # Check schedule
        if not self.is_within_schedule():
            return {'action': 'skipped', 'reason': 'outside schedule'}
        
        # Check probability
        if random.random() > self.probability:
            return {'action': 'skipped', 'reason': 'probability check'}
        
        # Get eligible instances
        eligible = self.get_eligible_instances()
        if not eligible:
            return {'action': 'skipped', 'reason': 'no eligible instances'}
        
        # Select random victim
        victim = random.choice(eligible)
        
        # Terminate!
        result = {
            'action': 'terminated',
            'instance_id': victim['id'],
            'service': victim.get('service_name'),
            'timestamp': datetime.now().isoformat()
        }
        
        self.client.terminate_instance(victim['id'])
        self.termination_log.append(result)
        
        return result
    
    def run_continuously(self, interval_seconds: int = 300):
        """Run chaos monkey on a schedule"""
        print("Chaos Monkey starting... 🐵")
        
        while True:
            result = self.run()
            if result['action'] == 'terminated':
                print(f"🔥 Terminated {result['instance_id']} "
                      f"({result['service']})")
            else:
                print(f"😴 Skipped: {result['reason']}")
            
            time.sleep(interval_seconds)
```

### Steady State Metrics

```python
# steady_state.py
# Define and monitor steady state

from dataclasses import dataclass
from typing import List, Callable
from prometheus_client import CollectorRegistry, Gauge

@dataclass
class BusinessMetric:
    """
    Netflix insight: measure BUSINESS metrics, not just technical ones.
    Users don't care about CPU; they care about streams starting.
    """
    name: str
    description: str
    query: Callable[[], float]
    unit: str
    
    # Steady state bounds
    min_healthy: float
    max_healthy: float

# Netflix's key business metric
streams_per_second = BusinessMetric(
    name="streams_starting_per_second",
    description="Rate of successful stream starts",
    query=lambda: prometheus.query("rate(streams_started_total[1m])"),
    unit="streams/sec",
    min_healthy=50000,
    max_healthy=200000
)

class SteadyStateMonitor:
    """Monitor steady state during chaos experiments"""
    
    def __init__(self, metrics: List[BusinessMetric]):
        self.metrics = metrics
        self.baseline = {}
    
    def capture_baseline(self, duration_seconds: int = 300):
        """Capture baseline metrics before experiment"""
        samples = {m.name: [] for m in self.metrics}
        
        for _ in range(duration_seconds // 10):
            for metric in self.metrics:
                samples[metric.name].append(metric.query())
            time.sleep(10)
        
        # Calculate baseline statistics
        for metric in self.metrics:
            values = samples[metric.name]
            self.baseline[metric.name] = {
                'mean': sum(values) / len(values),
                'min': min(values),
                'max': max(values)
            }
    
    def check_steady_state(self) -> dict:
        """Check if all metrics are within healthy bounds"""
        results = {}
        all_healthy = True
        
        for metric in self.metrics:
            current = metric.query()
            healthy = metric.min_healthy <= current <= metric.max_healthy
            
            results[metric.name] = {
                'current': current,
                'healthy_range': (metric.min_healthy, metric.max_healthy),
                'is_healthy': healthy
            }
            
            if not healthy:
                all_healthy = False
        
        results['all_healthy'] = all_healthy
        return results
    
    def deviation_from_baseline(self) -> dict:
        """How far are we from baseline?"""
        deviations = {}
        
        for metric in self.metrics:
            current = metric.query()
            baseline = self.baseline.get(metric.name, {}).get('mean', current)
            
            if baseline != 0:
                deviation_pct = ((current - baseline) / baseline) * 100
            else:
                deviation_pct = 0
            
            deviations[metric.name] = {
                'current': current,
                'baseline': baseline,
                'deviation_percent': deviation_pct
            }
        
        return deviations
```

### Blast Radius Control

```python
# blast_radius.py
# Control the scope of chaos experiments

from enum import Enum
from dataclasses import dataclass
from typing import List, Optional

class BlastRadius(Enum):
    """Start small, expand as confidence grows"""
    SINGLE_INSTANCE = 1      # One instance
    SERVICE_PERCENTAGE = 2   # X% of a service
    ENTIRE_SERVICE = 3       # All instances of a service
    AVAILABILITY_ZONE = 4    # Entire AZ
    REGION = 5               # Entire region (Chaos Kong)

@dataclass
class ExperimentScope:
    """Define the scope of an experiment"""
    blast_radius: BlastRadius
    target_service: Optional[str] = None
    target_percentage: float = 0.1
    target_az: Optional[str] = None
    target_region: Optional[str] = None
    
    def get_targets(self, all_instances: List[dict]) -> List[dict]:
        """Get instances within the blast radius"""
        
        if self.blast_radius == BlastRadius.SINGLE_INSTANCE:
            # Just one random instance
            import random
            eligible = [i for i in all_instances 
                       if i.get('service') == self.target_service]
            return [random.choice(eligible)] if eligible else []
        
        elif self.blast_radius == BlastRadius.SERVICE_PERCENTAGE:
            # X% of a service
            import random
            eligible = [i for i in all_instances 
                       if i.get('service') == self.target_service]
            count = max(1, int(len(eligible) * self.target_percentage))
            return random.sample(eligible, min(count, len(eligible)))
        
        elif self.blast_radius == BlastRadius.ENTIRE_SERVICE:
            return [i for i in all_instances 
                   if i.get('service') == self.target_service]
        
        elif self.blast_radius == BlastRadius.AVAILABILITY_ZONE:
            return [i for i in all_instances 
                   if i.get('az') == self.target_az]
        
        elif self.blast_radius == BlastRadius.REGION:
            # Chaos Kong - nuclear option
            return [i for i in all_instances 
                   if i.get('region') == self.target_region]
        
        return []


class GraduatedChaos:
    """Gradually increase blast radius as confidence grows"""
    
    def __init__(self, service: str):
        self.service = service
        self.current_level = BlastRadius.SINGLE_INSTANCE
        self.success_streak = 0
        self.required_successes = 5  # Before escalating
    
    def record_result(self, success: bool):
        if success:
            self.success_streak += 1
            if self.success_streak >= self.required_successes:
                self.escalate()
        else:
            self.success_streak = 0
            self.de_escalate()
    
    def escalate(self):
        """Increase blast radius"""
        levels = list(BlastRadius)
        current_idx = levels.index(self.current_level)
        if current_idx < len(levels) - 1:
            self.current_level = levels[current_idx + 1]
            self.success_streak = 0
            print(f"Escalating to {self.current_level.name}")
    
    def de_escalate(self):
        """Decrease blast radius after failure"""
        levels = list(BlastRadius)
        current_idx = levels.index(self.current_level)
        if current_idx > 0:
            self.current_level = levels[current_idx - 1]
            print(f"De-escalating to {self.current_level.name}")
```

## Mental Model

Netflix chaos engineering asks:

1. **What is steady state?** Define normal in measurable terms
2. **What could go wrong?** Real-world failures to simulate
3. **What's our hypothesis?** System should maintain steady state
4. **How small can we start?** Minimize blast radius
5. **Did we learn something?** Every experiment should teach us

## Signature Netflix Moves

- Chaos Monkey for random instance termination
- Steady state hypothesis before every experiment
- Business metrics over technical metrics
- Production experiments (with safeguards)
- Graduated blast radius expansion
- Automated, continuous chaos
