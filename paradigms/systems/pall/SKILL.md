---
name: pall-jit-mastery
description: Write high-performance JIT compilers and interpreters in the style of Mike Pall, creator of LuaJIT. Emphasizes trace-based compilation, aggressive optimization, and understanding CPU microarchitecture. Use when building JITs, interpreters, or any code where every cycle counts.
---

# Mike Pall Style Guide

## Overview

Mike Pall created LuaJIT, widely considered one of the most impressive JIT compilers ever written. A single developer achieved performance competitive with production JVMs while maintaining a tiny codebase. His work demonstrates that deep understanding of hardware and algorithms beats large teams with brute force.

## Core Philosophy

> "Measure, don't guess."

> "The fastest code is code that doesn't run."

> "Understand your hardware or it will humble you."

Pall believes in ruthless optimization through deep understanding—knowing the CPU so well that you can predict cycle counts by reading assembly.

## Design Principles

1. **Trace-Based Compilation**: Optimize what actually runs, not what might run.

2. **Microarchitecture Awareness**: Write code for the real CPU, not an abstract machine.

3. **Minimal Abstraction**: Every layer costs cycles.

4. **Data-Oriented Design**: Memory layout dominates performance.

## When Writing Performance-Critical Code

### Always

- Benchmark before and after every change
- Understand the generated assembly
- Profile to find actual hot spots
- Consider cache behavior for every data structure
- Know your target CPU's pipeline
- Test on multiple architectures

### Never

- Assume an optimization helps without measuring
- Ignore branch prediction effects
- Use abstractions that hide memory access patterns
- Optimize cold code paths
- Trust microbenchmarks for macro decisions
- Assume compiler optimizations happen

### Prefer

- Trace compilation over method compilation
- Linear memory access over pointer chasing
- Branchless code in hot paths
- Tables over computed branches
- Inline caching for polymorphic calls
- Specialized code paths over generic

## Code Patterns

### Trace-Based Compilation

```c
// LuaJIT's key insight: trace hot paths, not methods
// A trace is a linear sequence of operations

typedef struct Trace {
    uint32_t *mcode;       // Generated machine code
    IRIns *ir;             // IR instructions
    uint16_t nins;         // Number of IR instructions
    uint16_t nk;           // Number of constants
    SnapShot *snap;        // Side exit snapshots
    uint16_t nsnap;
    // Link to next trace (for loops)
    struct Trace *link;
} Trace;

// Recording: capture operations as they execute
void record_instruction(JitState *J, BCIns ins) {
    switch (bc_op(ins)) {
    case BC_ADDVN:
        // Record: result = slot[A] + constant[D]
        TRef tr = emitir(IR_ADD, J->slots[bc_a(ins)], 
                         lj_ir_knum(J, bc_d(ins)));
        J->slots[bc_a(ins)] = tr;
        break;
    // ... other bytecodes
    }
}

// Key: traces are LINEAR
// No control flow in the trace itself
// Side exits handle divergence
```

### IR Design for Speed

```c
// LuaJIT IR: compact, cache-friendly, no pointers

typedef struct IRIns {
    uint16_t op;       // Operation + type
    uint16_t op1;      // First operand (IR ref or slot)
    uint16_t op2;      // Second operand
    uint16_t prev;     // Previous instruction (for CSE chains)
} IRIns;  // 8 bytes, fits in cache line nicely

// IR references are indices, not pointers
// Enables: compact storage, easy serialization, cache efficiency

#define IRREF_BIAS  0x8000
#define irref_isk(r)  ((r) < IRREF_BIAS)  // Is constant?

// Constants stored separately, referenced by negative indices
// Instructions stored linearly, referenced by positive indices

// Example IR sequence for: x = a + b * c
// K001  NUM  3.14        -- constant
// 0001  SLOAD #1         -- load slot 1 (a)
// 0002  SLOAD #2         -- load slot 2 (b)
// 0003  SLOAD #3         -- load slot 3 (c)
// 0004  MUL  0002 0003   -- b * c
// 0005  ADD  0001 0004   -- a + (b * c)
```

### Side Exits and Guards

```c
// Traces assume types and values
// Guards verify assumptions, exit if wrong

void emit_guard(JitState *J, IRType expected, TRef tr) {
    IRIns *ir = &J->cur.ir[tref_ref(tr)];
    
    if (ir->t != expected) {
        // Emit type guard
        emitir(IR_GUARD, tr, expected);
        
        // Record snapshot for side exit
        snapshot_add(J);
    }
}

// Side exit: restore interpreter state, continue there
typedef struct SnapShot {
    uint16_t ref;        // First IR ref in snapshot
    uint8_t nslots;      // Number of slots to restore
    uint8_t topslot;     // Top slot number
    uint32_t *map;       // Slot -> IR ref mapping
} SnapShot;

// When guard fails:
// 1. Look up snapshot for this guard
// 2. Restore Lua stack from IR values
// 3. Jump back to interpreter
// 4. Maybe record a new trace from exit point
```

### Assembly-Level Optimization

```c
// LuaJIT generates assembly directly
// Every instruction chosen deliberately

// x86-64 code emission helpers
static void emit_rr(ASMState *as, x86Op op, Reg r1, Reg r2) {
    // REX prefix if needed
    if (r1 >= 8 || r2 >= 8) {
        *--as->mcp = 0x40 | ((r1 >> 3) << 2) | (r2 >> 3);
    }
    *--as->mcp = 0xc0 | ((r1 & 7) << 3) | (r2 & 7);
    *--as->mcp = op;
}

// Register allocation: linear scan, but smarter
// Allocate backwards from trace end for better results

void ra_allocate(ASMState *as) {
    // Process IR in reverse order
    for (IRRef ref = as->curins; ref >= as->stopins; ref--) {
        IRIns *ir = &as->ir[ref];
        
        // Allocate destination register
        Reg dest = ra_dest(as, ir);
        
        // Allocate source registers
        ra_left(as, ir, dest);
        ra_right(as, ir);
    }
}

// Key insight: backwards allocation sees all uses
// Can make better spill decisions
```

### Memory Access Patterns

```c
// Cache-friendly data structures are critical

// BAD: Linked list of variable-size nodes
struct Node {
    struct Node *next;
    int type;
    union {
        double num;
        struct String *str;
        // ...
    } value;
};

// GOOD: Separate arrays by type (SoA)
struct ValueArray {
    uint8_t *types;      // Type tags: sequential access
    TValue *values;      // Values: sequential access
    size_t count;
};

// Iteration patterns matter enormously
// This is ~10x faster than pointer chasing:
for (size_t i = 0; i < arr->count; i++) {
    if (arr->types[i] == TYPE_NUMBER) {
        sum += arr->values[i].n;
    }
}
```

### Inline Caching

```c
// Polymorphic inline cache for property access
// Avoids hash lookup in common case

typedef struct InlineCache {
    uint32_t shape_id;    // Expected object shape
    uint16_t offset;      // Cached property offset
    uint16_t _pad;
} InlineCache;

TValue get_property_cached(Object *obj, String *key, InlineCache *ic) {
    // Fast path: shape matches
    if (likely(obj->shape_id == ic->shape_id)) {
        return obj->slots[ic->offset];  // Direct access!
    }
    
    // Slow path: lookup and update cache
    uint16_t offset = shape_lookup(obj->shape, key);
    ic->shape_id = obj->shape_id;
    ic->offset = offset;
    return obj->slots[offset];
}

// Monomorphic: one shape, one offset
// Polymorphic: small set of shapes
// Megamorphic: too many shapes, fall back to hash
```

### Branch Prediction Awareness

```c
// CPUs predict branches; help them be right

// BAD: Unpredictable branches in hot loop
for (int i = 0; i < n; i++) {
    if (data[i] > threshold) {  // 50% taken = unpredictable
        sum += data[i];
    }
}

// GOOD: Branchless version
for (int i = 0; i < n; i++) {
    int mask = -(data[i] > threshold);  // 0 or -1
    sum += data[i] & mask;
}

// GOOD: Sort first if possible
qsort(data, n, sizeof(int), compare);
for (int i = 0; i < n && data[i] <= threshold; i++) {
    // All branches now predictable
}

// Loop unrolling: reduce branch overhead
for (int i = 0; i + 4 <= n; i += 4) {
    sum += data[i];
    sum += data[i + 1];
    sum += data[i + 2];
    sum += data[i + 3];
}
```

### Type Specialization

```c
// Generate specialized code for each type combination
// LuaJIT specializes aggressively

// Generic add (slow)
TValue generic_add(TValue a, TValue b) {
    if (tvisnum(a) && tvisnum(b)) {
        return numV(numV(a) + numV(b));
    } else if (tvisstr(a) || tvisstr(b)) {
        return concat(tostring(a), tostring(b));
    }
    // ... metamethod lookup
}

// Specialized add for numbers (fast)
// Generated when trace shows both args are numbers
double specialized_add_nn(double a, double b) {
    return a + b;  // Single instruction
}

// Type guards ensure specialization is valid
// Side exit if types don't match expected
```

## Performance Mental Model

```
CPU Pipeline Awareness
══════════════════════════════════════════════════════════════

Latency (cycles)    Operation
────────────────────────────────────────────────────────────
1                   Register-to-register ALU
3-4                 L1 cache hit
~12                 L2 cache hit
~40                 L3 cache hit
~200                Main memory
~10-20              Branch mispredict penalty
~100+               Page fault

Key insight: Memory is the bottleneck
            Computation is nearly free by comparison
            Optimize for memory access patterns first
```

## Mental Model

Pall approaches optimization by asking:

1. **What's the hot path?** Trace it, optimize it
2. **What does the assembly look like?** If you can't read it, you can't optimize it
3. **Where are the cache misses?** Memory dominates everything
4. **What are the branch patterns?** Predictable branches are free
5. **Can I specialize?** Generic code is slow code

## Signature Pall Moves

- **Trace compilation**: JIT what runs, not what's written
- **Compact IR**: 8-byte instructions, index-based references
- **Backwards register allocation**: See all uses before deciding
- **NaN boxing**: Encode type and value in 64-bit doubles
- **Side exit snapshots**: Restore interpreter state precisely
- **Assembly-level thinking**: Know the cost of every instruction
- **FFI that's actually fast**: C calls without overhead
