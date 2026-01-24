#!/usr/bin/env python3
"""
pyramid_analyzer.py
Analyze detection rules against Bianco's Pyramid of Pain.

Usage:
    python pyramid_analyzer.py --rules rules.yaml
    python pyramid_analyzer.py --sigma-dir ./sigma-rules/
"""

import argparse
import os
import re
from dataclasses import dataclass
from enum import IntEnum
from pathlib import Path
from typing import Dict, List, Optional
import json

try:
    import yaml
    HAS_YAML = True
except ImportError:
    HAS_YAML = False


class PyramidLevel(IntEnum):
    """Pyramid of Pain levels - higher = more valuable."""
    HASH = 1
    IP_ADDRESS = 2
    DOMAIN = 3
    ARTIFACT = 4
    TOOL = 5
    TTP = 6


@dataclass
class DetectionRule:
    """A detection rule with pyramid classification."""
    
    name: str
    source_file: str
    level: PyramidLevel
    confidence: float  # 0-1, how confident in classification
    indicators: List[str]
    mitre_techniques: List[str]
    
    @property
    def adversary_pain(self) -> str:
        pain_map = {
            PyramidLevel.HASH: "Trivial",
            PyramidLevel.IP_ADDRESS: "Easy", 
            PyramidLevel.DOMAIN: "Simple",
            PyramidLevel.ARTIFACT: "Annoying",
            PyramidLevel.TOOL: "Challenging",
            PyramidLevel.TTP: "Tough!"
        }
        return pain_map[self.level]


class PyramidClassifier:
    """Classify detection rules by Pyramid of Pain level."""
    
    # Patterns indicating hash-based detection
    HASH_PATTERNS = [
        r'\b[a-fA-F0-9]{32}\b',  # MD5
        r'\b[a-fA-F0-9]{40}\b',  # SHA1
        r'\b[a-fA-F0-9]{64}\b',  # SHA256
        r'hash[_\s]*[:=]',
        r'file_hash',
        r'imphash',
    ]
    
    # Patterns indicating IP-based detection
    IP_PATTERNS = [
        r'\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b',
        r'ip_address',
        r'src_ip|dst_ip',
        r'remote_ip|source_ip',
    ]
    
    # Patterns indicating domain-based detection
    DOMAIN_PATTERNS = [
        r'\.com\b|\.net\b|\.org\b|\.ru\b|\.cn\b',
        r'domain[_\s]*[:=]',
        r'dns_query',
        r'hostname',
    ]
    
    # Patterns indicating artifact-based detection
    ARTIFACT_PATTERNS = [
        r'HKLM|HKCU|Registry',
        r'\\AppData\\',
        r'\\Windows\\Temp',
        r'mutex',
        r'user_agent',
        r'\\pipe\\',
    ]
    
    # Patterns indicating tool-based detection
    TOOL_PATTERNS = [
        r'mimikatz',
        r'cobalt\s*strike',
        r'metasploit',
        r'empire',
        r'bloodhound',
        r'psexec',
        r'procdump',
    ]
    
    # Patterns indicating TTP-based detection
    TTP_PATTERNS = [
        r'attack\.t\d{4}',
        r'CommandLine\|contains',
        r'process_creation',
        r'powershell.*download',
        r'invoke-expression|iex\s',
        r'credential.*dump',
        r'lateral.*movement',
    ]
    
    def classify(self, rule_content: str, rule_name: str = "") -> PyramidLevel:
        """Classify a rule by its highest-value indicator type."""
        content_lower = rule_content.lower()
        
        # Check from highest to lowest value (TTP -> Hash)
        if self._matches_any(content_lower, self.TTP_PATTERNS):
            return PyramidLevel.TTP
        
        if self._matches_any(content_lower, self.TOOL_PATTERNS):
            return PyramidLevel.TOOL
        
        if self._matches_any(content_lower, self.ARTIFACT_PATTERNS):
            return PyramidLevel.ARTIFACT
        
        if self._matches_any(content_lower, self.DOMAIN_PATTERNS):
            return PyramidLevel.DOMAIN
        
        if self._matches_any(content_lower, self.IP_PATTERNS):
            return PyramidLevel.IP_ADDRESS
        
        if self._matches_any(content_lower, self.HASH_PATTERNS):
            return PyramidLevel.HASH
        
        # Default to ARTIFACT if can't classify
        return PyramidLevel.ARTIFACT
    
    def _matches_any(self, content: str, patterns: List[str]) -> bool:
        return any(re.search(p, content, re.IGNORECASE) for p in patterns)
    
    def extract_mitre(self, content: str) -> List[str]:
        """Extract MITRE ATT&CK technique IDs."""
        pattern = r'[Tt]\d{4}(?:\.\d{3})?'
        return list(set(re.findall(pattern, content)))


class PyramidAnalyzer:
    """Analyze detection rules against the Pyramid of Pain."""
    
    def __init__(self):
        self.classifier = PyramidClassifier()
        self.rules: List[DetectionRule] = []
    
    def analyze_sigma_file(self, filepath: str) -> Optional[DetectionRule]:
        """Analyze a single Sigma rule file."""
        if not HAS_YAML:
            print("Warning: PyYAML not installed, parsing as text")
            return self._analyze_text_file(filepath)
        
        with open(filepath, 'r') as f:
            content = f.read()
        
        try:
            rule_data = yaml.safe_load(content)
            if not rule_data:
                return None
            
            name = rule_data.get('title', Path(filepath).stem)
            level = self.classifier.classify(content, name)
            mitre = self.classifier.extract_mitre(content)
            
            # Also check tags for MITRE
            tags = rule_data.get('tags', [])
            for tag in tags:
                if 'attack.t' in tag.lower():
                    tech = tag.split('.')[-1].upper()
                    if tech.startswith('T'):
                        mitre.append(tech)
            
            return DetectionRule(
                name=name,
                source_file=filepath,
                level=level,
                confidence=0.8,
                indicators=[],
                mitre_techniques=list(set(mitre))
            )
        except Exception as e:
            print(f"Error parsing {filepath}: {e}")
            return None
    
    def _analyze_text_file(self, filepath: str) -> Optional[DetectionRule]:
        """Fallback text-based analysis."""
        with open(filepath, 'r') as f:
            content = f.read()
        
        name = Path(filepath).stem
        level = self.classifier.classify(content, name)
        mitre = self.classifier.extract_mitre(content)
        
        return DetectionRule(
            name=name,
            source_file=filepath,
            level=level,
            confidence=0.6,
            indicators=[],
            mitre_techniques=mitre
        )
    
    def analyze_directory(self, directory: str) -> List[DetectionRule]:
        """Analyze all Sigma rules in a directory."""
        rules = []
        
        for root, _, files in os.walk(directory):
            for file in files:
                if file.endswith(('.yml', '.yaml')):
                    filepath = os.path.join(root, file)
                    rule = self.analyze_sigma_file(filepath)
                    if rule:
                        rules.append(rule)
        
        self.rules = rules
        return rules
    
    def generate_report(self) -> dict:
        """Generate pyramid analysis report."""
        if not self.rules:
            return {'error': 'No rules analyzed'}
        
        # Count by level
        level_counts = {level: 0 for level in PyramidLevel}
        for rule in self.rules:
            level_counts[rule.level] += 1
        
        # Calculate percentages
        total = len(self.rules)
        level_percentages = {
            level.name: {
                'count': count,
                'percentage': round((count / total) * 100, 1)
            }
            for level, count in level_counts.items()
        }
        
        # High-value ratio (TTP + Tools)
        high_value = level_counts[PyramidLevel.TTP] + level_counts[PyramidLevel.TOOL]
        high_value_ratio = round((high_value / total) * 100, 1)
        
        # Low-value ratio (Hash + IP)
        low_value = level_counts[PyramidLevel.HASH] + level_counts[PyramidLevel.IP_ADDRESS]
        low_value_ratio = round((low_value / total) * 100, 1)
        
        # Recommendations
        recommendations = []
        if level_percentages['TTP']['percentage'] < 30:
            recommendations.append("Increase TTP-based detections (target: >30%)")
        if level_percentages['HASH']['percentage'] > 20:
            recommendations.append("Reduce reliance on hash-based detection")
        if high_value_ratio < 40:
            recommendations.append("Focus on high-value detections (TTPs + Tools)")
        
        # Maturity assessment
        if high_value_ratio > 50:
            maturity = "Mature - Strong TTP focus"
        elif high_value_ratio > 30:
            maturity = "Developing - Building TTP coverage"
        else:
            maturity = "Immature - Over-reliant on IOCs"
        
        return {
            'total_rules': total,
            'levels': level_percentages,
            'high_value_ratio': f"{high_value_ratio}%",
            'low_value_ratio': f"{low_value_ratio}%",
            'maturity': maturity,
            'recommendations': recommendations
        }
    
    def print_report(self):
        """Print formatted report."""
        report = self.generate_report()
        
        print("\n" + "=" * 60)
        print("PYRAMID OF PAIN ANALYSIS")
        print("=" * 60)
        print(f"\nTotal Rules Analyzed: {report['total_rules']}")
        print(f"Maturity Assessment: {report['maturity']}")
        print(f"\nHigh-Value (TTPs + Tools): {report['high_value_ratio']}")
        print(f"Low-Value (Hashes + IPs): {report['low_value_ratio']}")
        
        print("\n" + "-" * 60)
        print("DISTRIBUTION BY PYRAMID LEVEL")
        print("-" * 60)
        
        # Print as ASCII pyramid
        levels = report['levels']
        max_width = 50
        
        for level in reversed(PyramidLevel):
            level_data = levels[level.name]
            bar_width = int((level_data['percentage'] / 100) * max_width)
            bar = "█" * bar_width
            print(f"{level.name:12} | {bar:<{max_width}} | {level_data['count']:3} ({level_data['percentage']:5.1f}%)")
        
        if report['recommendations']:
            print("\n" + "-" * 60)
            print("RECOMMENDATIONS")
            print("-" * 60)
            for rec in report['recommendations']:
                print(f"• {rec}")
        
        print("\n" + "=" * 60)


def main():
    parser = argparse.ArgumentParser(description="Analyze detections against Pyramid of Pain")
    parser.add_argument("--sigma-dir", help="Directory containing Sigma rules")
    parser.add_argument("--rules", help="YAML file with rule definitions")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    
    args = parser.parse_args()
    
    analyzer = PyramidAnalyzer()
    
    if args.sigma_dir:
        analyzer.analyze_directory(args.sigma_dir)
    elif args.rules:
        if HAS_YAML:
            with open(args.rules) as f:
                # Parse custom rules format
                pass
        else:
            print("PyYAML required for --rules option")
            return
    else:
        # Demo mode
        print("Demo mode - analyzing sample rules")
        
        # Create sample rules for demonstration
        sample_rules = [
            DetectionRule("Known Malware Hash", "hash.yml", PyramidLevel.HASH, 0.9, [], []),
            DetectionRule("C2 IP Block", "ip.yml", PyramidLevel.IP_ADDRESS, 0.9, [], []),
            DetectionRule("Malware Domain", "domain.yml", PyramidLevel.DOMAIN, 0.9, [], []),
            DetectionRule("Registry Persistence", "reg.yml", PyramidLevel.ARTIFACT, 0.9, [], ["T1547"]),
            DetectionRule("Cobalt Strike Beacon", "cs.yml", PyramidLevel.TOOL, 0.9, [], ["T1059"]),
            DetectionRule("LSASS Memory Access", "lsass.yml", PyramidLevel.TTP, 0.9, [], ["T1003.001"]),
            DetectionRule("PowerShell Download Cradle", "ps.yml", PyramidLevel.TTP, 0.9, [], ["T1059.001"]),
            DetectionRule("Suspicious Scheduled Task", "schtask.yml", PyramidLevel.TTP, 0.9, [], ["T1053"]),
        ]
        analyzer.rules = sample_rules
    
    if args.json:
        print(json.dumps(analyzer.generate_report(), indent=2))
    else:
        analyzer.print_report()


if __name__ == "__main__":
    main()
