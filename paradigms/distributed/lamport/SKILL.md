---
name: lamport-formal-distributed
description: Design distributed systems in the style of Leslie Lamport, creator of Paxos, TLA+, and LaTeX. Emphasizes formal specification, logical time, and rigorous reasoning about concurrent systems. Use when designing consensus protocols or proving system correctness.
---

# Leslie Lamport Style Guide

## Overview

Leslie Lamport is a Turing Award winner who invented logical clocks, the Paxos consensus algorithm, TLA+ specification language, and LaTeX. His work forms the theoretical foundation of modern distributed systems.

## Core Philosophy

> "A distributed system is one in which the failure of a computer you didn't even know existed can render your own computer unusable."

> "If you're thinking without writing, you only think you're thinking."

> "The way to get correct programs is to first get something that is obviously correct and then make it efficient."

Lamport believes that distributed systems are too complex to reason about informally. Formal specification isn't optional—it's essential.

## Design Principles

1. **Formal Specification First**: Write the spec before the code.

2. **Time Is Logical, Not Physical**: Use happens-before, not wall clocks.

3. **Safety Before Liveness**: First ensure nothing bad happens, then ensure something good does.

4. **State Machines**: Model systems as state machines for clarity.

## When Writing Code

### Always

- Write a formal specification (TLA+ or similar)
- Define safety and liveness properties explicitly
- Use logical timestamps for ordering events
- Model failures as part of the specification
- Prove correctness before implementing
- Consider all interleavings

### Never

- Assume reliable networks
- Rely on synchronized clocks
- Ignore failure modes
- Test into correctness (testing finds bugs, not proves absence)
- Hand-wave about "eventual consistency"

### Prefer

- State machine specifications
- Logical clocks over physical clocks
- Consensus protocols over ad-hoc coordination
- Formal proofs over informal arguments
- Explicit failure handling

## Code Patterns

### Logical Clocks (Lamport Timestamps)

```python
# Lamport's logical clock: happens-before ordering

class LamportClock:
    def __init__(self):
        self.time = 0
    
    def tick(self):
        """Local event: increment clock"""
        self.time += 1
        return self.time
    
    def send(self):
        """Send message: increment and return timestamp"""
        self.time += 1
        return self.time
    
    def receive(self, msg_timestamp):
        """Receive message: max(local, received) + 1"""
        self.time = max(self.time, msg_timestamp) + 1
        return self.time

# Usage:
# Process A: clock.tick() -> 1, clock.send() -> 2
# Process B: clock.receive(2) -> 3
# Now we know: A's event 2 happened-before B's event 3
```

### Vector Clocks (Causal Ordering)

```python
# Vector clocks: detect concurrent events

class VectorClock:
    def __init__(self, node_id, num_nodes):
        self.node_id = node_id
        self.clock = [0] * num_nodes
    
    def tick(self):
        """Local event"""
        self.clock[self.node_id] += 1
        return self.clock.copy()
    
    def send(self):
        """Send: increment own component"""
        self.clock[self.node_id] += 1
        return self.clock.copy()
    
    def receive(self, other_clock):
        """Receive: element-wise max, then increment own"""
        for i in range(len(self.clock)):
            self.clock[i] = max(self.clock[i], other_clock[i])
        self.clock[self.node_id] += 1
        return self.clock.copy()
    
    @staticmethod
    def compare(vc1, vc2):
        """Compare: <, >, =, or concurrent"""
        less = all(a <= b for a, b in zip(vc1, vc2))
        greater = all(a >= b for a, b in zip(vc1, vc2))
        
        if less and not greater:
            return "before"  # vc1 happened-before vc2
        elif greater and not less:
            return "after"   # vc1 happened-after vc2
        elif less and greater:
            return "equal"
        else:
            return "concurrent"  # Neither happened-before the other
```

### TLA+ Specification

```tla
--------------------------- MODULE SimpleConsensus ---------------------------
EXTENDS Integers, FiniteSets

CONSTANTS Nodes, Values

VARIABLES 
    proposed,   \* proposed[n] = value proposed by node n
    decided     \* decided[n] = value decided by node n (or null)

TypeInvariant ==
    /\ proposed \in [Nodes -> Values \union {NULL}]
    /\ decided \in [Nodes -> Values \union {NULL}]

\* Safety: Agreement - all decided values are the same
Agreement ==
    \A n1, n2 \in Nodes:
        (decided[n1] # NULL /\ decided[n2] # NULL) =>
            decided[n1] = decided[n2]

\* Safety: Validity - decided value was proposed
Validity ==
    \A n \in Nodes:
        decided[n] # NULL => 
            \E m \in Nodes: proposed[m] = decided[n]

\* Liveness: Termination - eventually all decide
Termination ==
    <>(\A n \in Nodes: decided[n] # NULL)

Init ==
    /\ proposed = [n \in Nodes |-> NULL]
    /\ decided = [n \in Nodes |-> NULL]

Propose(n, v) ==
    /\ proposed[n] = NULL
    /\ proposed' = [proposed EXCEPT ![n] = v]
    /\ UNCHANGED decided

Decide(n, v) ==
    /\ decided[n] = NULL
    /\ \E m \in Nodes: proposed[m] = v
    /\ decided' = [decided EXCEPT ![n] = v]
    /\ UNCHANGED proposed

Next ==
    \E n \in Nodes, v \in Values:
        Propose(n, v) \/ Decide(n, v)

Spec == Init /\ [][Next]_<<proposed, decided>>
==============================================================================
```

### Paxos Simplified

```python
# Simplified Paxos - single decree

class PaxosNode:
    def __init__(self, node_id, nodes):
        self.node_id = node_id
        self.nodes = nodes
        
        # Acceptor state
        self.promised = 0      # Highest proposal number promised
        self.accepted_num = 0  # Number of accepted proposal
        self.accepted_val = None
        
    def prepare(self, proposal_num):
        """Phase 1a: Proposer sends prepare"""
        responses = []
        for node in self.nodes:
            resp = node.handle_prepare(proposal_num)
            if resp:
                responses.append(resp)
        return responses
    
    def handle_prepare(self, proposal_num):
        """Phase 1b: Acceptor handles prepare"""
        if proposal_num > self.promised:
            self.promised = proposal_num
            return {
                'promised': True,
                'accepted_num': self.accepted_num,
                'accepted_val': self.accepted_val
            }
        return {'promised': False}
    
    def accept(self, proposal_num, value):
        """Phase 2a: Proposer sends accept"""
        responses = []
        for node in self.nodes:
            resp = node.handle_accept(proposal_num, value)
            responses.append(resp)
        return responses
    
    def handle_accept(self, proposal_num, value):
        """Phase 2b: Acceptor handles accept"""
        if proposal_num >= self.promised:
            self.promised = proposal_num
            self.accepted_num = proposal_num
            self.accepted_val = value
            return {'accepted': True}
        return {'accepted': False}
    
    def propose(self, value):
        """Full Paxos proposal"""
        n = self.generate_proposal_number()
        
        # Phase 1: Prepare
        promises = self.prepare(n)
        if len([p for p in promises if p['promised']]) <= len(self.nodes) // 2:
            return None  # Failed to get majority
        
        # Use highest accepted value if any
        highest = max(promises, key=lambda p: p['accepted_num'])
        if highest['accepted_val'] is not None:
            value = highest['accepted_val']
        
        # Phase 2: Accept
        accepts = self.accept(n, value)
        if len([a for a in accepts if a['accepted']]) > len(self.nodes) // 2:
            return value  # Consensus reached!
        
        return None
```

### State Machine Replication

```python
# State machine replication: the foundation of distributed consensus

class ReplicatedStateMachine:
    """
    Key insight: if all replicas start in the same state
    and apply the same operations in the same order,
    they will end up in the same state.
    """
    
    def __init__(self):
        self.state = {}
        self.log = []        # Ordered log of operations
        self.commit_index = 0
    
    def apply(self, operation):
        """Apply operation to state machine"""
        op_type = operation['type']
        
        if op_type == 'set':
            self.state[operation['key']] = operation['value']
        elif op_type == 'delete':
            self.state.pop(operation['key'], None)
        elif op_type == 'increment':
            key = operation['key']
            self.state[key] = self.state.get(key, 0) + 1
    
    def append_to_log(self, operation):
        """Add operation to log (not yet applied)"""
        self.log.append(operation)
        return len(self.log) - 1  # Return log index
    
    def commit(self, index):
        """Apply all operations up to index"""
        while self.commit_index <= index:
            self.apply(self.log[self.commit_index])
            self.commit_index += 1
    
    # The consensus protocol ensures all replicas
    # have the same log in the same order
```

## Mental Model

Lamport approaches distributed systems by asking:

1. **What is the state?** Define the state machine precisely
2. **What are the safety properties?** What must never happen?
3. **What are the liveness properties?** What must eventually happen?
4. **How do we order events?** Logical time, not physical time
5. **Can I specify this formally?** If not, I don't understand it

## Signature Lamport Moves

- TLA+ specifications before implementation
- Logical clocks for event ordering
- Safety and liveness as separate concerns
- State machine replication for consensus
- Happens-before reasoning
- Formal proofs of correctness
