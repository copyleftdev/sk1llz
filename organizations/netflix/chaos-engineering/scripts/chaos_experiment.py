#!/usr/bin/env python3
"""
chaos_experiment.py
Netflix-style chaos experiment framework.

Usage:
    python chaos_experiment.py --config experiment.yaml
    python chaos_experiment.py --dry-run --config experiment.yaml
"""

import argparse
import json
import logging
import sys
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime
from typing import Callable, Dict, List, Optional
from enum import Enum

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


class ExperimentStatus(Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    ABORTED = "aborted"
    FAILED = "failed"


@dataclass
class SteadyStateHypothesis:
    """Define what 'normal' looks like."""
    
    name: str
    description: str
    probe: Callable[[], float]
    tolerance_min: float
    tolerance_max: float
    
    def check(self) -> dict:
        """Check if steady state is satisfied."""
        try:
            value = self.probe()
            satisfied = self.tolerance_min <= value <= self.tolerance_max
            return {
                'name': self.name,
                'value': value,
                'tolerance': (self.tolerance_min, self.tolerance_max),
                'satisfied': satisfied
            }
        except Exception as e:
            return {
                'name': self.name,
                'error': str(e),
                'satisfied': False
            }


@dataclass
class ChaosAction(ABC):
    """Base class for chaos actions."""
    
    name: str
    description: str
    
    @abstractmethod
    def execute(self) -> dict:
        """Inject the failure."""
        pass
    
    @abstractmethod
    def rollback(self) -> dict:
        """Undo the failure."""
        pass


@dataclass
class InstanceTerminationAction(ChaosAction):
    """Terminate a random instance (Chaos Monkey style)."""
    
    target_service: str
    instance_selector: Callable[[], str] = None
    
    def execute(self) -> dict:
        instance_id = self.instance_selector() if self.instance_selector else "mock-instance"
        logger.info(f"🔥 Terminating instance: {instance_id}")
        # In real implementation: cloud_client.terminate_instance(instance_id)
        return {'action': 'terminate', 'instance_id': instance_id}
    
    def rollback(self) -> dict:
        logger.info("Auto-scaling should replace terminated instance")
        return {'action': 'noop', 'reason': 'auto-scaling handles recovery'}


@dataclass
class LatencyInjectionAction(ChaosAction):
    """Inject latency into service calls."""
    
    target_service: str
    latency_ms: int
    percentage: float = 100.0
    
    def execute(self) -> dict:
        logger.info(f"🐢 Injecting {self.latency_ms}ms latency to {self.target_service}")
        # In real implementation: service_mesh.add_latency(...)
        return {'action': 'inject_latency', 'latency_ms': self.latency_ms}
    
    def rollback(self) -> dict:
        logger.info(f"Removing latency injection from {self.target_service}")
        # In real implementation: service_mesh.remove_latency(...)
        return {'action': 'remove_latency'}


@dataclass
class ExperimentResult:
    """Results of a chaos experiment."""
    
    experiment_name: str
    status: ExperimentStatus
    started_at: datetime
    completed_at: Optional[datetime] = None
    
    hypothesis_before: Optional[dict] = None
    hypothesis_during: Optional[dict] = None
    hypothesis_after: Optional[dict] = None
    
    action_result: Optional[dict] = None
    rollback_result: Optional[dict] = None
    
    abort_reason: Optional[str] = None
    
    def to_dict(self) -> dict:
        return {
            'experiment': self.experiment_name,
            'status': self.status.value,
            'started_at': self.started_at.isoformat(),
            'completed_at': self.completed_at.isoformat() if self.completed_at else None,
            'hypothesis_before': self.hypothesis_before,
            'hypothesis_during': self.hypothesis_during,
            'hypothesis_after': self.hypothesis_after,
            'action_result': self.action_result,
            'success': self._is_success()
        }
    
    def _is_success(self) -> bool:
        """Experiment succeeds if hypothesis held throughout."""
        if self.status != ExperimentStatus.COMPLETED:
            return False
        
        before_ok = self.hypothesis_before and self.hypothesis_before.get('satisfied', False)
        during_ok = self.hypothesis_during and self.hypothesis_during.get('satisfied', False)
        after_ok = self.hypothesis_after and self.hypothesis_after.get('satisfied', False)
        
        return before_ok and during_ok and after_ok


@dataclass
class ChaosExperiment:
    """A complete chaos experiment definition."""
    
    name: str
    description: str
    hypothesis: SteadyStateHypothesis
    action: ChaosAction
    duration_seconds: int = 60
    
    # Safety controls
    abort_on_hypothesis_failure: bool = True
    max_duration_seconds: int = 300
    
    def run(self, dry_run: bool = False) -> ExperimentResult:
        """Execute the chaos experiment."""
        result = ExperimentResult(
            experiment_name=self.name,
            status=ExperimentStatus.RUNNING,
            started_at=datetime.now()
        )
        
        logger.info(f"🧪 Starting experiment: {self.name}")
        logger.info(f"📝 Hypothesis: {self.hypothesis.description}")
        
        # Step 1: Check steady state BEFORE
        logger.info("Checking steady state before experiment...")
        result.hypothesis_before = self.hypothesis.check()
        
        if not result.hypothesis_before['satisfied']:
            logger.warning("❌ Steady state not satisfied before experiment!")
            result.status = ExperimentStatus.ABORTED
            result.abort_reason = "Pre-experiment steady state check failed"
            result.completed_at = datetime.now()
            return result
        
        logger.info(f"✅ Steady state OK: {result.hypothesis_before['value']}")
        
        if dry_run:
            logger.info("🏃 DRY RUN - Skipping actual chaos injection")
            result.status = ExperimentStatus.COMPLETED
            result.completed_at = datetime.now()
            return result
        
        try:
            # Step 2: Inject chaos
            logger.info(f"💥 Injecting chaos: {self.action.name}")
            result.action_result = self.action.execute()
            
            # Step 3: Monitor during experiment
            logger.info(f"⏱️  Monitoring for {self.duration_seconds} seconds...")
            
            check_interval = min(10, self.duration_seconds // 3)
            elapsed = 0
            
            while elapsed < self.duration_seconds:
                time.sleep(check_interval)
                elapsed += check_interval
                
                result.hypothesis_during = self.hypothesis.check()
                logger.info(f"📊 [{elapsed}s] Steady state: {result.hypothesis_during['value']}")
                
                if self.abort_on_hypothesis_failure and not result.hypothesis_during['satisfied']:
                    logger.warning("⚠️  Hypothesis failed during experiment - aborting!")
                    result.status = ExperimentStatus.ABORTED
                    result.abort_reason = "Hypothesis failed during experiment"
                    break
            
        except Exception as e:
            logger.error(f"💀 Experiment failed: {e}")
            result.status = ExperimentStatus.FAILED
            result.abort_reason = str(e)
        
        finally:
            # Step 4: Always rollback
            logger.info("🔄 Rolling back chaos injection...")
            result.rollback_result = self.action.rollback()
        
        # Step 5: Check steady state AFTER
        logger.info("Checking steady state after rollback...")
        time.sleep(5)  # Allow recovery
        result.hypothesis_after = self.hypothesis.check()
        
        if result.status == ExperimentStatus.RUNNING:
            result.status = ExperimentStatus.COMPLETED
        
        result.completed_at = datetime.now()
        
        # Report
        if result._is_success():
            logger.info("🎉 Experiment PASSED - System remained resilient!")
        else:
            logger.warning("😰 Experiment FAILED - System was not resilient")
        
        return result


def demo_experiment():
    """Demonstrate the chaos experiment framework."""
    
    # Mock probe that simulates error rate
    error_rate = [0.5]  # Mutable to allow changes during experiment
    
    def get_error_rate() -> float:
        return error_rate[0]
    
    # Define hypothesis
    hypothesis = SteadyStateHypothesis(
        name="Error rate within tolerance",
        description="Error rate should remain below 1%",
        probe=get_error_rate,
        tolerance_min=0,
        tolerance_max=1.0
    )
    
    # Define action
    action = InstanceTerminationAction(
        name="Terminate random instance",
        description="Kill a random instance to test redundancy",
        target_service="api-service",
        instance_selector=lambda: "i-12345"
    )
    
    # Create experiment
    experiment = ChaosExperiment(
        name="API Service Instance Failure",
        description="Verify API service handles instance failure gracefully",
        hypothesis=hypothesis,
        action=action,
        duration_seconds=30
    )
    
    # Run it
    result = experiment.run(dry_run=True)
    print(json.dumps(result.to_dict(), indent=2))


def main():
    parser = argparse.ArgumentParser(description="Run chaos experiments")
    parser.add_argument("--config", help="Experiment configuration file (YAML)")
    parser.add_argument("--dry-run", action="store_true", help="Don't actually inject chaos")
    parser.add_argument("--demo", action="store_true", help="Run demo experiment")
    
    args = parser.parse_args()
    
    if args.demo:
        demo_experiment()
    elif args.config:
        # Load from config file
        print(f"Loading experiment from {args.config}")
        # Implementation would parse YAML and construct experiment
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
