---
name: lipton-mutation-testing
description: Evaluate test quality in the style of Richard Lipton, father of mutation testing. Emphasizes injecting small faults (mutants) to measure test effectiveness, the competent programmer hypothesis, and the coupling effect. Use when assessing test suite quality, improving test coverage, or building mutation testing tools.
---

# Richard Lipton Mutation Testing Style Guide

## Overview

Richard Lipton is the father of mutation testing, introducing the concept in the early 1970s. His foundational 1978 paper "Hints on Test Data Selection: Help for the Practicing Programmer" (with DeMillo and Sayward) established the theoretical basis for evaluating test quality. The core insight: if your tests can't detect small, simple faults (mutants), they certainly won't detect complex real bugs.

## Core Philosophy

> "If a test suite cannot detect a simple fault, it will not detect a complex one."

> "Good tests kill mutants. Surviving mutants reveal test weaknesses."

> "The mutation score is the only honest metric of test effectiveness."

Mutation testing inverts the question from "does my code pass tests?" to "do my tests actually detect faults?" By systematically injecting small bugs and measuring how many your tests catch, you get an objective measure of test quality that coverage metrics cannot provide.

## Design Principles

1. **Competent Programmer Hypothesis**: Real bugs are small deviations from correct code.

2. **Coupling Effect**: Tests that detect simple faults will detect complex ones.

3. **Mutation Score**: The percentage of killed mutants measures test effectiveness.

4. **Equivalent Mutants**: Some mutants don't change behavior—identify and exclude them.

5. **Mutation Operators**: Systematic rules for generating meaningful mutations.

## Mutation Testing Process

```
┌─────────────────────────────────────────────────────────────┐
│                  MUTATION TESTING PROCESS                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. ORIGINAL CODE                                            │
│     def is_adult(age):                                       │
│         return age >= 18                                     │
│                                                              │
│                    │                                         │
│                    ▼                                         │
│                                                              │
│  2. GENERATE MUTANTS (apply mutation operators)              │
│                                                              │
│     Mutant 1: return age > 18    (>= → >)                   │
│     Mutant 2: return age <= 18   (>= → <=)                  │
│     Mutant 3: return age >= 17   (18 → 17)                  │
│     Mutant 4: return age >= 19   (18 → 19)                  │
│     Mutant 5: return True        (replace expression)       │
│                                                              │
│                    │                                         │
│                    ▼                                         │
│                                                              │
│  3. RUN TESTS AGAINST EACH MUTANT                           │
│                                                              │
│     Mutant 1: KILLED (test_age_18 failed)                   │
│     Mutant 2: KILLED (test_age_20 failed)                   │
│     Mutant 3: KILLED (test_age_18 failed)                   │
│     Mutant 4: SURVIVED ← Test gap found!                    │
│     Mutant 5: KILLED (test_age_10 failed)                   │
│                                                              │
│                    │                                         │
│                    ▼                                         │
│                                                              │
│  4. CALCULATE MUTATION SCORE                                 │
│                                                              │
│     Killed: 4 / Total: 5 = 80% mutation score               │
│                                                              │
│  5. IMPROVE TESTS (to kill survivors)                       │
│                                                              │
│     Add: test_age_19() → asserts is_adult(19) == True      │
│     Re-run: Mutant 4 now KILLED                             │
│     New score: 100%                                          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Mutation Operators

### Arithmetic Operator Replacement (AOR)

```python
# Original
result = a + b

# Mutants
result = a - b    # + → -
result = a * b    # + → *
result = a / b    # + → /
result = a % b    # + → %
result = a ** b   # + → **
```

### Relational Operator Replacement (ROR)

```python
# Original
if x >= y:

# Mutants
if x > y:     # >= → >
if x <= y:    # >= → <=
if x < y:     # >= → <
if x == y:    # >= → ==
if x != y:    # >= → !=
if True:      # replace with True
if False:     # replace with False
```

### Conditional Operator Replacement (COR)

```python
# Original
if a and b:

# Mutants
if a or b:      # and → or
if a:           # remove b
if b:           # remove a
if True:        # always true
if False:       # always false
```

### Statement Deletion (SDL)

```python
# Original
def process(x):
    validate(x)
    result = compute(x)
    log(result)
    return result

# Mutants (delete each statement)
def process(x):
    # validate(x)  ← deleted
    result = compute(x)
    log(result)
    return result

def process(x):
    validate(x)
    result = compute(x)
    # log(result)  ← deleted
    return result
```

### Constant Replacement (CR)

```python
# Original
TIMEOUT = 30
MAX_RETRIES = 3

# Mutants
TIMEOUT = 0      # boundary
TIMEOUT = 31     # off by one
TIMEOUT = -30    # sign change
MAX_RETRIES = 0  # boundary
MAX_RETRIES = 2  # off by one
MAX_RETRIES = 4  # off by one
```

## When Applying Mutation Testing

### Always

- Run mutation testing on critical code paths
- Kill surviving mutants with targeted tests
- Track mutation score over time
- Identify equivalent mutants (no behavioral change)
- Use mutation testing to validate test refactoring
- Focus on boundary conditions and edge cases

### Never

- Aim for 100% blindly (equivalent mutants exist)
- Ignore surviving mutants in critical code
- Confuse mutation score with code coverage
- Run without timeout (infinite loop mutants)
- Mutate test code (only production code)
- Skip analysis of why mutants survived

### Prefer

- Mutation score over line coverage
- Targeted mutations over exhaustive generation
- Analyzing survivors over just counting kills
- Boundary mutation operators first
- Testing critical paths with high mutation score
- CI integration for regression

## Code Patterns

### Mutation Testing Framework

```python
import ast
import copy
from typing import List, Callable, Tuple
from dataclasses import dataclass
from enum import Enum

class MutantStatus(Enum):
    KILLED = "killed"
    SURVIVED = "survived"
    TIMEOUT = "timeout"
    ERROR = "error"
    EQUIVALENT = "equivalent"


@dataclass
class Mutant:
    id: int
    operator: str
    original: str
    mutated: str
    location: Tuple[int, int]  # line, column
    status: MutantStatus = None
    killing_test: str = None


@dataclass
class MutationResult:
    total_mutants: int
    killed: int
    survived: int
    timeout: int
    equivalent: int
    mutation_score: float
    survivors: List[Mutant]


class MutationOperator:
    """Base class for mutation operators."""
    
    name: str = "base"
    
    def mutate(self, node: ast.AST) -> List[ast.AST]:
        """Generate mutated versions of the node."""
        raise NotImplementedError


class ArithmeticOperatorReplacement(MutationOperator):
    """Replace arithmetic operators: + - * / % **"""
    
    name = "AOR"
    
    OPERATORS = {
        ast.Add: [ast.Sub, ast.Mult, ast.Div, ast.Mod],
        ast.Sub: [ast.Add, ast.Mult, ast.Div, ast.Mod],
        ast.Mult: [ast.Add, ast.Sub, ast.Div, ast.Mod],
        ast.Div: [ast.Add, ast.Sub, ast.Mult, ast.Mod],
        ast.Mod: [ast.Add, ast.Sub, ast.Mult, ast.Div],
    }
    
    def mutate(self, node: ast.BinOp) -> List[ast.BinOp]:
        if type(node.op) not in self.OPERATORS:
            return []
        
        mutants = []
        for replacement_op in self.OPERATORS[type(node.op)]:
            mutant = copy.deepcopy(node)
            mutant.op = replacement_op()
            mutants.append(mutant)
        
        return mutants


class RelationalOperatorReplacement(MutationOperator):
    """Replace relational operators: < <= > >= == !="""
    
    name = "ROR"
    
    OPERATORS = {
        ast.Lt: [ast.LtE, ast.Gt, ast.GtE, ast.Eq, ast.NotEq],
        ast.LtE: [ast.Lt, ast.Gt, ast.GtE, ast.Eq, ast.NotEq],
        ast.Gt: [ast.Lt, ast.LtE, ast.GtE, ast.Eq, ast.NotEq],
        ast.GtE: [ast.Lt, ast.LtE, ast.Gt, ast.Eq, ast.NotEq],
        ast.Eq: [ast.Lt, ast.LtE, ast.Gt, ast.GtE, ast.NotEq],
        ast.NotEq: [ast.Lt, ast.LtE, ast.Gt, ast.GtE, ast.Eq],
    }
    
    def mutate(self, node: ast.Compare) -> List[ast.Compare]:
        mutants = []
        
        for i, op in enumerate(node.ops):
            if type(op) not in self.OPERATORS:
                continue
            
            for replacement_op in self.OPERATORS[type(op)]:
                mutant = copy.deepcopy(node)
                mutant.ops[i] = replacement_op()
                mutants.append(mutant)
        
        return mutants


class ConditionalOperatorReplacement(MutationOperator):
    """Replace conditional operators: and or"""
    
    name = "COR"
    
    def mutate(self, node: ast.BoolOp) -> List[ast.AST]:
        mutants = []
        
        # and → or, or → and
        mutant = copy.deepcopy(node)
        if isinstance(node.op, ast.And):
            mutant.op = ast.Or()
        else:
            mutant.op = ast.And()
        mutants.append(mutant)
        
        # Remove each operand
        for i in range(len(node.values)):
            if len(node.values) > 1:
                mutant = copy.deepcopy(node)
                mutant.values = [v for j, v in enumerate(node.values) if j != i]
                if len(mutant.values) == 1:
                    mutants.append(mutant.values[0])
                else:
                    mutants.append(mutant)
        
        return mutants


class StatementDeletion(MutationOperator):
    """Delete statements."""
    
    name = "SDL"
    
    def mutate(self, node: ast.stmt) -> List[ast.Pass]:
        # Replace statement with pass
        return [ast.Pass()]


class ConstantReplacement(MutationOperator):
    """Replace constants with boundary values."""
    
    name = "CR"
    
    def mutate(self, node: ast.Constant) -> List[ast.Constant]:
        mutants = []
        
        if isinstance(node.value, int):
            # Boundary mutations
            mutants.extend([
                ast.Constant(value=0),
                ast.Constant(value=1),
                ast.Constant(value=-1),
                ast.Constant(value=node.value + 1),
                ast.Constant(value=node.value - 1),
                ast.Constant(value=-node.value),
            ])
        elif isinstance(node.value, bool):
            mutants.append(ast.Constant(value=not node.value))
        elif isinstance(node.value, str):
            mutants.extend([
                ast.Constant(value=""),
                ast.Constant(value=node.value + "mutated"),
            ])
        
        # Remove duplicates of original
        return [m for m in mutants if m.value != node.value]


class MutationEngine:
    """Generate and test mutants."""
    
    def __init__(self, 
                 operators: List[MutationOperator] = None,
                 timeout_seconds: float = 5.0):
        self.operators = operators or [
            ArithmeticOperatorReplacement(),
            RelationalOperatorReplacement(),
            ConditionalOperatorReplacement(),
            ConstantReplacement(),
        ]
        self.timeout = timeout_seconds
    
    def generate_mutants(self, source_code: str) -> List[Mutant]:
        """Generate all mutants for the given source code."""
        tree = ast.parse(source_code)
        mutants = []
        mutant_id = 0
        
        for node in ast.walk(tree):
            for operator in self.operators:
                node_mutants = self._try_mutate(node, operator)
                
                for mutated_node in node_mutants:
                    mutant_id += 1
                    mutants.append(Mutant(
                        id=mutant_id,
                        operator=operator.name,
                        original=ast.unparse(node),
                        mutated=ast.unparse(mutated_node),
                        location=(getattr(node, 'lineno', 0), 
                                 getattr(node, 'col_offset', 0)),
                    ))
        
        return mutants
    
    def _try_mutate(self, node: ast.AST, operator: MutationOperator) -> List[ast.AST]:
        """Try to apply operator to node."""
        try:
            return operator.mutate(node)
        except (TypeError, AttributeError):
            return []
    
    def run_mutation_testing(self,
                              source_code: str,
                              test_function: Callable[[], bool]) -> MutationResult:
        """
        Run mutation testing.
        
        Args:
            source_code: The code to mutate
            test_function: A function that runs tests, returns True if all pass
        
        Returns:
            MutationResult with statistics and survivors
        """
        mutants = self.generate_mutants(source_code)
        
        killed = 0
        survived = 0
        timeout = 0
        survivors = []
        
        for mutant in mutants:
            status = self._test_mutant(mutant, source_code, test_function)
            mutant.status = status
            
            if status == MutantStatus.KILLED:
                killed += 1
            elif status == MutantStatus.SURVIVED:
                survived += 1
                survivors.append(mutant)
            elif status == MutantStatus.TIMEOUT:
                timeout += 1
        
        total = killed + survived
        score = (killed / total * 100) if total > 0 else 0
        
        return MutationResult(
            total_mutants=len(mutants),
            killed=killed,
            survived=survived,
            timeout=timeout,
            equivalent=0,  # Requires human analysis
            mutation_score=score,
            survivors=survivors,
        )
    
    def _test_mutant(self,
                      mutant: Mutant,
                      original_source: str,
                      test_function: Callable) -> MutantStatus:
        """Test a single mutant."""
        # Create mutated source
        mutated_source = original_source.replace(
            mutant.original, 
            mutant.mutated,
            1  # Replace first occurrence only
        )
        
        try:
            # Execute mutated code
            exec_globals = {}
            exec(mutated_source, exec_globals)
            
            # Run tests with timeout
            import signal
            
            def timeout_handler(signum, frame):
                raise TimeoutError()
            
            signal.signal(signal.SIGALRM, timeout_handler)
            signal.alarm(int(self.timeout))
            
            try:
                tests_pass = test_function()
                signal.alarm(0)
                
                if tests_pass:
                    return MutantStatus.SURVIVED
                else:
                    return MutantStatus.KILLED
            except TimeoutError:
                return MutantStatus.TIMEOUT
            
        except Exception:
            # Mutant caused error - counts as killed
            return MutantStatus.KILLED
```

### Analyzing Survivors

```python
class SurvivorAnalyzer:
    """Analyze why mutants survived to improve tests."""
    
    def analyze_survivors(self, 
                          result: MutationResult,
                          source_code: str) -> List[dict]:
        """
        Analyze each surviving mutant and suggest test improvements.
        """
        analyses = []
        
        for mutant in result.survivors:
            analysis = {
                'mutant': mutant,
                'diagnosis': self._diagnose(mutant),
                'suggested_test': self._suggest_test(mutant),
                'is_equivalent': self._check_equivalent(mutant, source_code),
            }
            analyses.append(analysis)
        
        return analyses
    
    def _diagnose(self, mutant: Mutant) -> str:
        """Diagnose why this mutant might have survived."""
        
        if mutant.operator == 'ROR':
            return (f"Boundary condition not tested. "
                   f"Original: {mutant.original}, Mutant: {mutant.mutated}. "
                   f"Add test at exact boundary value.")
        
        elif mutant.operator == 'AOR':
            return (f"Arithmetic operation not fully tested. "
                   f"Test with values that distinguish {mutant.original} from {mutant.mutated}.")
        
        elif mutant.operator == 'CR':
            return (f"Constant value not significant to tests. "
                   f"Add test that specifically depends on value being {mutant.original}.")
        
        elif mutant.operator == 'COR':
            return (f"Logical condition not fully exercised. "
                   f"Test with combinations that distinguish {mutant.original} from {mutant.mutated}.")
        
        return "Unknown - manual analysis required."
    
    def _suggest_test(self, mutant: Mutant) -> str:
        """Suggest a test to kill this mutant."""
        
        if '>=' in mutant.original and '>' in mutant.mutated:
            # >= mutated to >, need test at exact boundary
            return "Add test with value at exact boundary (the equality case)."
        
        if '<=' in mutant.original and '<' in mutant.mutated:
            return "Add test with value at exact boundary (the equality case)."
        
        if 'and' in mutant.original.lower() and 'or' in mutant.mutated.lower():
            return "Add test where first condition is True, second is False."
        
        return f"Add test that produces different result for {mutant.original} vs {mutant.mutated}."
    
    def _check_equivalent(self, mutant: Mutant, source_code: str) -> bool:
        """
        Check if mutant is equivalent (produces same behavior).
        This is undecidable in general - heuristics only.
        """
        # Common equivalent mutant patterns
        equivalent_patterns = [
            # x * 1 → x * -1 when x is always 0
            # return x → return +x
            # etc.
        ]
        
        # This requires human judgment ultimately
        return False


def generate_test_for_survivor(mutant: Mutant) -> str:
    """
    Generate a test skeleton to kill a surviving mutant.
    """
    return f'''
def test_kill_mutant_{mutant.id}():
    """
    Kill mutant: {mutant.operator}
    Original: {mutant.original}
    Mutated:  {mutant.mutated}
    
    This test should pass with original code
    but fail with mutated code.
    """
    # TODO: Add test that distinguishes original from mutant
    # The key is finding an input where:
    #   original({mutant.original}) != mutant({mutant.mutated})
    
    result = function_under_test(input_that_distinguishes)
    assert result == expected_from_original
'''
```

### Mutation Score Tracking

```python
class MutationScoreTracker:
    """Track mutation score over time for quality metrics."""
    
    def __init__(self, project_name: str):
        self.project = project_name
        self.history = []
    
    def record(self, 
               module: str,
               result: MutationResult,
               commit_hash: str = None):
        """Record mutation testing result."""
        self.history.append({
            'timestamp': datetime.now(),
            'commit': commit_hash,
            'module': module,
            'mutation_score': result.mutation_score,
            'total_mutants': result.total_mutants,
            'killed': result.killed,
            'survived': result.survived,
            'survivors': [
                {'operator': m.operator, 'location': m.location}
                for m in result.survivors
            ]
        })
    
    def trend_report(self) -> dict:
        """Generate trend report."""
        if len(self.history) < 2:
            return {'trend': 'insufficient data'}
        
        scores = [h['mutation_score'] for h in self.history]
        
        return {
            'current_score': scores[-1],
            'previous_score': scores[-2],
            'change': scores[-1] - scores[-2],
            'trend': 'improving' if scores[-1] > scores[-2] else 'degrading',
            'all_time_high': max(scores),
            'all_time_low': min(scores),
            'average': sum(scores) / len(scores),
        }
    
    def quality_gate(self, 
                     minimum_score: float = 80.0,
                     max_regression: float = 5.0) -> Tuple[bool, str]:
        """
        CI quality gate based on mutation score.
        """
        if not self.history:
            return False, "No mutation testing results"
        
        current = self.history[-1]['mutation_score']
        
        if current < minimum_score:
            return False, f"Mutation score {current}% below minimum {minimum_score}%"
        
        if len(self.history) >= 2:
            previous = self.history[-2]['mutation_score']
            regression = previous - current
            
            if regression > max_regression:
                return False, f"Mutation score regressed by {regression}% (max allowed: {max_regression}%)"
        
        return True, f"Mutation score {current}% meets quality standards"
```

## Mental Model

Lipton approaches test quality by asking:

1. **Can tests detect simple faults?** If not, they won't detect complex ones
2. **What's the mutation score?** The honest metric of test effectiveness
3. **Why did mutants survive?** Each survivor reveals a test weakness
4. **Is it equivalent?** Some mutants can't be killed (same behavior)
5. **Which operators matter?** Focus on the mutations that model real bugs

## The Mutation Testing Checklist

```
□ Select mutation operators appropriate to language
□ Generate mutants for critical code paths
□ Run test suite against each mutant
□ Calculate mutation score (killed / total)
□ Analyze each survivor
□ Identify equivalent mutants (cannot be killed)
□ Write tests to kill non-equivalent survivors
□ Track mutation score over time
□ Set quality gates in CI
```

## Signature Lipton Moves

- Competent Programmer Hypothesis
- Coupling Effect
- Mutation operators (AOR, ROR, COR, SDL, CR)
- Mutation score as quality metric
- Equivalent mutant identification
- Survivor analysis
- Boundary-focused mutations
- Test gap detection through surviving mutants
