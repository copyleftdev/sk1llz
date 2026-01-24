#!/usr/bin/env python3
"""
pythonic_check.py
Hettinger-style code checker for Pythonic patterns.

Usage:
    python pythonic_check.py <file.py>
    python pythonic_check.py --directory ./src/
"""

import argparse
import ast
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import List, Iterator


@dataclass
class Suggestion:
    """A suggestion for more Pythonic code."""
    file: str
    line: int
    category: str
    message: str
    original: str
    suggestion: str


class PythonicChecker(ast.NodeVisitor):
    """AST visitor that checks for non-Pythonic patterns."""
    
    def __init__(self, filename: str, source_lines: List[str]):
        self.filename = filename
        self.source_lines = source_lines
        self.suggestions: List[Suggestion] = []
    
    def add_suggestion(self, node, category: str, message: str, 
                       original: str, suggestion: str):
        self.suggestions.append(Suggestion(
            file=self.filename,
            line=node.lineno,
            category=category,
            message=message,
            original=original,
            suggestion=suggestion
        ))
    
    def visit_For(self, node):
        """Check for C-style iteration patterns."""
        # Check for `for i in range(len(x)):`
        if (isinstance(node.iter, ast.Call) and
            isinstance(node.iter.func, ast.Name) and
            node.iter.func.id == 'range' and
            len(node.iter.args) == 1):
            
            arg = node.iter.args[0]
            if (isinstance(arg, ast.Call) and
                isinstance(arg.func, ast.Name) and
                arg.func.id == 'len'):
                
                self.add_suggestion(
                    node,
                    "iteration",
                    "Use enumerate() instead of range(len())",
                    "for i in range(len(items)):",
                    "for i, item in enumerate(items):"
                )
        
        self.generic_visit(node)
    
    def visit_If(self, node):
        """Check for patterns that could use dict.get() or other idioms."""
        # Check for `if key in dict: ... else: ...` patterns
        if (isinstance(node.test, ast.Compare) and
            len(node.test.ops) == 1 and
            isinstance(node.test.ops[0], ast.In)):
            
            # Could potentially suggest dict.get() or setdefault()
            pass
        
        self.generic_visit(node)
    
    def visit_Call(self, node):
        """Check for function call anti-patterns."""
        # Check for manual dict construction that could use dict comprehension
        if (isinstance(node.func, ast.Attribute) and
            node.func.attr == 'append'):
            # Could be building a list that should be a comprehension
            pass
        
        self.generic_visit(node)
    
    def visit_Assign(self, node):
        """Check for assignment patterns."""
        # Check for `x = x + 1` that should be `x += 1`
        if (len(node.targets) == 1 and
            isinstance(node.targets[0], ast.Name) and
            isinstance(node.value, ast.BinOp)):
            
            target_name = node.targets[0].id
            if (isinstance(node.value.left, ast.Name) and
                node.value.left.id == target_name):
                
                op_map = {
                    ast.Add: '+=', ast.Sub: '-=',
                    ast.Mult: '*=', ast.Div: '/='
                }
                op_type = type(node.value.op)
                if op_type in op_map:
                    self.add_suggestion(
                        node,
                        "augmented_assignment",
                        f"Use augmented assignment {op_map[op_type]}",
                        f"{target_name} = {target_name} + ...",
                        f"{target_name} += ..."
                    )
        
        self.generic_visit(node)
    
    def visit_Compare(self, node):
        """Check comparison patterns."""
        # Check for `type(x) == SomeType` instead of isinstance()
        if (len(node.ops) == 1 and
            isinstance(node.ops[0], ast.Eq) and
            isinstance(node.left, ast.Call) and
            isinstance(node.left.func, ast.Name) and
            node.left.func.id == 'type'):
            
            self.add_suggestion(
                node,
                "type_check",
                "Use isinstance() for type checking",
                "type(x) == SomeType",
                "isinstance(x, SomeType)"
            )
        
        # Check for `x == None` instead of `x is None`
        if (len(node.ops) == 1 and
            isinstance(node.ops[0], ast.Eq) and
            len(node.comparators) == 1 and
            isinstance(node.comparators[0], ast.Constant) and
            node.comparators[0].value is None):
            
            self.add_suggestion(
                node,
                "none_comparison",
                "Use 'is None' instead of '== None'",
                "x == None",
                "x is None"
            )
        
        self.generic_visit(node)


def check_source_patterns(source: str, filename: str) -> List[Suggestion]:
    """Check source code for regex-based patterns."""
    suggestions = []
    lines = source.split('\n')
    
    patterns = [
        # Manual file handling
        (r'^\s*f\s*=\s*open\([^)]+\)\s*$', 
         "file_handling",
         "Use 'with open()' context manager",
         "f = open('file')",
         "with open('file') as f:"),
        
        # Empty list/dict literals for accumulation
        (r'^\s*\w+\s*=\s*\[\]\s*$.*\n.*\.append\(',
         "list_comprehension",
         "Consider list comprehension instead of append loop",
         "result = []; for x in y: result.append(z)",
         "result = [z for x in y]"),
        
        # String concatenation in loop
        (r'^\s*\w+\s*\+=\s*["\']',
         "string_concat",
         "Use join() for string concatenation in loops",
         "s += item",
         "s = ''.join(items)"),
    ]
    
    for i, line in enumerate(lines, 1):
        for pattern, category, message, original, suggestion in patterns:
            if re.search(pattern, line):
                suggestions.append(Suggestion(
                    file=filename,
                    line=i,
                    category=category,
                    message=message,
                    original=original,
                    suggestion=suggestion
                ))
    
    return suggestions


def analyze_file(filepath: Path) -> List[Suggestion]:
    """Analyze a Python file for Pythonic improvements."""
    try:
        source = filepath.read_text()
        tree = ast.parse(source)
    except SyntaxError as e:
        return [Suggestion(
            file=str(filepath),
            line=e.lineno or 0,
            category="syntax_error",
            message=f"Syntax error: {e.msg}",
            original="",
            suggestion=""
        )]
    
    lines = source.split('\n')
    checker = PythonicChecker(str(filepath), lines)
    checker.visit(tree)
    
    # Add regex-based checks
    source_suggestions = check_source_patterns(source, str(filepath))
    
    return checker.suggestions + source_suggestions


def analyze_directory(directory: Path) -> Iterator[Suggestion]:
    """Analyze all Python files in a directory."""
    for filepath in directory.rglob('*.py'):
        yield from analyze_file(filepath)


def print_report(suggestions: List[Suggestion]):
    """Print formatted report."""
    if not suggestions:
        print("✅ No Pythonic improvements suggested!")
        return
    
    print("\n" + "=" * 60)
    print("PYTHONIC CODE REVIEW (Hettinger Style)")
    print("=" * 60)
    
    by_category = {}
    for s in suggestions:
        by_category.setdefault(s.category, []).append(s)
    
    for category, items in sorted(by_category.items()):
        print(f"\n## {category.replace('_', ' ').title()} ({len(items)} issues)")
        print("-" * 40)
        
        for s in items:
            print(f"\n📍 {s.file}:{s.line}")
            print(f"   💡 {s.message}")
            if s.original:
                print(f"   ❌ {s.original}")
                print(f"   ✅ {s.suggestion}")
    
    print("\n" + "=" * 60)
    print(f"Total suggestions: {len(suggestions)}")
    print("\nRemember Hettinger's wisdom:")
    print("  'Beautiful is better than ugly.'")
    print("  'There should be one obvious way to do it.'")
    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(
        description="Check Python code for Pythonic patterns (Hettinger style)"
    )
    parser.add_argument("path", nargs="?", help="File or directory to check")
    parser.add_argument("--directory", "-d", help="Directory to check recursively")
    
    args = parser.parse_args()
    
    if args.directory:
        target = Path(args.directory)
        suggestions = list(analyze_directory(target))
    elif args.path:
        target = Path(args.path)
        if target.is_dir():
            suggestions = list(analyze_directory(target))
        else:
            suggestions = analyze_file(target)
    else:
        # Demo mode
        demo_code = '''
x = []
for i in range(len(items)):
    x.append(items[i] * 2)

if type(obj) == str:
    print(obj)

count = count + 1

if value == None:
    value = default
'''
        print("Demo mode - analyzing sample code:")
        print(demo_code)
        
        tree = ast.parse(demo_code)
        checker = PythonicChecker("<demo>", demo_code.split('\n'))
        checker.visit(tree)
        suggestions = checker.suggestions
    
    print_report(suggestions)


if __name__ == "__main__":
    main()
