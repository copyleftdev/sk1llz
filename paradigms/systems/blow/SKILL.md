---
name: blow-compiler-gamedev
description: Design languages and write game engine code in the style of Jonathan Blow, creator of Braid, The Witness, and the Jai programming language. Emphasizes programmer productivity, compile-time execution, and rejecting complexity that doesn't serve the programmer. Use when designing languages, game engines, or performance-critical creative tools.
---

# Jonathan Blow Style Guide

## Overview

Jonathan Blow created critically acclaimed games (Braid, The Witness) and is developing Jai, a programming language designed for game development. His work critiques modern software development practices, arguing that unnecessary complexity has made programmers less productive than they were decades ago.

## Core Philosophy

> "Complexity is the enemy. Simplicity enables speed."

> "The language should do work for the programmer, not create work."

> "Good tools make hard things possible and easy things trivial."

Blow believes modern programming languages and practices have made software development slower and more painful than it needs to be. His language work aims to fix this.

## Design Principles

1. **Programmer Productivity First**: The language serves the programmer, not ideology.

2. **Compile-Time Power**: Move work from runtime to compile time.

3. **Zero Hidden Costs**: No implicit allocations, copies, or indirection.

4. **Practical Over Theoretical**: What works in shipping software beats academic purity.

## When Designing Languages/Systems

### Always

- Make common operations trivial to express
- Provide compile-time execution for metaprogramming
- Keep syntax simple and readable
- Enable low-level control when needed
- Design for fast compilation
- Support incremental compilation

### Never

- Hide costs from the programmer
- Require boilerplate for simple tasks
- Make error messages cryptic
- Force one paradigm when another fits better
- Sacrifice programmer productivity for language purity
- Add features without clear benefit

### Prefer

- Compile-time over runtime computation
- Explicit over implicit behavior
- Built-in metaprogramming over external tools
- Fast iteration over perfect safety
- Practical defaults over configurable everything
- Struct-of-arrays support in the language

## Code Patterns

### Compile-Time Execution (Jai Concept)

```jai
// Jai: run any code at compile time with #run

// Generate a lookup table at compile time
SINE_TABLE :: #run generate_sine_table();

generate_sine_table :: () -> [256]float {
    result: [256]float;
    for i: 0..255 {
        result[i] = sin(cast(float)i / 256.0 * TAU);
    }
    return result;
}

// Use at runtime: zero computation, just table lookup
fast_sin :: (x: float) -> float {
    index := cast(int)(x / TAU * 256) & 255;
    return SINE_TABLE[index];
}

// #run can execute ANY code:
// - Read files
// - Call external programs
// - Generate code
// - Compute constants
// No separate macro language needed
```

### Zero-Cost Iteration

```jai
// Jai: for loops that understand your data

// Iterate array
for values {
    print("%\n", it);  // 'it' is implicit iterator
}

// With index
for value, index: values {
    print("[%] = %\n", index, value);
}

// Iterate by pointer (no copy)
for *value: values {
    value.x += 1;  // Modifies in place
}

// Reverse iteration
for < values {
    print("%\n", it);  // Last to first
}

// The compiler knows the iteration pattern
// No iterator objects, no virtual dispatch
// Compiles to simple pointer arithmetic
```

### SOA/AOS Flexibility

```jai
// Struct definition works either way
Entity :: struct {
    position: Vector3;
    velocity: Vector3;
    health: float;
    flags: u32;
}

// Array of Structures (typical)
entities_aos: [1000]Entity;

// Structure of Arrays (Jai native support)
entities_soa: SOA [1000]Entity;

// Access looks the same
entities_aos[5].position.x = 10;
entities_soa[5].position.x = 10;

// But memory layout differs:
// AOS: [pos vel health flags][pos vel health flags]...
// SOA: [pos pos pos...][vel vel vel...][health health...]

// SOA is better for SIMD, cache efficiency
// Language handles the transformation
```

### Explicit Memory Control

```jai
// No hidden allocations
// Programmer chooses memory strategy

// Stack allocation (default)
buffer: [1024]u8;

// Explicit heap
data := alloc(1024);
defer free(data);  // Deterministic cleanup

// Custom allocator
game_allocator: Allocator;
entity := alloc(Entity, allocator = game_allocator);

// Temporary allocation (frame allocator)
temp_string := tprint("Value: %", value);
// Automatically freed at frame end

// No garbage collection
// No hidden reference counting
// You know exactly what memory does
```

### Code Modification at Compile Time

```jai
// Modify/generate code during compilation

#insert :: (code: string) -> void {
    // Insert generated code at this point
}

// Generate struct fields
Vector :: struct {
    #insert #run generate_components(3);  // Generates x, y, z
}

generate_components :: (n: int) -> string {
    builder: String_Builder;
    for i: 0..n-1 {
        name := cast(u8)('x' + i);
        print_to_builder(*builder, "%: float;\n", to_string(*name, 1));
    }
    return builder_to_string(*builder);
}

// Result equivalent to:
// Vector :: struct { x: float; y: float; z: float; }

// Full language available for metaprogramming
// Not a limited macro DSL
```

### Fast Compile Times

```jai
// Jai designed for fast compilation from the start

// Module system: no header files
// Just import what you need
#import "Basic";
#import "Math";

// Incremental compilation built-in
// Change one file, rebuild only what's affected

// No template instantiation explosion
// Polymorphism without code bloat

// Typical game project:
// C++: minutes to build
// Jai: seconds to build

// Fast iteration = more experiments = better code
```

### Explicit Polymorphism

```jai
// Polymorphism without hidden vtables

// Type-parametric (like templates, but cleaner)
Array :: struct(T: Type) {
    data: *T;
    count: int;
    allocated: int;
}

push :: (array: *Array($T), value: T) {
    if array.count >= array.allocated {
        grow(array);
    }
    array.data[array.count] = value;
    array.count += 1;
}

// $T means: infer type from usage
// Generates specialized code, no runtime dispatch

// Interface polymorphism when needed
Drawable :: struct {
    draw: (self: *Drawable) -> void;
}

// But it's explicit: you see the function pointer
// No hidden vtable magic
```

### Error Handling Without Exceptions

```jai
// Multiple return values for errors

read_file :: (path: string) -> string, bool {
    file, success := open(path);
    if !success return "", false;
    defer close(file);
    
    contents := read_entire_file(file);
    return contents, true;
}

// Usage
contents, ok := read_file("config.txt");
if !ok {
    log_error("Failed to read config");
    return;
}

// Or with 'if' initialization
if contents, ok := read_file("config.txt"); ok {
    process(contents);
} else {
    handle_error();
}

// No exception overhead
// No hidden control flow
// Errors are values, handled explicitly
```

### Game Loop Clarity

```jai
// Clear, explicit game loop
// No framework hiding what happens

main :: () {
    init_window(1920, 1080, "Game");
    defer deinit_window();
    
    game_state: GameState;
    init_game(*game_state);
    
    while !should_quit() {
        // Fixed timestep
        dt :: 1.0 / 60.0;
        
        // Input
        input := get_input();
        
        // Update
        update_game(*game_state, input, dt);
        
        // Render
        begin_frame();
        render_game(*game_state);
        end_frame();
        
        // Frame timing
        wait_for_frame_end();
    }
}

// Everything visible
// No hidden callbacks or event systems
// Easy to understand, debug, and profile
```

## Language Design Philosophy

```
Jai Design Priorities
══════════════════════════════════════════════════════════════

Priority    Feature                  Why
────────────────────────────────────────────────────────────
1           Compile speed            Fast iteration
2           Runtime speed            Games need performance  
3           Programmer joy           Code should feel good
4           Compile-time execution   Metaprogramming done right
5           Explicit over implicit   No hidden behavior

Anti-priorities:
- Academic purity
- Backward compatibility with C++
- Making all errors compile-time errors
- Preventing all possible bugs

Philosophy: Trust the programmer, give them tools,
           don't slow them down "for their own good"
```

## Mental Model

Blow approaches language and system design by asking:

1. **Does this help ship software?** Features must serve real work
2. **What's the cost to the programmer?** Every feature has UX implications
3. **Can this run at compile time?** Move work earlier when possible
4. **Is this complexity justified?** Simple solutions often exist
5. **Will this make iteration faster?** Speed of development matters

## Signature Blow Moves

- **#run anything**: Full language available at compile time
- **SOA built-in**: Data layout without manual transformation
- **No header files**: Module system that just works
- **Explicit allocators**: Control memory without boilerplate
- **Fast compilation**: Seconds not minutes
- **Multiple return values**: Error handling without exceptions
- **Named arguments**: Readable function calls
- **Defer statement**: Cleanup without RAII complexity
- **Critique of complexity**: Question "best practices"
