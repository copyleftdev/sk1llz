# Raymond Hettinger Philosophy

## Core Beliefs

### "There Should Be One Obvious Way"

> "Python is a language optimized for code reading, not code writing."

Hettinger champions Python's readability over cleverness. Code is read far more often than written.

### The Transformation Pipeline

Hettinger's signature teaching approach: show the progression from naive to Pythonic code.

```python
# Level 0: C-style
i = 0
while i < len(items):
    print(items[i])
    i += 1

# Level 1: Range
for i in range(len(items)):
    print(items[i])

# Level 2: Direct iteration
for item in items:
    print(item)

# Level 3: With index if needed
for i, item in enumerate(items):
    print(i, item)
```

## Key Principles

### 1. Use the Right Tool

```python
# Bad: Manual dictionary building
d = {}
for item in items:
    key = get_key(item)
    if key not in d:
        d[key] = []
    d[key].append(item)

# Good: defaultdict
from collections import defaultdict
d = defaultdict(list)
for item in items:
    d[get_key(item)].append(item)

# Best: groupby (if sorted)
from itertools import groupby
d = {k: list(g) for k, g in groupby(sorted(items, key=get_key), key=get_key)}
```

### 2. Named Tuples Over Tuples

```python
# Bad: Mysterious indices
point = (10, 20)
x = point[0]

# Good: Self-documenting
from collections import namedtuple
Point = namedtuple('Point', ['x', 'y'])
point = Point(10, 20)
x = point.x
```

### 3. Context Managers for Resource Management

```python
# Bad: Manual cleanup
f = open('file.txt')
try:
    data = f.read()
finally:
    f.close()

# Good: Context manager
with open('file.txt') as f:
    data = f.read()
```

### 4. Generators for Memory Efficiency

```python
# Bad: Build entire list in memory
def get_squares(n):
    result = []
    for i in range(n):
        result.append(i ** 2)
    return result

# Good: Yield one at a time
def get_squares(n):
    for i in range(n):
        yield i ** 2
```

## Famous Talks

1. **"Transforming Code into Beautiful, Idiomatic Python"** (PyCon 2013)
   - The canonical "Pythonic code" talk
   - Shows before/after transformations

2. **"Beyond PEP 8"** (PyCon 2015)
   - Style isn't just formatting
   - Focus on readability, not rules

3. **"Super Considered Super"** (PyCon 2015)
   - Proper use of `super()` in Python 3
   - Cooperative multiple inheritance

4. **"Dataclasses: The Code Generator"** (PyCon 2018)
   - Modern Python data classes
   - Reducing boilerplate

## Hettinger's Standard Library Contributions

- `collections` module (Counter, defaultdict, deque, namedtuple, OrderedDict)
- `itertools` module (chain, groupby, islice, etc.)
- `functools` module (lru_cache, partial, reduce)
- Set operations and frozenset
- Decimal module improvements

## Quotes Collection

> "There's an important distinction between code that runs and code that is correct."

> "If you want to go fast, first focus on going correctly."

> "The art of programming is the art of organizing complexity."

> "Beautiful is better than ugly. Explicit is better than implicit."

> "Don't use classes when a simple function will do."
