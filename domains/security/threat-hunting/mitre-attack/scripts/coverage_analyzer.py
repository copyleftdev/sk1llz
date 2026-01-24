#!/usr/bin/env python3
"""
coverage_analyzer.py
Analyze detection coverage against MITRE ATT&CK framework.

Usage:
    python coverage_analyzer.py --rules rules_dir/ --output report.json
    python coverage_analyzer.py --demo
"""

import argparse
import json
import re
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Set, Optional


# MITRE ATT&CK Tactics and their IDs
TACTICS = {
    "TA0001": "Initial Access",
    "TA0002": "Execution",
    "TA0003": "Persistence",
    "TA0004": "Privilege Escalation",
    "TA0005": "Defense Evasion",
    "TA0006": "Credential Access",
    "TA0007": "Discovery",
    "TA0008": "Lateral Movement",
    "TA0009": "Collection",
    "TA0010": "Exfiltration",
    "TA0011": "Command and Control",
    "TA0040": "Impact",
}

# High-priority techniques per tactic (subset for demo)
HIGH_PRIORITY_TECHNIQUES = {
    "TA0001": ["T1566", "T1190", "T1133"],  # Phishing, Exploit, External Services
    "TA0002": ["T1059", "T1204", "T1053"],  # Scripting, User Execution, Scheduled Task
    "TA0003": ["T1547", "T1053", "T1543"],  # Boot Autostart, Scheduled Task, Create Service
    "TA0004": ["T1548", "T1055", "T1134"],  # Abuse Elevation, Process Injection, Token Manipulation
    "TA0005": ["T1070", "T1027", "T1562"],  # Indicator Removal, Obfuscation, Impair Defenses
    "TA0006": ["T1003", "T1110", "T1558"],  # OS Credential Dumping, Brute Force, Kerberos
    "TA0007": ["T1087", "T1082", "T1083"],  # Account Discovery, System Info, File Discovery
    "TA0008": ["T1021", "T1570", "T1072"],  # Remote Services, Lateral Tool Transfer
    "TA0009": ["T1560", "T1005", "T1074"],  # Archive Data, Local Data, Data Staged
    "TA0010": ["T1041", "T1567", "T1048"],  # Exfil Over C2, Web Service, Alt Protocol
    "TA0011": ["T1071", "T1095", "T1573"],  # App Layer Protocol, Non-App Protocol, Encrypted
    "TA0040": ["T1486", "T1490", "T1489"],  # Data Encrypted, Inhibit Recovery, Service Stop
}


@dataclass
class DetectionRule:
    """Represents a detection rule."""
    name: str
    path: str
    techniques: List[str] = field(default_factory=list)
    tactics: List[str] = field(default_factory=list)
    
    
@dataclass  
class CoverageReport:
    """Coverage analysis report."""
    total_rules: int
    techniques_covered: Set[str]
    tactics_coverage: Dict[str, int]
    high_priority_coverage: Dict[str, List[str]]
    gaps: Dict[str, List[str]]
    rules_by_technique: Dict[str, List[str]]


def parse_sigma_rule(path: Path) -> Optional[DetectionRule]:
    """Parse a Sigma rule file for ATT&CK mappings."""
    try:
        content = path.read_text()
        
        rule = DetectionRule(
            name=path.stem,
            path=str(path)
        )
        
        # Extract title
        title_match = re.search(r'^title:\s*(.+)$', content, re.MULTILINE)
        if title_match:
            rule.name = title_match.group(1).strip()
        
        # Extract ATT&CK technique IDs (T1xxx format)
        technique_pattern = r'attack\.t(\d{4})(?:\.(\d{3}))?'
        for match in re.finditer(technique_pattern, content, re.IGNORECASE):
            technique_id = f"T{match.group(1)}"
            if match.group(2):
                technique_id += f".{match.group(2)}"
            rule.techniques.append(technique_id)
        
        # Also check for explicit technique references
        explicit_pattern = r'T(\d{4})(?:\.(\d{3}))?'
        for match in re.finditer(explicit_pattern, content):
            technique_id = f"T{match.group(1)}"
            if match.group(2):
                technique_id += f".{match.group(2)}"
            if technique_id not in rule.techniques:
                rule.techniques.append(technique_id)
        
        # Extract tactics
        tactic_pattern = r'attack\.(\w+)'
        for match in re.finditer(tactic_pattern, content, re.IGNORECASE):
            tactic = match.group(1).lower()
            if tactic not in ['t' + str(i) for i in range(10000)]:  # Skip technique refs
                rule.tactics.append(tactic)
        
        return rule if rule.techniques else None
        
    except Exception as e:
        print(f"Error parsing {path}: {e}")
        return None


def analyze_coverage(rules: List[DetectionRule]) -> CoverageReport:
    """Analyze coverage across rules."""
    techniques_covered = set()
    tactics_coverage = defaultdict(int)
    rules_by_technique = defaultdict(list)
    
    for rule in rules:
        for technique in rule.techniques:
            base_technique = technique.split('.')[0]
            techniques_covered.add(base_technique)
            rules_by_technique[base_technique].append(rule.name)
        
        for tactic in rule.tactics:
            tactics_coverage[tactic] += 1
    
    # Calculate high-priority coverage
    high_priority_coverage = {}
    gaps = {}
    
    for tactic_id, techniques in HIGH_PRIORITY_TECHNIQUES.items():
        covered = []
        missing = []
        
        for tech in techniques:
            if tech in techniques_covered:
                covered.append(tech)
            else:
                missing.append(tech)
        
        high_priority_coverage[tactic_id] = covered
        if missing:
            gaps[tactic_id] = missing
    
    return CoverageReport(
        total_rules=len(rules),
        techniques_covered=techniques_covered,
        tactics_coverage=dict(tactics_coverage),
        high_priority_coverage=high_priority_coverage,
        gaps=gaps,
        rules_by_technique=dict(rules_by_technique)
    )


def format_report(report: CoverageReport) -> str:
    """Format coverage report as text."""
    output = []
    output.append("=" * 60)
    output.append("MITRE ATT&CK COVERAGE ANALYSIS")
    output.append("=" * 60)
    output.append("")
    
    output.append(f"Total Rules Analyzed: {report.total_rules}")
    output.append(f"Unique Techniques Covered: {len(report.techniques_covered)}")
    output.append("")
    
    # Tactic coverage
    output.append("## Coverage by Tactic")
    output.append("-" * 40)
    for tactic_id, tactic_name in TACTICS.items():
        high_pri = HIGH_PRIORITY_TECHNIQUES.get(tactic_id, [])
        covered = report.high_priority_coverage.get(tactic_id, [])
        pct = (len(covered) / len(high_pri) * 100) if high_pri else 0
        bar = "█" * int(pct / 10) + "░" * (10 - int(pct / 10))
        output.append(f"{tactic_name:25} [{bar}] {pct:5.1f}% ({len(covered)}/{len(high_pri)} high-pri)")
    
    output.append("")
    
    # Gaps
    output.append("## Coverage Gaps (High Priority)")
    output.append("-" * 40)
    for tactic_id, missing in report.gaps.items():
        tactic_name = TACTICS.get(tactic_id, tactic_id)
        output.append(f"\n{tactic_name}:")
        for tech in missing:
            output.append(f"  ⚠ {tech} - NOT COVERED")
    
    output.append("")
    
    # Techniques with most rules
    output.append("## Most Covered Techniques")
    output.append("-" * 40)
    sorted_tech = sorted(
        report.rules_by_technique.items(),
        key=lambda x: len(x[1]),
        reverse=True
    )[:10]
    for tech, rules in sorted_tech:
        output.append(f"{tech}: {len(rules)} rules")
    
    output.append("")
    output.append("=" * 60)
    
    # Overall score
    total_high_pri = sum(len(t) for t in HIGH_PRIORITY_TECHNIQUES.values())
    total_covered = sum(len(c) for c in report.high_priority_coverage.values())
    overall_pct = (total_covered / total_high_pri * 100) if total_high_pri else 0
    
    output.append(f"Overall High-Priority Coverage: {overall_pct:.1f}%")
    
    if overall_pct >= 80:
        output.append("★★★★★ Excellent coverage")
    elif overall_pct >= 60:
        output.append("★★★★☆ Good coverage")
    elif overall_pct >= 40:
        output.append("★★★☆☆ Moderate coverage - gaps exist")
    elif overall_pct >= 20:
        output.append("★★☆☆☆ Limited coverage - significant gaps")
    else:
        output.append("★☆☆☆☆ Minimal coverage - major investment needed")
    
    return "\n".join(output)


def demo():
    """Run demo analysis with sample rules."""
    print("\n=== MITRE ATT&CK COVERAGE ANALYZER DEMO ===\n")
    
    # Create sample rules
    sample_rules = [
        DetectionRule("PowerShell Download Cradle", "rules/exec/powershell_download.yml",
                     techniques=["T1059.001", "T1105"], tactics=["execution"]),
        DetectionRule("Mimikatz Credential Dump", "rules/cred/mimikatz.yml",
                     techniques=["T1003.001"], tactics=["credential_access"]),
        DetectionRule("Scheduled Task Creation", "rules/persist/schtask.yml",
                     techniques=["T1053.005"], tactics=["persistence", "execution"]),
        DetectionRule("Registry Run Key", "rules/persist/runkey.yml",
                     techniques=["T1547.001"], tactics=["persistence"]),
        DetectionRule("LSASS Access", "rules/cred/lsass.yml",
                     techniques=["T1003.001"], tactics=["credential_access"]),
        DetectionRule("DNS Tunneling", "rules/c2/dns_tunnel.yml",
                     techniques=["T1071.004"], tactics=["command_and_control"]),
        DetectionRule("Lateral Movement via SMB", "rules/lateral/smb.yml",
                     techniques=["T1021.002"], tactics=["lateral_movement"]),
        DetectionRule("Data Archive", "rules/exfil/archive.yml",
                     techniques=["T1560.001"], tactics=["collection"]),
    ]
    
    report = analyze_coverage(sample_rules)
    print(format_report(report))
    
    # Also output as JSON
    print("\n--- JSON Output ---")
    json_report = {
        "total_rules": report.total_rules,
        "techniques_covered": list(report.techniques_covered),
        "gaps": report.gaps,
        "high_priority_coverage": report.high_priority_coverage
    }
    print(json.dumps(json_report, indent=2))


def main():
    parser = argparse.ArgumentParser(description="MITRE ATT&CK Coverage Analyzer")
    parser.add_argument("--rules", "-r", help="Directory containing detection rules")
    parser.add_argument("--output", "-o", help="Output file for JSON report")
    parser.add_argument("--demo", action="store_true", help="Run demo analysis")
    
    args = parser.parse_args()
    
    if args.demo:
        demo()
        return
    
    if not args.rules:
        parser.print_help()
        return
    
    rules_dir = Path(args.rules)
    if not rules_dir.exists():
        print(f"Error: Directory not found: {args.rules}")
        return
    
    # Parse all YAML files
    rules = []
    for yaml_file in rules_dir.rglob("*.yml"):
        rule = parse_sigma_rule(yaml_file)
        if rule:
            rules.append(rule)
    
    for yaml_file in rules_dir.rglob("*.yaml"):
        rule = parse_sigma_rule(yaml_file)
        if rule:
            rules.append(rule)
    
    if not rules:
        print("No rules with ATT&CK mappings found")
        return
    
    report = analyze_coverage(rules)
    print(format_report(report))
    
    if args.output:
        json_report = {
            "total_rules": report.total_rules,
            "techniques_covered": list(report.techniques_covered),
            "gaps": report.gaps,
            "high_priority_coverage": report.high_priority_coverage,
            "rules_by_technique": report.rules_by_technique
        }
        Path(args.output).write_text(json.dumps(json_report, indent=2))
        print(f"\nJSON report written to: {args.output}")


if __name__ == "__main__":
    main()
