# Distributed Systems Masters

> "A distributed system is one in which the failure of a computer you didn't even know existed can render your own computer unusable." — Leslie Lamport

## The Pantheon

### The Theorist

- **[Leslie Lamport](lamport/)** — Paxos, TLA+, logical clocks, Turing Award winner
  - *Focus*: Formal verification, consensus, time in distributed systems

### The Practitioner

- **[Jeff Dean](dean/)** — MapReduce, BigTable, Spanner, TensorFlow at Google
  - *Focus*: Large-scale systems, performance, practical distributed computing

### The Educator

- **[Martin Kleppmann](kleppmann/)** — Author of "Designing Data-Intensive Applications"
  - *Focus*: Data systems, replication, consistency models, teaching

## Shared Principles

These masters agree on:

1. **Failures Are Normal**: Design for failure, not around it
2. **Consistency Is Complex**: Understand CAP, PACELC, linearizability
3. **Time Is Tricky**: No global clock, only happens-before
4. **Verify Formally**: Complex systems need formal reasoning
5. **Measure Everything**: You can't improve what you don't measure

## The Eight Fallacies of Distributed Computing

```
1. The network is reliable
2. Latency is zero
3. Bandwidth is infinite
4. The network is secure
5. Topology doesn't change
6. There is one administrator
7. Transport cost is zero
8. The network is homogeneous
```

## How to Use These Skills

1. **Start with Lamport** for theoretical foundations and formal thinking
2. **Add Dean** for practical large-scale system design
3. **Layer Kleppmann** for understanding data systems deeply

## Additional Resources

- [Designing Data-Intensive Applications](https://dataintensive.net/)
- [TLA+ Resources](https://lamport.azurewebsites.net/tla/tla.html)
- [Google Research Papers](https://research.google/pubs/)
