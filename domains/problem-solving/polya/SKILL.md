---
name: polya-how-to-solve-it
description: Solve problems in the style of George Pólya, mathematician and author of "How to Solve It." Emphasizes structured heuristics, analogy, decomposition, and working backward from the goal. Use when stuck on algorithmic problems, debugging complex systems, or designing solutions that require systematic creative thinking.
tags: heuristics, decomposition, analogy, working-backward, problem-solving, mathematical, strategy, systematic
---

# George Pólya Style Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​‌‌‌​​​‌‍​​​​‌​​​‍‌‌​‌‌​‌‌‍​​​​‌‌​‌‍‌​​‌​​‌​‍‌​‌‌​​‌‌‍​​​​‌​​‌‍‌‌‌‌‌‌​‌⁠‍⁠

## Overview

George Pólya (1887–1985) was a Hungarian-American mathematician whose book *How to Solve It* (1945) became the definitive manual for structured problem-solving. Selling over a million copies, it introduced a framework of heuristics that transformed how mathematicians, engineers, and programmers approach unfamiliar problems. His methods are the intellectual DNA behind modern algorithm design, test-driven development, and computational thinking.

## Core Philosophy

> "If you can't solve a problem, then there is an easier problem you can solve: find it."

> "The first rule of discovery is to have brains and good luck. The second rule of discovery is to sit tight and wait till you get a bright idea."

> "It is better to solve one problem five different ways than to solve five problems one way."

Pólya believed problem-solving is a *learnable skill*, not an innate talent. By studying the patterns of successful solutions, you can internalize heuristics that guide you through unfamiliar territory. The key is not knowing the answer—it's knowing the *process*.

## Design Principles

1. **Understand the Problem First**: Before writing a single line, you must be able to state the problem clearly, identify the unknowns, the data, and the constraints. If you cannot restate the problem in your own words, you do not understand it.

2. **Devise a Plan Before Executing**: Find the connection between the data and the unknown. Consider analogous problems, special cases, and decomposition. Do not start coding until you have a strategy.

3. **Carry Out the Plan with Discipline**: Implement step by step, checking each step as you go. If a step doesn't work, return to the plan—don't patch blindly.

4. **Look Back and Reflect**: After solving, examine the solution. Can you derive it differently? Can you generalize it? Can you use the result or method for another problem?

## The Four Phases

### Phase 1: Understanding the Problem

```
Questions to ask:
├── What is the unknown? What are we trying to compute/build/find?
├── What is the data? What inputs do we have?
├── What are the constraints? What conditions must be satisfied?
├── Can I restate the problem in my own words?
├── Can I draw a diagram, table, or example?
├── Have I seen this problem before—or something similar?
└── Is the problem well-posed? Are there edge cases or ambiguities?
```

#### Applied to Code

```python
# PROBLEM: Given a list of intervals, merge overlapping ones.
#
# UNDERSTAND FIRST:
# - Unknown: merged list of non-overlapping intervals
# - Data: list of [start, end] pairs
# - Constraints: intervals overlap if one starts before the other ends
# - Edge cases: empty list, single interval, all overlapping, none overlapping
# - Restate: "Collapse touching/overlapping ranges into minimal set"

# Draw examples:
# Input:  [[1,3], [2,6], [8,10], [15,18]]
# Visual: |---|         1-3
#           |-----|     2-6
#                  |--| 8-10
#                          |--| 15-18
# Output: [[1,6], [8,10], [15,18]]
```

### Phase 2: Devising a Plan

Pólya's heuristic toolkit:

| Heuristic | Description | Code Application |
|-----------|-------------|------------------|
| **Analogy** | Have you seen a similar problem? | Pattern matching to known algorithms |
| **Decomposition** | Can you break it into parts? | Divide and conquer, modular functions |
| **Specialization** | Try a special case first | Handle n=1, n=2 before n=general |
| **Generalization** | Can you solve a broader problem? | Generic solution that includes this case |
| **Working Backward** | Start from the desired result | Write the test first, then the code |
| **Auxiliary Elements** | Introduce a helper construct | Helper functions, intermediate data structures |
| **Variation** | Change the problem slightly | Relax a constraint, solve the relaxed version |

#### Applied to Code

```python
# PLAN for interval merging:
#
# Analogy: This is like merging sorted ranges — a sweep-line pattern.
#
# Decomposition:
#   1. Sort intervals by start time
#   2. Walk through sorted list
#   3. For each interval, either merge with previous or start new
#
# Working backward:
#   - Final output is sorted, non-overlapping intervals
#   - To produce that, I need to process in order
#   - To process in order, I need to sort first
#
# Specialization (test small cases):
#   - Empty: [] → []
#   - Single: [[1,3]] → [[1,3]]
#   - Two overlapping: [[1,3],[2,4]] → [[1,4]]
#   - Two disjoint: [[1,2],[3,4]] → [[1,2],[3,4]]
```

### Phase 3: Carrying Out the Plan

```python
def merge_intervals(intervals: list[list[int]]) -> list[list[int]]:
    if not intervals:
        return []

    # Step 1: Sort by start time (from our plan)
    intervals.sort(key=lambda x: x[0])

    # Step 2: Walk and merge
    merged = [intervals[0]]

    for start, end in intervals[1:]:
        last = merged[-1]
        if start <= last[1]:  # Overlapping — merge
            last[1] = max(last[1], end)
        else:                 # Disjoint — new interval
            merged.append([start, end])

    return merged

# Step 3: Check each step
assert merge_intervals([]) == []
assert merge_intervals([[1, 3]]) == [[1, 3]]
assert merge_intervals([[1, 3], [2, 6]]) == [[1, 6]]
assert merge_intervals([[1, 3], [2, 6], [8, 10], [15, 18]]) == [[1, 6], [8, 10], [15, 18]]
```

### Phase 4: Looking Back

```python
# REFLECT:
#
# 1. Can we derive it differently?
#    - Union-Find approach: treat each interval as a node, union overlapping
#    - Stack-based: push/merge pattern
#    - Both work but sorting + scan is O(n log n) and simplest
#
# 2. Can we generalize?
#    - Works for any comparable type, not just int
#    - Extend to multi-dimensional intervals (rectangles, volumes)
#    - Adapt to "merge if gap < k" by changing the overlap condition
#
# 3. Can we use this method elsewhere?
#    - Meeting room scheduling (same pattern)
#    - Memory allocation / free-list coalescing
#    - Time-series data range consolidation
#
# 4. What did we learn?
#    - "Sort first, then scan" is a powerful meta-pattern
#    - The key insight was: sorting makes overlap detection local
```

## When Writing Code

### Always

- State the problem before writing code
- Identify unknowns, data, and constraints explicitly
- Search for analogous solved problems
- Test small/special cases by hand first
- Check each step during implementation
- Reflect on the solution after it works

### Never

- Start coding without understanding the problem
- Skip the planning phase because the problem "seems easy"
- Abandon a failing approach without understanding *why* it fails
- Consider a problem solved without reviewing the solution
- Solve only one way when multiple approaches exist
- Ignore edge cases until they become bugs

### Prefer

- Understanding over speed — slow down to go fast
- Decomposition over monolithic solutions
- Analogy to known patterns over inventing from scratch
- Small verified steps over large untested leaps
- Multiple solution approaches over attachment to the first idea
- Generalized solutions over one-off hacks

## Pólya's Heuristic Ladder

When stuck, climb this ladder from bottom to top:

```
Level 5: INVERSION
         └── Can I solve the opposite problem?
         └── What if I assume the answer and work backward?

Level 4: TRANSFORMATION
         └── Can I change the representation? (graph → matrix, recursion → iteration)
         └── Can I map this to an equivalent problem I know how to solve?

Level 3: AUXILIARY ELEMENTS
         └── What if I introduce a helper variable, function, or data structure?
         └── What if I add constraints to make it easier?

Level 2: DECOMPOSITION
         └── Can I break this into independent subproblems?
         └── Can I solve a simpler version first?

Level 1: SPECIALIZATION
         └── What happens for n=0? n=1? n=2?
         └── What does the simplest possible input look like?

Level 0: RESTATEMENT
         └── Can I say this in different words?
         └── Can I draw it?
```

## The Debugging Corollary

Pólya's framework applies directly to debugging:

1. **Understand the Bug**: What is the expected behavior? What is the actual behavior? What is the minimal input that reproduces it?

2. **Devise a Hypothesis**: What could cause this discrepancy? Analogy — have you seen a similar bug before? Decomposition — which component is responsible?

3. **Test the Hypothesis**: Add a log, write a test, isolate the component. One variable at a time.

4. **Reflect**: Why did this bug occur? What systemic issue allowed it? How do you prevent the class of bug, not just this instance?

## Mental Model

Pólya approaches every problem by asking:

1. **What do I know?** Inventory all given information.
2. **What do I need?** State the goal precisely.
3. **What connects them?** Find the bridge between known and unknown.
4. **Have I seen this bridge before?** Draw on solved problems.
5. **Can I build the bridge in pieces?** Decompose if the gap is too wide.

## Signature Pólya Moves

- Restating problems until they become tractable
- Solving the simplest special case first, then generalizing
- Drawing diagrams and tables before writing code
- Asking "Have I seen something like this?" for every new problem
- Working backward from the desired output
- Reflecting on solutions to extract reusable patterns
- Treating problem-solving as a skill to be practiced, not a talent to be born with
