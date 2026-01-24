#!/usr/bin/env python3
"""
error_budget_calculator.py
Google SRE-style error budget calculator and tracker.

Usage:
    python error_budget_calculator.py --slo 99.9 --window 30
    python error_budget_calculator.py --slo 99.9 --current-sli 99.85 --window 30
"""

import argparse
from dataclasses import dataclass
from datetime import timedelta
from typing import Optional


@dataclass
class ErrorBudgetReport:
    """Error budget calculation results."""
    
    slo_percent: float
    window_days: int
    error_budget_percent: float
    error_budget_minutes: float
    
    current_sli: Optional[float] = None
    budget_consumed_percent: Optional[float] = None
    budget_remaining_minutes: Optional[float] = None
    burn_rate: Optional[float] = None
    time_until_exhaustion: Optional[timedelta] = None
    status: str = "Unknown"
    
    def __str__(self) -> str:
        lines = [
            "=" * 50,
            "ERROR BUDGET REPORT",
            "=" * 50,
            f"SLO Target:        {self.slo_percent}%",
            f"Window:            {self.window_days} days",
            f"Error Budget:      {self.error_budget_percent:.4f}%",
            f"Budget (time):     {self._format_duration(self.error_budget_minutes)}",
        ]
        
        if self.current_sli is not None:
            lines.extend([
                "-" * 50,
                f"Current SLI:       {self.current_sli}%",
                f"Budget Consumed:   {self.budget_consumed_percent:.1f}%",
                f"Budget Remaining:  {self._format_duration(self.budget_remaining_minutes)}",
                f"Burn Rate:         {self.burn_rate:.1f}x",
                f"Status:            {self.status}",
            ])
            
            if self.time_until_exhaustion and self.burn_rate > 1:
                lines.append(
                    f"Time to Exhaust:   {self._format_duration(self.time_until_exhaustion.total_seconds() / 60)}"
                )
        
        lines.append("=" * 50)
        return "\n".join(lines)
    
    @staticmethod
    def _format_duration(minutes: float) -> str:
        """Format minutes as human-readable duration."""
        if minutes < 1:
            return f"{minutes * 60:.1f} seconds"
        elif minutes < 60:
            return f"{minutes:.1f} minutes"
        elif minutes < 1440:  # Less than a day
            return f"{minutes / 60:.1f} hours"
        else:
            return f"{minutes / 1440:.1f} days"


def calculate_error_budget(
    slo_percent: float,
    window_days: int,
    current_sli: Optional[float] = None
) -> ErrorBudgetReport:
    """
    Calculate error budget based on SLO.
    
    Args:
        slo_percent: Target SLO as percentage (e.g., 99.9)
        window_days: Rolling window in days
        current_sli: Current measured SLI (optional)
    
    Returns:
        ErrorBudgetReport with calculations
    """
    # Basic calculations
    error_budget_percent = 100 - slo_percent
    window_minutes = window_days * 24 * 60
    error_budget_minutes = window_minutes * (error_budget_percent / 100)
    
    report = ErrorBudgetReport(
        slo_percent=slo_percent,
        window_days=window_days,
        error_budget_percent=error_budget_percent,
        error_budget_minutes=error_budget_minutes,
    )
    
    if current_sli is not None:
        # Current performance calculations
        current_error_percent = 100 - current_sli
        
        # Budget consumption
        budget_consumed_percent = (current_error_percent / error_budget_percent) * 100
        budget_remaining_minutes = error_budget_minutes * (1 - budget_consumed_percent / 100)
        
        # Burn rate
        expected_burn_rate = 1.0  # Normal rate
        actual_burn_rate = current_error_percent / error_budget_percent * window_days
        burn_rate = actual_burn_rate / expected_burn_rate if expected_burn_rate > 0 else 0
        
        # Time until exhaustion at current rate
        if burn_rate > 1 and budget_remaining_minutes > 0:
            days_until_exhaustion = budget_remaining_minutes / (
                window_minutes * current_error_percent / 100 / window_days
            )
            time_until_exhaustion = timedelta(days=days_until_exhaustion)
        else:
            time_until_exhaustion = None
        
        # Status determination
        budget_remaining_percent = 100 - budget_consumed_percent
        if budget_remaining_percent > 50:
            status = "✅ HEALTHY - Safe to ship features"
        elif budget_remaining_percent > 25:
            status = "⚠️  WARNING - Review recent changes"
        elif budget_remaining_percent > 0:
            status = "🔴 CRITICAL - Focus on reliability"
        else:
            status = "🚨 EXHAUSTED - Feature freeze required"
        
        report.current_sli = current_sli
        report.budget_consumed_percent = budget_consumed_percent
        report.budget_remaining_minutes = max(0, budget_remaining_minutes)
        report.burn_rate = burn_rate
        report.time_until_exhaustion = time_until_exhaustion
        report.status = status
    
    return report


def print_slo_reference_table():
    """Print reference table of common SLOs."""
    print("\nSLO Reference Table (30-day window)")
    print("-" * 60)
    print(f"{'SLO':<10} {'Error Budget':<15} {'Downtime/Month':<20}")
    print("-" * 60)
    
    slos = [99, 99.5, 99.9, 99.95, 99.99, 99.999]
    for slo in slos:
        error_budget = 100 - slo
        minutes = 30 * 24 * 60 * (error_budget / 100)
        
        if minutes >= 60:
            downtime = f"{minutes / 60:.1f} hours"
        else:
            downtime = f"{minutes:.1f} minutes"
        
        print(f"{slo}%{'':<5} {error_budget:.3f}%{'':<10} {downtime}")
    
    print("-" * 60)


def main():
    parser = argparse.ArgumentParser(
        description="Calculate SRE error budgets",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --slo 99.9 --window 30
  %(prog)s --slo 99.9 --current-sli 99.85 --window 30
  %(prog)s --reference
        """
    )
    
    parser.add_argument(
        "--slo",
        type=float,
        help="Target SLO percentage (e.g., 99.9)"
    )
    parser.add_argument(
        "--window",
        type=int,
        default=30,
        help="Rolling window in days (default: 30)"
    )
    parser.add_argument(
        "--current-sli",
        type=float,
        help="Current measured SLI percentage"
    )
    parser.add_argument(
        "--reference",
        action="store_true",
        help="Print SLO reference table"
    )
    
    args = parser.parse_args()
    
    if args.reference:
        print_slo_reference_table()
        return
    
    if not args.slo:
        parser.print_help()
        return
    
    report = calculate_error_budget(
        slo_percent=args.slo,
        window_days=args.window,
        current_sli=args.current_sli
    )
    
    print(report)


if __name__ == "__main__":
    main()
