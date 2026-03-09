# Rust Masters⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍​‌‌​‌​​‌‍​​‌​‌​​‌‍‌‌​‌‌‌‌​‍‌‌​​​​​‌‍​​​​‌​‌​‍‌​​‌‌​‌‌⁠‍⁠

> "Fearless concurrency." — The Rust Promise

## The Pantheon

### The Creator

- **[Graydon Hoare](hoare/)** — Original creator of Rust
  - *Focus*: Memory safety without garbage collection, systems programming

### Language Architects

- **[Niko Matsakis](matsakis/)** — Language team lead, borrow checker architect
  - *Focus*: Ownership system, lifetimes, type system design

- **[Aaron Turon](turon/)** — Former Rust team lead
  - *Focus*: Async/await, API design, ecosystem vision

### The Educators

- **[Steve Klabnik](klabnik/)** — Author of "The Rust Programming Language"
  - *Focus*: Teaching Rust, documentation, community building

- **[Carol Nichols](nichols/)** — Co-author of "The Rust Book", crates.io
  - *Focus*: Clear explanations, practical patterns, ecosystem

### Deep Systems

- **[Mara Bos](bos/)** — Library team lead, author of "Rust Atomics and Locks"
  - *Focus*: Concurrency primitives, low-level systems, atomics

## Shared Principles

Rust's designers united around:

1. **Memory Safety**: No dangling pointers, no data races—guaranteed at compile time
2. **Zero-Cost Abstractions**: High-level code compiles to efficient machine code
3. **Fearless Concurrency**: The type system prevents data races
4. **Ownership**: Every value has exactly one owner
5. **Explicitness**: No hidden allocations, no implicit copies

## The Rust Guarantees

```
Memory safety without garbage collection.
Concurrency without data races.
Abstraction without overhead.
Stability without stagnation.
```

## Core Concepts

| Concept | Meaning |
|---------|---------|
| **Ownership** | Each value has one owner; when owner goes out of scope, value is dropped |
| **Borrowing** | References that don't take ownership (`&T`, `&mut T`) |
| **Lifetimes** | Compiler-tracked duration that references are valid |
| **Move Semantics** | Values are moved by default, not copied |
| **RAII** | Resources freed when owners go out of scope |
| **Send/Sync** | Traits that enable safe concurrency |

## How to Use These Skills

1. **Start with Hoare** for the foundational "why" of Rust
2. **Add Klabnik/Nichols** for idiomatic patterns and teaching
3. **Layer Matsakis** for deep ownership/lifetime understanding
4. **Apply Turon** for async and API design
5. **Consult Bos** for concurrent and low-level code

## Additional Resources

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) (unsafe Rust)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [This Week in Rust](https://this-week-in-rust.org/)
