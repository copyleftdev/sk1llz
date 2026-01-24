#!/usr/bin/env python3
"""
sigma_converter.py
Convert Sigma rules to various SIEM query formats.

Usage:
    python sigma_converter.py --rule rule.yml --target splunk
    python sigma_converter.py --rule rule.yml --target elastic
"""

import argparse
import re
from dataclasses import dataclass
from typing import Dict, List, Any, Optional
from pathlib import Path

try:
    import yaml
    HAS_YAML = True
except ImportError:
    HAS_YAML = False


@dataclass
class SigmaRule:
    """Parsed Sigma rule."""
    title: str
    description: str
    logsource: Dict[str, str]
    detection: Dict[str, Any]
    level: str
    tags: List[str]
    
    @classmethod
    def from_dict(cls, data: dict) -> 'SigmaRule':
        return cls(
            title=data.get('title', 'Untitled'),
            description=data.get('description', ''),
            logsource=data.get('logsource', {}),
            detection=data.get('detection', {}),
            level=data.get('level', 'medium'),
            tags=data.get('tags', [])
        )


class SigmaConverter:
    """Base converter for Sigma rules."""
    
    def __init__(self, rule: SigmaRule):
        self.rule = rule
    
    def convert(self) -> str:
        raise NotImplementedError
    
    def _parse_condition(self, condition: str) -> str:
        """Parse Sigma condition into target format."""
        raise NotImplementedError
    
    def _convert_selection(self, name: str, selection: Dict) -> str:
        """Convert a selection block."""
        raise NotImplementedError


class SplunkConverter(SigmaConverter):
    """Convert Sigma to Splunk SPL."""
    
    # Field mapping from Sigma to Splunk CIM
    FIELD_MAP = {
        'Image': 'process_path',
        'CommandLine': 'process',
        'ParentImage': 'parent_process_path',
        'User': 'user',
        'Computer': 'dest',
        'TargetFilename': 'file_path',
        'DestinationIp': 'dest_ip',
        'DestinationPort': 'dest_port',
        'SourceIp': 'src_ip',
    }
    
    def convert(self) -> str:
        """Convert Sigma rule to Splunk SPL."""
        lines = [f'`# {self.rule.title}`']
        lines.append(f'`# Level: {self.rule.level}`')
        lines.append('')
        
        # Build index/sourcetype from logsource
        index_clause = self._get_index_clause()
        lines.append(index_clause)
        
        # Build search from detection
        search_clauses = []
        condition = self.rule.detection.get('condition', '')
        
        for key, value in self.rule.detection.items():
            if key == 'condition':
                continue
            if isinstance(value, dict):
                clause = self._convert_selection(key, value)
                search_clauses.append((key, clause))
        
        # Apply condition logic
        query = self._apply_condition(condition, search_clauses)
        lines.append(query)
        
        return '\n'.join(lines)
    
    def _get_index_clause(self) -> str:
        """Generate index and sourcetype clause."""
        logsource = self.rule.logsource
        
        if logsource.get('product') == 'windows':
            if logsource.get('service') == 'sysmon':
                return 'index=windows sourcetype="XmlWinEventLog:Microsoft-Windows-Sysmon/Operational"'
            elif logsource.get('service') == 'security':
                return 'index=windows sourcetype="WinEventLog:Security"'
            elif logsource.get('category') == 'process_creation':
                return 'index=windows (sourcetype="XmlWinEventLog:Microsoft-Windows-Sysmon/Operational" EventCode=1)'
        
        return 'index=* '
    
    def _convert_selection(self, name: str, selection: Dict) -> str:
        """Convert selection to SPL clause."""
        clauses = []
        
        for field, value in selection.items():
            # Handle modifiers
            field_name, *modifiers = field.split('|')
            splunk_field = self.FIELD_MAP.get(field_name, field_name)
            
            if isinstance(value, list):
                # OR across values
                value_clauses = []
                for v in value:
                    value_clauses.append(self._format_value(splunk_field, v, modifiers))
                clauses.append(f'({" OR ".join(value_clauses)})')
            else:
                clauses.append(self._format_value(splunk_field, value, modifiers))
        
        # Check if 'all' modifier means AND
        if any('all' in field.split('|') for field in selection.keys()):
            return ' '.join(clauses)
        
        return ' '.join(clauses)
    
    def _format_value(self, field: str, value: str, modifiers: List[str]) -> str:
        """Format a field=value clause with modifiers."""
        if 'contains' in modifiers:
            return f'{field}="*{value}*"'
        elif 'startswith' in modifiers:
            return f'{field}="{value}*"'
        elif 'endswith' in modifiers:
            return f'{field}="*{value}"'
        elif 're' in modifiers:
            return f'| regex {field}="{value}"'
        else:
            return f'{field}="{value}"'
    
    def _apply_condition(self, condition: str, clauses: List[tuple]) -> str:
        """Apply condition logic to clauses."""
        if not condition:
            return ' AND '.join(c[1] for c in clauses)
        
        result = condition
        for name, clause in clauses:
            result = result.replace(name, f'({clause})')
        
        result = result.replace(' and ', ' AND ')
        result = result.replace(' or ', ' OR ')
        result = result.replace(' not ', ' NOT ')
        
        return result


class ElasticConverter(SigmaConverter):
    """Convert Sigma to Elasticsearch Query DSL / Lucene."""
    
    FIELD_MAP = {
        'Image': 'process.executable',
        'CommandLine': 'process.command_line',
        'ParentImage': 'process.parent.executable',
        'User': 'user.name',
        'Computer': 'host.name',
        'TargetFilename': 'file.path',
        'DestinationIp': 'destination.ip',
        'DestinationPort': 'destination.port',
        'SourceIp': 'source.ip',
    }
    
    def convert(self) -> str:
        """Convert Sigma rule to Elasticsearch Lucene query."""
        lines = [f'// {self.rule.title}']
        lines.append(f'// Level: {self.rule.level}')
        lines.append('')
        
        # Build query from detection
        search_clauses = []
        condition = self.rule.detection.get('condition', '')
        
        for key, value in self.rule.detection.items():
            if key == 'condition':
                continue
            if isinstance(value, dict):
                clause = self._convert_selection(key, value)
                search_clauses.append((key, clause))
        
        query = self._apply_condition(condition, search_clauses)
        lines.append(query)
        
        return '\n'.join(lines)
    
    def _convert_selection(self, name: str, selection: Dict) -> str:
        """Convert selection to Lucene clause."""
        clauses = []
        
        for field, value in selection.items():
            field_name, *modifiers = field.split('|')
            es_field = self.FIELD_MAP.get(field_name, field_name)
            
            if isinstance(value, list):
                value_clauses = []
                for v in value:
                    value_clauses.append(self._format_value(es_field, v, modifiers))
                clauses.append(f'({" OR ".join(value_clauses)})')
            else:
                clauses.append(self._format_value(es_field, value, modifiers))
        
        return ' AND '.join(clauses)
    
    def _format_value(self, field: str, value: str, modifiers: List[str]) -> str:
        """Format a field:value clause with modifiers."""
        # Escape special Lucene characters
        value = re.sub(r'([+\-&|!(){}[\]^"~*?:\\/])', r'\\\1', str(value))
        
        if 'contains' in modifiers:
            return f'{field}:*{value}*'
        elif 'startswith' in modifiers:
            return f'{field}:{value}*'
        elif 'endswith' in modifiers:
            return f'{field}:*{value}'
        elif 're' in modifiers:
            return f'{field}:/{value}/'
        else:
            return f'{field}:"{value}"'
    
    def _apply_condition(self, condition: str, clauses: List[tuple]) -> str:
        """Apply condition logic to clauses."""
        if not condition:
            return ' AND '.join(c[1] for c in clauses)
        
        result = condition
        for name, clause in clauses:
            result = result.replace(name, f'({clause})')
        
        result = result.replace(' and ', ' AND ')
        result = result.replace(' or ', ' OR ')
        result = result.replace(' not ', ' NOT ')
        
        return result


CONVERTERS = {
    'splunk': SplunkConverter,
    'elastic': ElasticConverter,
    'elasticsearch': ElasticConverter,
}


def convert_rule(rule_path: str, target: str) -> str:
    """Convert a Sigma rule file to target format."""
    if not HAS_YAML:
        raise RuntimeError("PyYAML required: pip install pyyaml")
    
    with open(rule_path) as f:
        data = yaml.safe_load(f)
    
    rule = SigmaRule.from_dict(data)
    
    converter_class = CONVERTERS.get(target.lower())
    if not converter_class:
        raise ValueError(f"Unknown target: {target}. Available: {list(CONVERTERS.keys())}")
    
    converter = converter_class(rule)
    return converter.convert()


def demo():
    """Demo with a sample rule."""
    sample_rule = {
        'title': 'Suspicious PowerShell Download',
        'description': 'Detects PowerShell download cradles',
        'logsource': {
            'category': 'process_creation',
            'product': 'windows'
        },
        'detection': {
            'selection_process': {
                'Image|endswith': ['\\powershell.exe', '\\pwsh.exe']
            },
            'selection_cmdline': {
                'CommandLine|contains|all': ['Invoke-', 'http']
            },
            'filter': {
                'User': 'SYSTEM'
            },
            'condition': '(selection_process and selection_cmdline) and not filter'
        },
        'level': 'high',
        'tags': ['attack.execution', 'attack.t1059.001']
    }
    
    rule = SigmaRule.from_dict(sample_rule)
    
    print("=" * 60)
    print("SIGMA RULE CONVERSION DEMO")
    print("=" * 60)
    print(f"\nRule: {rule.title}")
    print(f"Level: {rule.level}")
    print(f"Tags: {', '.join(rule.tags)}")
    
    print("\n" + "-" * 60)
    print("SPLUNK SPL:")
    print("-" * 60)
    splunk = SplunkConverter(rule)
    print(splunk.convert())
    
    print("\n" + "-" * 60)
    print("ELASTICSEARCH LUCENE:")
    print("-" * 60)
    elastic = ElasticConverter(rule)
    print(elastic.convert())
    
    print("\n" + "=" * 60)


def main():
    parser = argparse.ArgumentParser(description="Convert Sigma rules to SIEM formats")
    parser.add_argument("--rule", "-r", help="Path to Sigma rule YAML file")
    parser.add_argument("--target", "-t", choices=list(CONVERTERS.keys()),
                       help="Target SIEM format")
    parser.add_argument("--demo", action="store_true", help="Run demo conversion")
    
    args = parser.parse_args()
    
    if args.demo:
        demo()
    elif args.rule and args.target:
        result = convert_rule(args.rule, args.target)
        print(result)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
