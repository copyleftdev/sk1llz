---
name: click-jvm-optimization
description: Design JIT compilers and optimize managed runtimes in the style of Cliff Click, architect of the HotSpot JVM C2 compiler and creator of sea-of-nodes IR. Emphasizes advanced compiler optimizations, intermediate representation design, and making dynamic languages fast. Use when building JIT compilers, VMs, or working on compiler backends.
---

# Cliff Click Style Guide

## Overview

Cliff Click was the chief architect of the HotSpot Server Compiler (C2), which made Java competitive with C++ for server workloads. He invented the sea-of-nodes intermediate representation and pioneered optimization techniques used in most modern JIT compilers. His work proved that dynamic languages could be fast.

## Core Philosophy

> "Optimization is about proving things don't happen."

> "The best optimization is the one that removes code entirely."

> "A good IR makes optimizations fall out naturally."

Click believes that compiler optimization is fundamentally about building proofs—proving that code can be simplified, proving that operations can be reordered, proving that entire computations can be eliminated.

## Design Principles

1. **Sea of Nodes IR**: Data dependencies, not artificial instruction order.

2. **Speculative Optimization**: Optimize for the common case, deoptimize when wrong.

3. **Type Speculation**: Dynamic types can be optimized like static types.

4. **Escape Analysis**: Prove objects don't escape, eliminate allocations.

## When Building Compilers

### Always

- Design IR for the optimizations you want
- Preserve information needed for later passes
- Make deoptimization fast and correct
- Profile to guide optimization decisions
- Inline aggressively (with limits)
- Prove properties rather than assuming them

### Never

- Lose information during lowering too early
- Optimize without profiling data
- Make deoptimization expensive
- Assume optimization order doesn't matter
- Skip escape analysis for object languages
- Ignore memory aliasing

### Prefer

- Sea-of-nodes over linear IR for optimization
- Speculative optimization with guards
- Type feedback over static analysis alone
- Global value numbering over local CSE
- Loop transformations that enable vectorization
- Incremental compilation over batch

## Code Patterns

### Sea of Nodes IR

```java
// Traditional: Linear IR with explicit order
// t1 = load x
// t2 = load y  
// t3 = add t1, t2
// store z, t3

// Sea of Nodes: Graph with data dependencies only
//
//    Start
//      |
//   Memory
//    /   \
// Load x  Load y
//    \   /
//     Add
//      |
//    Store z
//      |
//     End

class Node {
    int opcode;
    Node[] inputs;   // Data dependencies
    Node[] outputs;  // Uses of this node
    
    // No explicit "next" instruction
    // Order determined by dependencies
}

// Benefits:
// - Reordering is free (just valid orderings)
// - Dead code elimination is trivial (no outputs)
// - Common subexpression elimination natural
// - Control flow is just another dependency
```

### Global Value Numbering

```java
// Find redundant computations across entire method

class ValueNumbering {
    Map<NodeKey, Node> valueNumbers = new HashMap<>();
    
    Node idealize(Node n) {
        // Compute canonical form
        NodeKey key = canonicalize(n);
        
        // Already computed this value?
        Node existing = valueNumbers.get(key);
        if (existing != null) {
            return existing;  // Reuse existing computation
        }
        
        valueNumbers.put(key, n);
        return n;
    }
    
    NodeKey canonicalize(Node n) {
        // Commutative ops: sort operands
        if (isCommutative(n.opcode)) {
            sortInputs(n);
        }
        
        // Algebraic identities
        // x + 0 → x
        // x * 1 → x  
        // x & x → x
        
        return new NodeKey(n.opcode, n.inputs);
    }
}

// In sea-of-nodes: value numbering IS the representation
// Each unique computation exists exactly once
```

### Speculative Optimization with Guards

```java
// Optimize for observed types, guard against others

void compileCallSite(CallSite site, ProfileData profile) {
    if (profile.isMonomorphic()) {
        // 95% of calls go to one method
        Class<?> observedType = profile.getObservedType();
        
        // Emit optimized code with guard
        emitTypeCheck(observedType);      // Guard
        emitDirectCall(observedType);     // Inline opportunity!
        emitDeoptimize();                 // Uncommon trap
        
    } else if (profile.isBimorphic()) {
        // Two types observed
        emitTypeSwitch(profile.getTypes());
        // Inline both paths
        
    } else {
        // Megamorphic: fall back to virtual dispatch
        emitVirtualCall();
    }
}

// Key insight: wrong guesses don't crash
// They just deoptimize and continue in interpreter
```

### Escape Analysis

```java
// Prove object doesn't escape → eliminate allocation

class EscapeAnalysis {
    boolean canEliminate(AllocationNode alloc) {
        // Track all uses of the allocation
        Set<Node> uses = alloc.getTransitiveUses();
        
        for (Node use : uses) {
            if (escapes(alloc, use)) {
                return false;
            }
        }
        
        return true;  // Safe to scalar replace
    }
    
    boolean escapes(AllocationNode alloc, Node use) {
        // Escapes if:
        // - Stored to heap (another object's field)
        // - Passed to unknown method
        // - Returned from method
        // - Stored to static field
        // - Used in synchronization
        
        if (use instanceof StoreField) {
            StoreField sf = (StoreField) use;
            return sf.getObject() != alloc;  // Storing TO another object
        }
        
        if (use instanceof Call) {
            Call call = (Call) use;
            return !isInlinedCall(call);
        }
        
        // ... other escape conditions
        return false;
    }
}

// Scalar replacement: object fields become local variables
// Point p = new Point(x, y);
// return p.x + p.y;
// 
// Becomes:
// int p_x = x;
// int p_y = y;
// return p_x + p_y;
// 
// No allocation!
```

### Loop Optimizations

```java
// Loop transformations for performance

class LoopOptimizations {
    
    void optimizeLoop(LoopNode loop) {
        // 1. Loop invariant code motion
        // Move computations out of loop if they don't change
        for (Node n : loop.getBody()) {
            if (isLoopInvariant(n, loop)) {
                moveBeforeLoop(n, loop);
            }
        }
        
        // 2. Induction variable analysis
        // Recognize i++, i += stride patterns
        InductionVar iv = findInductionVariable(loop);
        
        // 3. Range check elimination
        // array[i] where 0 <= i < array.length
        // Hoist bounds check outside loop
        if (canEliminateRangeCheck(loop, iv)) {
            hoistRangeCheck(loop, iv);
        }
        
        // 4. Loop unrolling
        // Reduce loop overhead, enable more optimization
        if (shouldUnroll(loop)) {
            unroll(loop, UNROLL_FACTOR);
        }
        
        // 5. Vectorization
        // Process multiple iterations with SIMD
        if (canVectorize(loop)) {
            vectorize(loop);
        }
    }
}
```

### Deoptimization Infrastructure

```java
// Fast path to slow path transition

class Deoptimization {
    // Deopt point: return to interpreter with correct state
    
    static class DeoptInfo {
        int bci;                    // Bytecode index
        Object[] locals;            // Local variable values
        Object[] stack;             // Stack values  
        Object[] monitors;          // Held locks
    }
    
    void deoptimize(DeoptInfo info) {
        // 1. Rebuild interpreter frame
        InterpreterFrame frame = new InterpreterFrame();
        frame.setBCI(info.bci);
        frame.setLocals(info.locals);
        frame.setStack(info.stack);
        
        // 2. Re-acquire monitors
        for (Object monitor : info.monitors) {
            monitorEnter(monitor);
        }
        
        // 3. Resume in interpreter
        // (Much slower, but correct)
        interpreter.execute(frame);
        
        // 4. Maybe recompile with new profile info
        // The failed speculation taught us something
    }
}

// Key: compiled code can ALWAYS return to interpreter
// This makes speculative optimization safe
```

### Type Feedback System

```java
// Collect runtime type information

class TypeProfile {
    // At each call site, track observed receiver types
    TypeProfileEntry[] receivers = new TypeProfileEntry[2];
    int count;
    
    void recordType(Class<?> type) {
        for (int i = 0; i < count; i++) {
            if (receivers[i].type == type) {
                receivers[i].count++;
                return;
            }
        }
        
        if (count < receivers.length) {
            receivers[count++] = new TypeProfileEntry(type, 1);
        } else {
            // Too many types: mark megamorphic
            morphism = MEGAMORPHIC;
        }
    }
    
    OptimizationHint getHint() {
        if (count == 1 && receivers[0].count > THRESHOLD) {
            return new Monomorphic(receivers[0].type);
        }
        if (count == 2) {
            return new Bimorphic(receivers[0].type, receivers[1].type);
        }
        return MEGAMORPHIC;
    }
}

// Profile-guided optimization:
// 1. Run in interpreter, collect profiles
// 2. Compile hot methods with profile data
// 3. Speculate based on observed types
// 4. Deoptimize if speculation wrong
// 5. Recompile with updated profile
```

### Register Allocation

```java
// Graph coloring register allocation

class RegisterAllocator {
    void allocate(Graph graph) {
        // Build interference graph
        // Two values interfere if both live at same point
        InterferenceGraph ig = buildInterferenceGraph(graph);
        
        // Color graph with K colors (K = register count)
        // Adjacent nodes get different colors
        Map<Node, Integer> coloring = colorGraph(ig);
        
        // Handle spills
        // If can't color, spill some values to stack
        while (coloring == null) {
            Node toSpill = selectSpillCandidate(ig);
            insertSpillCode(toSpill);
            ig = rebuild(ig, toSpill);
            coloring = colorGraph(ig);
        }
        
        // Assign physical registers
        for (Map.Entry<Node, Integer> e : coloring.entrySet()) {
            e.getKey().setRegister(physicalRegister(e.getValue()));
        }
    }
}
```

## JIT Compilation Philosophy

```
Tiered Compilation Strategy
══════════════════════════════════════════════════════════════

Tier    Compiler    Optimization    When Used
────────────────────────────────────────────────────────────
0       Interpreter None           First execution
1       C1 (Client) Light          Moderate hotness
2       C2 (Server) Aggressive     Very hot methods

Compilation triggers:
- Method entry count threshold
- Loop back-edge count threshold
- On-stack replacement for hot loops

Key insight: Most code is cold
            Compile only what matters
            Quick startup, peak performance eventually
```

## Mental Model

Click approaches compiler design by asking:

1. **What can I prove?** Optimizations are proofs
2. **What's the common case?** Speculate on it
3. **What information do I need?** Preserve it in the IR
4. **What can be eliminated?** The fastest code is no code
5. **How do I recover when wrong?** Deoptimization must work

## Signature Click Moves

- **Sea-of-nodes IR**: Dependencies, not artificial order
- **Speculative optimization**: Bet on the common case
- **Escape analysis**: Eliminate allocations entirely
- **Global value numbering**: One computation per value
- **Profile-guided optimization**: Runtime feedback guides compilation
- **Tiered compilation**: Quick startup, peak performance later
- **Deoptimization**: Safe return to interpreter
- **Loop optimizations**: Range checks, unrolling, vectorization
