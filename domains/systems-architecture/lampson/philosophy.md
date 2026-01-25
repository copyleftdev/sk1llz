# Butler Lampson: Philosophy & Mental Models

## The Architect's Architect

Butler Lampson has been building systems for over 50 years. His insights come from decades of watching what works, what fails, and what lasts. He doesn't just build systems—he thinks about how to think about building systems.

## Core Beliefs

### 1. Abstractions Must Earn Their Keep

Every abstraction has costs:
- Conceptual overhead (another thing to learn)
- Performance overhead (another layer of indirection)
- Rigidity (harder to change once adopted)

An abstraction must provide enough value to justify these costs. Most proposed abstractions fail this test.

### 2. Interfaces are Contracts

> "An interface is a contract between the implementer and the client."

The contract specifies:
- What the implementer promises to do
- What the client promises not to depend on
- What happens when things go wrong

Once a contract is published, it's very hard to change. Design interfaces for the long term.

### 3. Simplicity is Not Simple

> "Simplicity does not precede complexity, but follows it."

True simplicity requires:
- Deep understanding of the problem
- Ruthless removal of non-essentials
- Elegant handling of edge cases

Simple solutions are hard to find. Complex solutions are easy.

### 4. Security is Architecture

Security cannot be:
- Added as a layer
- Checked by a separate team
- Bolted on after deployment

Security must be:
- Designed into the architecture
- Expressed in the abstraction
- Verified throughout development

### 5. Systems Have Lifetimes

A system designed for 1 year is different from one designed for 10 years:

| Lifetime | Priorities |
|----------|-----------|
| 1 year | Speed of development, current features |
| 5 years | Maintainability, team changes |
| 10 years | Interface stability, data migration |
| 20+ years | Format preservation, technology changes |

Design for the appropriate lifetime.

## On Abstractions

### What Makes a Good Abstraction?

A good abstraction:
1. **Hides complexity** that users don't need
2. **Exposes power** that users do need
3. **Has clear semantics** that users can reason about
4. **Maintains invariants** without user effort
5. **Evolves gracefully** as requirements change

### The Abstraction Barrier

The abstraction barrier separates:
- **Above**: What users see (interface)
- **Below**: How it works (implementation)

Nothing should cross this barrier except through the defined interface. If users peek below, the abstraction has failed.

### When Abstractions Leak

Abstractions leak when:
- Performance depends on hidden details
- Errors expose internal structure
- Users must understand implementation to use correctly
- Changes below require changes above

Leaked abstractions are worse than no abstraction.

## On Interfaces

### The Principle of Least Astonishment

Users should be able to predict behavior from the interface:
- Similar operations should work similarly
- Error handling should be consistent
- Side effects should be obvious

If users are surprised, the interface is wrong.

### Interface Completeness

An interface should be:
- **Minimal**: No unnecessary operations
- **Complete**: Everything users need
- **Orthogonal**: Operations don't overlap

Finding this balance is the art of interface design.

### Interface Evolution

Interfaces should be designed for evolution:
- Optional parameters over new methods
- Versioning strategy from day one
- Deprecation path for old features
- Backward compatibility as default

But accept that sometimes you must break compatibility to make progress.

## On Performance

### The Right Time to Optimize

1. **First**: Make it work (correct)
2. **Then**: Make it right (clean)
3. **Finally**: Make it fast (optimized)

In that order. Always.

### The Common Case

> "Handle the common case well. The rare case should work, but can be slow."

Identify the common case through measurement, not intuition. Optimize that path aggressively. Let rare cases be slow—users will tolerate occasional slowness, not consistent slowness.

### Hints, Not Commands

A hint is information that:
- Speeds up computation
- Is not required for correctness
- Can be wrong without breaking anything

Examples:
- Cache contents (might be stale)
- Branch predictions (might be wrong)
- Prefetch targets (might not be needed)

Hints make common cases fast without complicating rare cases.

## On Security

### The Principle of Least Privilege

Every component should have only the privileges it needs:
- No more (limits damage from compromise)
- No less (must be able to function)

Capabilities naturally enforce this; ACLs fight against it.

### The Confused Deputy Problem

A "deputy" is a program that acts on behalf of multiple principals with different rights. Without capabilities:

```
User → Deputy → Resource
         ↑
      Attacker uses Deputy's privileges
```

Capabilities solve this: the deputy passes the user's capability, not its own.

### Defense in Depth

No single mechanism provides security. Layer defenses:
1. **Perimeter**: Firewalls, authentication
2. **Application**: Input validation, authorization
3. **Data**: Encryption, access control
4. **Monitoring**: Logging, anomaly detection

Each layer assumes the others might fail.

## Key Quotes

> "All problems in computer science can be solved by another level of indirection... except for the problem of too many layers of indirection."

> "An implementation must live with its published interface. So must its clients."

> "It is easier to change the specification to fit the program than vice versa."

> "Keep secrets of the implementation from the user of the abstraction."

> "Do one thing at a time, and do it well."

## The Lampson Test

Before finalizing a design, ask:
1. Can I explain the interface in one paragraph?
2. Do users need to understand the implementation?
3. Can the implementation change without affecting users?
4. Is security designed in or bolted on?
5. Will this interface still work in 10 years?

If any answer is unsatisfactory, reconsider the design.
