---
name: maciver-hypothesis-testing
description: Test software in the style of David MacIver, creator of Hypothesis. Emphasizes sophisticated shrinking, example databases, stateful testing, and practical property-based testing in Python. Use when testing Python code with complex data structures, APIs, or stateful systems.
---

# David MacIver Hypothesis Style Guide

## Overview

David MacIver created Hypothesis, widely considered the most sophisticated property-based testing library available. Hypothesis improves on QuickCheck with better shrinking (using internal reduction rather than type-based shrinking), an example database for regression testing, and deep integration with Python's ecosystem. MacIver's philosophy emphasizes that property-based testing should be practical, integrated into normal development workflows, and produce genuinely useful minimal examples.

## Core Philosophy

> "The purpose of Hypothesis is to make it easier to write better tests."

> "Shrinking should produce the simplest example, not just a smaller one."

> "Every failing example should be saved and replayed forever."

MacIver believes that property-based testing fails when it's treated as exotic. Hypothesis is designed to integrate seamlessly with pytest, produce human-readable minimal examples, and remember every failure to prevent regressions.

## Design Principles

1. **Integrated Shrinking**: Shrinking happens during generation, not after—producing simpler examples.

2. **Example Database**: Every failure is saved and replayed on subsequent runs.

3. **Compositional Strategies**: Build complex generators from simple ones.

4. **Practical by Default**: Sensible defaults that work for real projects.

5. **Stateful Testing**: First-class support for testing state machines.

## Hypothesis Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    HYPOTHESIS INTERNALS                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  STRATEGY (describes how to generate data)                  │
│      │                                                       │
│      ▼                                                       │
│  CONJECTURE ENGINE                                           │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Generates a stream of bytes (the "choice sequence")    │ │
│  │                                                         │ │
│  │ Strategy interprets bytes → structured data            │ │
│  │                                                         │ │
│  │ Shrinking = finding smaller choice sequences           │ │
│  │ that still fail (not type-aware, universal)            │ │
│  └────────────────────────────────────────────────────────┘ │
│      │                                                       │
│      ▼                                                       │
│  EXAMPLE DATABASE                                            │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ .hypothesis/examples/                                   │ │
│  │ Stores choice sequences for all failing examples       │ │
│  │ Replays them first on every run                        │ │
│  │ Never forgets a failure                                │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## When Using Hypothesis

### Always

- Use `@given` decorator for property-based tests
- Let Hypothesis shrink—don't manually minimize
- Commit the `.hypothesis` directory for CI
- Use `@example` for important edge cases
- Combine with pytest fixtures naturally
- Use `assume()` to filter invalid inputs

### Never

- Ignore the example database (commit it!)
- Use `random` directly—use strategies
- Catch exceptions to hide failures
- Set `max_examples` too low (<100)
- Skip `@example` for known edge cases
- Use `filter()` when `assume()` works

### Prefer

- Composite strategies over complex custom ones
- `@example` for regression tests
- `assume()` over `filter()` for preconditions
- Stateful testing for APIs
- `data()` strategy for dynamic generation
- Settings profiles for CI vs local

## Code Patterns

### Basic Property Tests

```python
from hypothesis import given, example, assume, settings
from hypothesis import strategies as st


# Basic property test
@given(st.lists(st.integers()))
def test_sort_preserves_length(xs):
    assert len(sorted(xs)) == len(xs)


# With explicit example for regression
@given(st.lists(st.integers()))
@example([])  # Empty list edge case
@example([1])  # Single element
@example([2, 1])  # Minimal unsorted
def test_sort_is_sorted(xs):
    result = sorted(xs)
    assert all(result[i] <= result[i+1] for i in range(len(result)-1))


# Roundtrip property
@given(st.binary())
def test_compress_decompress_roundtrip(data):
    assert decompress(compress(data)) == data


# With precondition
@given(st.integers(), st.integers())
def test_division(a, b):
    assume(b != 0)  # Skip when b is 0
    assert (a // b) * b + (a % b) == a
```

### Strategy Composition

```python
from hypothesis import strategies as st
from hypothesis import given

# Composite strategy for custom types
@st.composite
def user_strategy(draw):
    """Generate valid User objects."""
    name = draw(st.text(min_size=1, max_size=50))
    age = draw(st.integers(min_value=0, max_value=150))
    email = draw(st.emails())
    
    return User(name=name, age=age, email=email)


@given(user_strategy())
def test_user_serialization(user):
    assert User.from_json(user.to_json()) == user


# Recursive strategy for tree structures
def json_strategy():
    """Generate arbitrary JSON-compatible data."""
    return st.recursive(
        # Base case: simple values
        st.none() | st.booleans() | st.integers() | st.floats(allow_nan=False) | st.text(),
        # Recursive case: lists and dicts
        lambda children: st.lists(children) | st.dictionaries(st.text(), children),
        max_leaves=50
    )


@given(json_strategy())
def test_json_roundtrip(data):
    import json
    assert json.loads(json.dumps(data)) == data


# Dependent generation
@st.composite
def sorted_pair(draw):
    """Generate two integers where a <= b."""
    a = draw(st.integers())
    b = draw(st.integers(min_value=a))
    return (a, b)


@given(sorted_pair())
def test_range(pair):
    a, b = pair
    assert a <= b
```

### Stateful Testing

```python
from hypothesis.stateful import RuleBasedStateMachine, rule, invariant
from hypothesis import strategies as st


class DatabaseStateMachine(RuleBasedStateMachine):
    """
    Test a key-value store against a reference model.
    Hypothesis generates sequences of operations.
    """
    
    def __init__(self):
        super().__init__()
        self.model = {}  # Reference model
        self.db = Database()  # System under test
    
    @rule(key=st.text(), value=st.binary())
    def put(self, key, value):
        """Put a key-value pair."""
        self.model[key] = value
        self.db.put(key, value)
    
    @rule(key=st.text())
    def get(self, key):
        """Get a value by key."""
        model_result = self.model.get(key)
        db_result = self.db.get(key)
        assert model_result == db_result
    
    @rule(key=st.text())
    def delete(self, key):
        """Delete a key."""
        self.model.pop(key, None)
        self.db.delete(key)
    
    @invariant()
    def size_matches(self):
        """Invariant: size should always match model."""
        assert len(self.model) == self.db.size()


# Run the state machine test
TestDatabase = DatabaseStateMachine.TestCase


# More complex: with bundles for generated values
class QueueStateMachine(RuleBasedStateMachine):
    """Test a queue with references to pushed items."""
    
    def __init__(self):
        super().__init__()
        self.model = []
        self.queue = Queue()
    
    items = Bundle('items')
    
    @rule(target=items, item=st.integers())
    def push(self, item):
        """Push an item and track it."""
        self.model.append(item)
        self.queue.push(item)
        return item
    
    @rule()
    def pop(self):
        """Pop an item."""
        if self.model:
            expected = self.model.pop(0)
            actual = self.queue.pop()
            assert expected == actual
    
    @rule(item=items)
    def contains(self, item):
        """Check if a previously-pushed item is present."""
        expected = item in self.model
        actual = self.queue.contains(item)
        assert expected == actual
```

### Settings and Profiles

```python
from hypothesis import settings, Verbosity, Phase, HealthCheck
from hypothesis import given
from hypothesis import strategies as st


# Custom settings for a single test
@settings(
    max_examples=500,
    deadline=None,  # No time limit
    suppress_health_check=[HealthCheck.too_slow],
)
@given(st.lists(st.integers(), min_size=1000))
def test_large_lists(xs):
    assert sorted(xs) == sorted(xs)


# Register profiles for different environments
settings.register_profile("ci", max_examples=1000)
settings.register_profile("dev", max_examples=100)
settings.register_profile("debug", max_examples=10, verbosity=Verbosity.verbose)

# Load profile from environment
# HYPOTHESIS_PROFILE=ci pytest


# Deadline handling
@settings(deadline=200)  # 200ms max per example
@given(st.binary(min_size=1000))
def test_with_deadline(data):
    process(data)


# Control phases
@settings(
    phases=[
        Phase.explicit,  # Run @example first
        Phase.reuse,     # Run from database
        Phase.generate,  # Generate new examples
        Phase.shrink,    # Shrink failures
    ]
)
@given(st.integers())
def test_all_phases(n):
    assert n < 1000000
```

### Advanced Strategies

```python
from hypothesis import strategies as st
from hypothesis import given


# Fixed dictionaries
@given(st.fixed_dictionaries({
    'name': st.text(min_size=1),
    'age': st.integers(0, 150),
    'active': st.booleans(),
}))
def test_user_dict(user):
    assert 'name' in user
    assert 0 <= user['age'] <= 150


# One of several strategies
@given(st.one_of(
    st.none(),
    st.integers(),
    st.text(),
))
def test_nullable_values(value):
    # Value is None, int, or str
    assert value is None or isinstance(value, (int, str))


# Data strategy for dynamic generation
from hypothesis import given
from hypothesis.strategies import data

@given(data())
def test_dynamic_generation(data):
    # Generate based on runtime conditions
    n = data.draw(st.integers(min_value=1, max_value=10))
    xs = data.draw(st.lists(st.integers(), min_size=n, max_size=n))
    
    assert len(xs) == n


# Sampled from
@given(st.sampled_from(['red', 'green', 'blue']))
def test_colors(color):
    assert color in ['red', 'green', 'blue']


# Builds for dataclasses/namedtuples
from dataclasses import dataclass

@dataclass
class Point:
    x: float
    y: float

@given(st.builds(Point, x=st.floats(), y=st.floats()))
def test_point(point):
    assert hasattr(point, 'x')
    assert hasattr(point, 'y')


# From type annotations (inferred)
@given(st.from_type(Point))
def test_point_from_type(point):
    assert isinstance(point, Point)
```

### Shrinking Examples

```python
# Hypothesis shrinking in action

# BAD: This will shrink to something like [0, 0, 1]
@given(st.lists(st.integers()))
def test_no_duplicates_bad(xs):
    assert len(xs) == len(set(xs))  # Fails for [0, 0]

# Shrunk output: Falsifying example: test_no_duplicates_bad(xs=[0, 0])
# Note: Hypothesis found the MINIMAL failing case automatically


# The shrinking process:
# 1. Initial failure: [42, -17, 0, 999, 42, 3]
# 2. Try smaller list: [42, -17, 0, 999, 42] - still fails? no (no dup)
# 3. Try: [42, -17, 42] - fails? yes (has dup)
# 4. Try: [42, 42] - fails? yes
# 5. Try: [0, 0] - fails? yes (simpler values)
# 6. Try: [0] - fails? no
# 7. Minimal: [0, 0]


# Custom shrinking with map
@given(st.integers().map(lambda x: x * 2))
def test_even_numbers(n):
    assert n % 2 == 0  # Always passes, but shrinks to 0


# Filter affects shrinking
@given(st.integers().filter(lambda x: x > 10))
def test_large_numbers(n):
    assert n < 100  # Shrinks to 11 (smallest that passes filter)
```

### Integration with Pytest

```python
import pytest
from hypothesis import given, settings
from hypothesis import strategies as st


# Combine with fixtures
@pytest.fixture
def database():
    db = Database()
    yield db
    db.close()


@given(key=st.text(), value=st.binary())
def test_database_roundtrip(database, key, value):
    database.put(key, value)
    assert database.get(key) == value


# Parametrize + hypothesis
@pytest.mark.parametrize("operation", ["add", "subtract", "multiply"])
@given(a=st.integers(), b=st.integers())
def test_operations(operation, a, b):
    result = calculate(operation, a, b)
    # Verify properties based on operation
    if operation == "add":
        assert result == a + b


# Mark slow tests
@pytest.mark.slow
@settings(max_examples=10000)
@given(st.binary(min_size=10000))
def test_large_data(data):
    process(data)


# Skip in certain conditions
@pytest.mark.skipif(condition, reason="...")
@given(st.integers())
def test_conditional(n):
    pass
```

### Reproducing Failures

```python
from hypothesis import given, reproduce_failure, settings
from hypothesis import strategies as st


# Hypothesis prints reproduction code on failure:
# 
# Falsifying example: test_example(x=[0, 0])
# 
# You can add @reproduce_failure(...) to force this example:
# @reproduce_failure('6.100.0', b'AAAB')


# Force a specific failing example for debugging
@reproduce_failure('6.100.0', b'AAAB')  # Version and blob
@given(st.lists(st.integers()))
def test_reproduction(xs):
    assert len(xs) == len(set(xs))


# Example database location
# .hypothesis/examples/<test_function_name>
#
# Commit this to version control!
# Every CI run will replay all known failures first


# Explicit seed for determinism
@settings(database=None)  # Disable database
@given(st.integers())
def test_with_seed(n):
    assert n != 42


# Run with: pytest --hypothesis-seed=12345
```

### Testing APIs

```python
from hypothesis import given, assume
from hypothesis import strategies as st
import requests


# Generate valid HTTP requests
@st.composite
def http_request(draw):
    method = draw(st.sampled_from(['GET', 'POST', 'PUT', 'DELETE']))
    path = '/' + draw(st.text(
        alphabet='abcdefghijklmnopqrstuvwxyz/',
        min_size=1,
        max_size=50
    ))
    
    if method in ('POST', 'PUT'):
        body = draw(st.dictionaries(
            st.text(min_size=1),
            st.text() | st.integers() | st.booleans()
        ))
    else:
        body = None
    
    return {'method': method, 'path': path, 'body': body}


@given(http_request())
def test_api_doesnt_crash(request):
    """Property: API should never crash (return 5xx)."""
    response = make_request(
        request['method'],
        request['path'],
        json=request['body']
    )
    
    # 4xx is fine (client error), 5xx is not
    assert response.status_code < 500


# Schemathesis-style: generate from OpenAPI spec
@given(st.from_schema(openapi_schema))
def test_schema_conformance(request):
    response = client.request(**request)
    validate_response(response, openapi_schema)
```

## Mental Model

MacIver approaches testing by asking:

1. **What properties should hold?** Think invariants and roundtrips
2. **What's the simplest failure?** Trust Hypothesis shrinking
3. **Am I saving failures?** Commit the example database
4. **Can I compose strategies?** Build complex from simple
5. **Is it stateful?** Use RuleBasedStateMachine

## The Hypothesis Checklist

```
□ Use @given with appropriate strategies
□ Add @example for known edge cases
□ Use assume() for preconditions
□ Commit .hypothesis/ directory
□ Set appropriate max_examples (100+)
□ Configure CI profile with more examples
□ Use composite strategies for complex types
□ Use stateful testing for APIs
□ Trust the shrinking—don't manually minimize
□ Check deadline settings for slow tests
```

## Signature MacIver Moves

- Integrated shrinking (choice sequence reduction)
- Example database (never forget a failure)
- Composite strategies (@st.composite)
- Stateful testing (RuleBasedStateMachine)
- Settings profiles (CI vs dev)
- Type-based inference (st.from_type)
- Recursive strategies (st.recursive)
- Health checks for performance issues
