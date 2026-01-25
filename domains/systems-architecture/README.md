# Systems Architecture Masters

> "A distributed system is one in which the failure of a computer you didn't even know existed can render your own computer unusable." — Leslie Lamport

## The Pantheon

### The Theorist

- **[Leslie Lamport](lamport/)** — Paxos, Logical Clocks, TLA+
  - *Focus*: Rigorous distributed systems reasoning, formal specification, consensus protocols

### The Practitioner at Scale

- **[Jeff Dean](dean/)** — MapReduce, BigTable, Spanner, TensorFlow
  - *Focus*: Large-scale system design, performance optimization, building reliable systems from unreliable components

### The Transactionalist

- **[Jim Gray](gray/)** — ACID, Transaction Processing, Fault Tolerance
  - *Focus*: Data integrity, recovery, the science of reliable systems

### The Architect's Architect

- **[Butler Lampson](lampson/)** — Alto, Ethernet, System Design Hints
  - *Focus*: Abstraction layering, security architecture, design principles

### The Cloud Native

- **[Werner Vogels](vogels/)** — AWS, Dynamo, Eventual Consistency
  - *Focus*: Failure-first design, CAP trade-offs, API-driven architecture

## Shared Principles

These masters converge on:

1. **Failure is Normal**: Design for failure, not against it
2. **Simplicity Wins**: The simplest system that could work often does
3. **Measure Everything**: Intuition fails at scale; data doesn't
4. **Abstractions Have Costs**: Understand what's beneath your abstraction
5. **State is the Enemy**: Minimize, isolate, and make state explicit

## The Fundamental Trade-offs

Every systems architect navigates:

| Trade-off | Tension |
|-----------|---------|
| **Consistency vs. Availability** | CAP theorem constraints |
| **Latency vs. Throughput** | Batching, caching, pipelining |
| **Simplicity vs. Performance** | When to optimize |
| **Generality vs. Specialization** | Reuse vs. purpose-built |

## How to Use These Skills

1. **Start with Lamport** for rigorous thinking about distributed state and time
2. **Add Dean** for practical large-scale system patterns
3. **Layer Gray** when data integrity and transactions matter
4. **Apply Lampson** for architecture decisions and API design
5. **Finish with Vogels** for cloud-native and failure-tolerant thinking

## Key Papers

| Paper | Author | Concept |
|-------|--------|---------|
| "Time, Clocks, and the Ordering of Events" | Lamport | Logical time |
| "The Part-Time Parliament" (Paxos) | Lamport | Consensus |
| "MapReduce: Simplified Data Processing" | Dean & Ghemawat | Large-scale computation |
| "Bigtable: A Distributed Storage System" | Dean et al. | Structured storage at scale |
| "The Transaction Concept" | Gray | ACID formalization |
| "Hints for Computer System Design" | Lampson | Design principles |
| "Dynamo: Amazon's Highly Available Key-value Store" | Vogels et al. | Eventual consistency |

## Additional Resources

- [Designing Data-Intensive Applications](https://dataintensive.net/) — Martin Kleppmann
- [Google SRE Book](https://sre.google/sre-book/table-of-contents/)
- [AWS Architecture Center](https://aws.amazon.com/architecture/)
