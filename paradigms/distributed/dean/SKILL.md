---
name: dean-large-scale-systems
description: Design distributed systems in the style of Jeff Dean, Google Senior Fellow behind MapReduce, BigTable, Spanner, and TensorFlow. Emphasizes practical large-scale system design, performance optimization, and building infrastructure that serves billions. Use when designing systems that must scale massively.
---

# Jeff Dean Style Guide

## Overview

Jeff Dean is a Google Senior Fellow who has architected some of the most influential distributed systems: MapReduce, BigTable, Spanner, TensorFlow, and more. He approaches problems at a scale most engineers never encounter, yet his solutions are elegant and practical.

## Core Philosophy

> "Design for 10x growth, but plan to rewrite before 100x."

> "Latency at the 99th percentile matters more than the average."

> "Numbers every programmer should know."

Dean combines deep systems knowledge with practical engineering. He knows the numbers and designs systems that work at planet scale.

## Design Principles

1. **Design for Scale**: Build for 10x, rewrite before 100x.

2. **Tail Latency Matters**: Optimize the 99th percentile.

3. **Simple Abstractions**: MapReduce, BigTable—simple interfaces, complex implementation.

4. **Measure Everything**: You can't optimize what you don't measure.

## Numbers Every Programmer Should Know

```
L1 cache reference:                    0.5 ns
Branch mispredict:                     5 ns
L2 cache reference:                    7 ns
Mutex lock/unlock:                     25 ns
Main memory reference:                 100 ns
Compress 1KB with Zippy:               3,000 ns
Send 1KB over 1 Gbps network:          10,000 ns
Read 4KB randomly from SSD:            150,000 ns
Read 1MB sequentially from memory:     250,000 ns
Round trip within datacenter:          500,000 ns
Read 1MB sequentially from SSD:        1,000,000 ns
Disk seek:                             10,000,000 ns
Read 1MB sequentially from disk:       20,000,000 ns
Send packet CA->Netherlands->CA:       150,000,000 ns
```

## When Writing Code

### Always

- Know the numbers for your target platform
- Design for partial failure
- Add instrumentation from day one
- Plan for 10x growth
- Optimize for the common case
- Use tail-tolerant techniques

### Never

- Ignore tail latency
- Design without measuring
- Assume networks are reliable
- Build without considering failure modes
- Optimize prematurely, but don't ignore performance

### Prefer

- Batching over individual requests
- Parallel fanout with hedged requests
- Simple APIs over complex ones
- Idempotent operations
- Graceful degradation

## Code Patterns

### MapReduce Pattern

```python
# MapReduce: simple abstraction for parallel processing

from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor

def map_reduce(data, mapper, reducer, num_workers=4):
    """
    MapReduce framework:
    1. Map: transform each input to (key, value) pairs
    2. Shuffle: group by key
    3. Reduce: aggregate values for each key
    """
    
    # Map phase: parallel
    with ProcessPoolExecutor(max_workers=num_workers) as executor:
        map_results = list(executor.map(mapper, data))
    
    # Shuffle phase: group by key
    shuffled = defaultdict(list)
    for result in map_results:
        for key, value in result:
            shuffled[key].append(value)
    
    # Reduce phase: parallel
    with ProcessPoolExecutor(max_workers=num_workers) as executor:
        reduce_input = [(key, values) for key, values in shuffled.items()]
        final_results = list(executor.map(
            lambda kv: (kv[0], reducer(kv[0], kv[1])), 
            reduce_input
        ))
    
    return dict(final_results)


# Example: word count
def word_count_mapper(line):
    return [(word.lower(), 1) for word in line.split()]

def word_count_reducer(word, counts):
    return sum(counts)

# Usage
lines = ["Hello World", "Hello MapReduce", "World of Distributed Systems"]
result = map_reduce(lines, word_count_mapper, word_count_reducer)
# {'hello': 2, 'world': 2, 'mapreduce': 1, 'of': 1, 'distributed': 1, 'systems': 1}
```

### Tail-Tolerant Techniques

```python
import asyncio
import random

async def hedged_request(replicas, timeout_ms=10):
    """
    Hedged requests: send to multiple replicas, use first response.
    Dramatically reduces tail latency.
    """
    async def make_request(replica):
        latency = random.uniform(1, 100)  # Simulated
        await asyncio.sleep(latency / 1000)
        return f"Response from {replica}"
    
    # Start with one request
    tasks = [asyncio.create_task(make_request(replicas[0]))]
    
    try:
        # Wait briefly for first response
        done, pending = await asyncio.wait(
            tasks, 
            timeout=timeout_ms/1000,
            return_when=asyncio.FIRST_COMPLETED
        )
        
        if done:
            return done.pop().result()
        
        # Hedge: send to backup replicas
        for replica in replicas[1:]:
            tasks.append(asyncio.create_task(make_request(replica)))
        
        done, pending = await asyncio.wait(
            tasks,
            return_when=asyncio.FIRST_COMPLETED
        )
        
        # Cancel pending requests
        for task in pending:
            task.cancel()
        
        return done.pop().result()
        
    except Exception as e:
        return None


async def request_with_backup(primary, backup, timeout_ms=50):
    """
    Backup requests: if primary is slow, try backup.
    """
    try:
        return await asyncio.wait_for(
            fetch(primary), 
            timeout=timeout_ms/1000
        )
    except asyncio.TimeoutError:
        return await fetch(backup)
```

### BigTable-Style Design

```python
# BigTable: sorted string table with column families

class SSTable:
    """Immutable sorted string table"""
    
    def __init__(self, data):
        # Data is sorted by key
        self.data = sorted(data.items())
        self.index = self._build_index()
    
    def _build_index(self):
        """Sparse index: every Nth key"""
        return {k: i for i, (k, _) in enumerate(self.data) if i % 100 == 0}
    
    def get(self, key):
        """Binary search for key"""
        lo, hi = 0, len(self.data)
        while lo < hi:
            mid = (lo + hi) // 2
            if self.data[mid][0] < key:
                lo = mid + 1
            else:
                hi = mid
        
        if lo < len(self.data) and self.data[lo][0] == key:
            return self.data[lo][1]
        return None


class LSMTree:
    """Log-Structured Merge Tree: write-optimized storage"""
    
    def __init__(self):
        self.memtable = {}          # In-memory writes
        self.sstables = []          # Immutable on-disk tables
        self.memtable_limit = 1000
    
    def put(self, key, value):
        """Write to memtable (fast!)"""
        self.memtable[key] = value
        
        if len(self.memtable) >= self.memtable_limit:
            self._flush()
    
    def _flush(self):
        """Flush memtable to disk as SSTable"""
        sstable = SSTable(self.memtable)
        self.sstables.append(sstable)
        self.memtable = {}
        
        if len(self.sstables) > 4:
            self._compact()
    
    def _compact(self):
        """Merge SSTables to reduce read amplification"""
        merged = {}
        for sstable in self.sstables:
            for key, value in sstable.data:
                merged[key] = value
        
        self.sstables = [SSTable(merged)]
    
    def get(self, key):
        """Read: check memtable, then SSTables (newest first)"""
        if key in self.memtable:
            return self.memtable[key]
        
        for sstable in reversed(self.sstables):
            value = sstable.get(key)
            if value is not None:
                return value
        
        return None
```

### Sharding and Consistent Hashing

```python
import hashlib
from bisect import bisect_left

class ConsistentHash:
    """
    Consistent hashing: minimal key movement when nodes change.
    Used in distributed caches, databases, load balancers.
    """
    
    def __init__(self, nodes, virtual_nodes=100):
        self.virtual_nodes = virtual_nodes
        self.ring = []
        self.node_map = {}
        
        for node in nodes:
            self.add_node(node)
    
    def _hash(self, key):
        return int(hashlib.md5(key.encode()).hexdigest(), 16)
    
    def add_node(self, node):
        """Add node with virtual nodes for better distribution"""
        for i in range(self.virtual_nodes):
            virtual_key = f"{node}:{i}"
            hash_val = self._hash(virtual_key)
            self.ring.append(hash_val)
            self.node_map[hash_val] = node
        
        self.ring.sort()
    
    def remove_node(self, node):
        """Remove node and its virtual nodes"""
        for i in range(self.virtual_nodes):
            virtual_key = f"{node}:{i}"
            hash_val = self._hash(virtual_key)
            self.ring.remove(hash_val)
            del self.node_map[hash_val]
    
    def get_node(self, key):
        """Get the node responsible for this key"""
        if not self.ring:
            return None
        
        hash_val = self._hash(key)
        idx = bisect_left(self.ring, hash_val)
        
        if idx == len(self.ring):
            idx = 0
        
        return self.node_map[self.ring[idx]]
```

### Instrumentation and Monitoring

```python
import time
from collections import defaultdict
import statistics

class LatencyTracker:
    """Track latency percentiles - essential for tail latency"""
    
    def __init__(self, window_size=1000):
        self.window_size = window_size
        self.samples = []
    
    def record(self, latency_ms):
        self.samples.append(latency_ms)
        if len(self.samples) > self.window_size:
            self.samples.pop(0)
    
    def percentile(self, p):
        if not self.samples:
            return 0
        sorted_samples = sorted(self.samples)
        idx = int(len(sorted_samples) * p / 100)
        return sorted_samples[min(idx, len(sorted_samples) - 1)]
    
    def stats(self):
        return {
            'p50': self.percentile(50),
            'p90': self.percentile(90),
            'p99': self.percentile(99),
            'p999': self.percentile(99.9),
            'mean': statistics.mean(self.samples) if self.samples else 0,
        }


# Usage: wrap all external calls
tracker = LatencyTracker()

def tracked_operation():
    start = time.time()
    try:
        result = do_actual_work()
        return result
    finally:
        latency_ms = (time.time() - start) * 1000
        tracker.record(latency_ms)
```

## Mental Model

Dean approaches system design by asking:

1. **What are the numbers?** Latency, throughput, storage at each layer
2. **What's the common case?** Optimize for it
3. **What's the 99th percentile?** Tail latency kills user experience
4. **How does this scale?** Design for 10x
5. **How does this fail?** Plan for partial failure

## Signature Dean Moves

- Know the numbers (memory, disk, network latencies)
- MapReduce for parallel processing
- Hedged/backup requests for tail latency
- LSM trees for write-heavy workloads
- Consistent hashing for sharding
- Extensive instrumentation from day one
