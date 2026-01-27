---
name: cloudflare-performance-engineering
description: Engineer high-performance network systems in the style of Cloudflare's performance team. Emphasizes kernel bypass (XDP/eBPF), edge computing with V8 isolates, Rust for systems programming, smart routing, and measuring everything. Use when building globally distributed systems, DDoS mitigation, CDN infrastructure, or any system where every millisecond matters at massive scale.
---

# Cloudflare Performance Engineering Style Guide

## Overview

Cloudflare operates one of the world's largest networks, handling over 35 million HTTP requests per second across 330+ cities. Their performance engineering philosophy combines deep kernel expertise, innovative use of eBPF/XDP, Rust-based systems programming, and relentless measurement. Key figures include **John Graham-Cumming** (former CTO), **Marek Majkowski** (kernel/networking expert), and the teams behind Pingora, Workers, and quiche.

## Core Philosophy

> "If you can't measure it, you can't improve it—and you're probably making it worse."
>
> "Move the code to the data, not the data to the code."
>
> "The fastest packet is the one you never have to process."
>
> "Every millisecond matters when you multiply it by a trillion requests."

Cloudflare's approach: push computation as close to the user as possible (edge), eliminate unnecessary work at every layer (kernel bypass), use memory-safe systems languages (Rust), and measure everything in production with real user data.

## Key Visionaries

### John Graham-Cumming (Former CTO)

- Emphasis on security *and* performance as complementary, not competing
- "Help build a better Internet" mission driving architectural decisions
- Deep technical roots in formal methods and computer security

### Marek Majkowski

- Pioneer of XDP/eBPF adoption at scale for packet processing
- Author of foundational posts: "How to drop 10 million packets per second"
- Kernel bypass expertise, pushing Linux networking to its limits

### The Pingora Team

- Replaced NGINX with custom Rust proxy handling 35M+ req/sec
- 70% less CPU, 67% less memory vs. previous Lua/NGINX stack
- Demonstrates commitment to owning the entire stack

## Design Principles

1. **Edge-First Architecture**: Compute at the network edge, not in centralized data centers.

2. **Kernel Bypass When It Matters**: Use XDP/eBPF to process packets before they hit the kernel stack.

3. **Memory Safety at Scale**: Rust for new systems code—eliminate entire classes of vulnerabilities.

4. **Measure with Real Users**: RUM (Real User Measurement) over synthetic benchmarks.

5. **Smart Routing Over Dumb Pipes**: Use network intelligence to route around problems.

6. **Isolate, Don't Containerize**: V8 isolates for sub-millisecond cold starts.

## Performance Numbers to Know

```text
Cloudflare Network Scale:
──────────────────────────────────────────────────────────
Network locations                            330+ cities
Peak requests per second                     35,000,000+
Percentage of Internet traffic               ~20%
Average distance to any Internet user        <50ms

Packet Processing (XDP/eBPF):
──────────────────────────────────────────────────────────
iptables DROP                                ~2M pps/core
XDP DROP (kernel)                            ~10M pps/core
XDP DROP (native driver)                     ~26M pps/core
L4Drop (Cloudflare XDP)                      ~10M pps/core (with complex rules)

Workers (V8 Isolates):
──────────────────────────────────────────────────────────
Cold start time                              <1ms (vs 100ms+ containers)
Isolate memory overhead                      ~2MB (vs 35MB+ containers)
Time to global deployment                    <30 seconds

Pingora (Rust Proxy):
──────────────────────────────────────────────────────────
CPU reduction vs NGINX                       70%
Memory reduction vs NGINX                    67%
Connection reuse improvement                 Significant (multi-threaded)
```

## When Engineering for Performance

### Always

- Measure in production with real users (RUM), not just synthetic tests
- Know your p50, p95, p99, and p999 latencies—tail latency kills
- Process packets as early as possible in the stack (XDP > iptables > userspace)
- Use connection reuse aggressively—TCP handshakes are expensive
- Compress on the wire (CPU is cheaper than bandwidth)
- Cache at every layer: edge, tiered, origin shield
- Design for anycast—route users to nearest healthy PoP

### Never

- Trust synthetic benchmarks alone—production is different
- Process packets in userspace when kernel/XDP can do it
- Allocate memory in the hot path
- Ignore cold start latency for serverless workloads
- Route all traffic through origin—cache what you can
- Assume the network path is stable—routes change constantly
- Skip graceful degradation—partial service beats total failure

### Prefer

- XDP/eBPF over iptables for packet filtering
- Rust over C/C++ for new systems code
- V8 isolates over containers for edge compute
- Anycast over DNS-based load balancing
- Connection pooling over per-request connections
- Tiered caching over single-layer caches
- BBR over CUBIC for congestion control (especially on lossy networks)

## Architectural Patterns

### XDP Packet Processing Pipeline

```text
Packet arrives at NIC
         │
         ▼
┌─────────────────────┐
│    XDP Program      │  ← Runs in NIC driver, before sk_buff allocation
│  (eBPF bytecode)    │
└─────────────────────┘
         │
    ┌────┴────┬──────────┐
    ▼         ▼          ▼
XDP_DROP   XDP_TX    XDP_PASS
(discard)  (reflect)  (to kernel stack)
    │         │          │
    │         │          ▼
    │         │    Normal Linux
    │         │    networking
    │         │
    ▼         ▼
 ~26Mpps   Modified packet
 per core  sent back out

Key insight: No memory allocation for dropped packets = massive throughput
```

### Edge Computing with V8 Isolates

```text
Traditional Serverless:              Cloudflare Workers:
─────────────────────                ────────────────────
┌─────────────────┐                 ┌─────────────────────────┐
│   Container     │                 │     V8 Process          │
│ ┌─────────────┐ │                 │ ┌───┐ ┌───┐ ┌───┐ ┌───┐ │
│ │  Function   │ │                 │ │ I │ │ I │ │ I │ │ I │ │
│ │   Code      │ │                 │ │ s │ │ s │ │ s │ │ s │ │
│ └─────────────┘ │                 │ │ o │ │ o │ │ o │ │ o │ │
│  Node runtime   │                 │ │ 1 │ │ 2 │ │ 3 │ │ 4 │ │
│  OS overhead    │                 │ └───┘ └───┘ └───┘ └───┘ │
└─────────────────┘                 └─────────────────────────┘
    ~100ms cold start                    <1ms cold start
    ~35MB memory                         ~2MB per isolate

Key insight: Reuse V8 process, isolate tenants with isolates not VMs
```

### Smart Routing (Argo)

```text
Without Smart Routing:
─────────────────────
User → Nearest PoP → Public Internet (BGP) → Origin
       (fast)        (unpredictable)         

With Argo Smart Routing:
───────────────────────
User → Nearest PoP → Cloudflare Backbone → Exit PoP → Origin
       (fast)        (optimized, measured)   (closest)

Argo measures RTT, packet loss, and jitter across paths continuously.
Routes dynamically selected based on real-time conditions.
Typical improvement: 30% faster TTFB for dynamic content.
```

### Tiered Caching

```text
┌─────────────────────────────────────────────────────────────┐
│                         User Request                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Edge PoP (330+ locations)                                   │
│  Cache HIT? → Return immediately (fastest)                   │
└─────────────────────────────────────────────────────────────┘
                              │ MISS
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Upper-Tier PoP (Regional, ~20 locations)                    │
│  Cache HIT? → Return, populate edge cache                    │
│  Concentrates origin requests, improves hit ratio            │
└─────────────────────────────────────────────────────────────┘
                              │ MISS
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Origin Server                                               │
│  Single request even if 100 edge PoPs need the content      │
└─────────────────────────────────────────────────────────────┘

Key insight: Fewer origin requests = lower origin load + better cache hit ratio
```

## Code Patterns

### XDP Packet Filtering (eBPF/C)

```c
// SPDX-License-Identifier: GPL-2.0
#include <linux/bpf.h>
#include <linux/if_ether.h>
#include <linux/ip.h>
#include <linux/udp.h>
#include <bpf/bpf_helpers.h>

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 10000);
    __type(key, __u32);    // Source IP
    __type(value, __u64);  // Packet count
} blocked_ips SEC(".maps");

SEC("xdp")
int xdp_filter(struct xdp_md *ctx) {
    void *data_end = (void *)(long)ctx->data_end;
    void *data = (void *)(long)ctx->data;
    
    // Parse Ethernet header
    struct ethhdr *eth = data;
    if ((void *)(eth + 1) > data_end)
        return XDP_PASS;
    
    if (eth->h_proto != __constant_htons(ETH_P_IP))
        return XDP_PASS;
    
    // Parse IP header
    struct iphdr *ip = (void *)(eth + 1);
    if ((void *)(ip + 1) > data_end)
        return XDP_PASS;
    
    // Check blocklist - O(1) lookup in eBPF map
    __u32 src_ip = ip->saddr;
    __u64 *count = bpf_map_lookup_elem(&blocked_ips, &src_ip);
    if (count) {
        (*count)++;
        return XDP_DROP;  // Dropped at driver level, ~26Mpps
    }
    
    return XDP_PASS;
}

char LICENSE[] SEC("license") = "GPL";
```

### Rust Connection Pool (Pingora Style)

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use dashmap::DashMap;

/// Connection pool optimized for high-concurrency edge proxying.
/// 
/// Cloudflare's Pingora uses connection reuse aggressively to avoid
/// TCP handshake overhead. A single connection can serve many requests.
pub struct ConnectionPool<C> {
    pools: DashMap<String, Vec<C>>,
    max_idle_per_host: usize,
    semaphore: Arc<Semaphore>,
}

impl<C: Connection> ConnectionPool<C> {
    pub fn new(max_connections: usize, max_idle_per_host: usize) -> Self {
        Self {
            pools: DashMap::new(),
            max_idle_per_host,
            semaphore: Arc::new(Semaphore::new(max_connections)),
        }
    }
    
    /// Get a connection, reusing if possible.
    /// 
    /// Connection reuse is critical at Cloudflare scale:
    /// - Avoids TCP 3-way handshake (1 RTT saved)
    /// - Avoids TLS handshake (1-2 RTT saved)
    /// - Keeps TCP windows warm (better throughput)
    pub async fn get(&self, host: &str) -> Result<PooledConnection<C>, Error> {
        // Try to reuse existing connection
        if let Some(mut pool) = self.pools.get_mut(host) {
            if let Some(conn) = pool.pop() {
                if conn.is_healthy() {
                    return Ok(PooledConnection::new(conn, self, host.to_string()));
                }
                // Connection unhealthy, let it drop
            }
        }
        
        // Acquire permit for new connection
        let _permit = self.semaphore.acquire().await?;
        
        // Create new connection
        let conn = C::connect(host).await?;
        Ok(PooledConnection::new(conn, self, host.to_string()))
    }
    
    /// Return connection to pool for reuse.
    fn return_connection(&self, host: String, conn: C) {
        if !conn.is_healthy() {
            return; // Don't pool unhealthy connections
        }
        
        let mut pool = self.pools.entry(host).or_insert_with(Vec::new);
        if pool.len() < self.max_idle_per_host {
            pool.push(conn);
        }
        // If pool is full, connection is dropped
    }
}
```

### Workers-Style Edge Handler (JavaScript)

```javascript
/**
 * Cloudflare Workers run in V8 isolates at the edge.
 * 
 * Key performance principles:
 * - Sub-millisecond cold starts (isolates, not containers)
 * - Compute at the edge, close to users
 * - Stream responses, don't buffer
 * - Use the Cache API aggressively
 */
export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const cacheKey = new Request(url.toString(), request);
    const cache = caches.default;
    
    // Check edge cache first (fastest path)
    let response = await cache.match(cacheKey);
    if (response) {
      // Clone to add header without mutating cached response
      response = new Response(response.body, response);
      response.headers.set('X-Cache', 'HIT');
      return response;
    }
    
    // Cache miss - fetch from origin
    const originResponse = await fetch(request);
    
    // Only cache successful, cacheable responses
    if (originResponse.ok && isCacheable(originResponse)) {
      // Clone because response body can only be read once
      response = originResponse.clone();
      
      // Cache in background (don't block response)
      ctx.waitUntil(cache.put(cacheKey, response));
    }
    
    // Return immediately, caching happens async
    return originResponse;
  }
};

function isCacheable(response) {
  const cacheControl = response.headers.get('Cache-Control') || '';
  return !cacheControl.includes('no-store') && 
         !cacheControl.includes('private');
}
```

### Performance Measurement (RUM Style)

```javascript
/**
 * Real User Measurement (RUM) - Cloudflare's approach to performance data.
 * 
 * Key metrics:
 * - TCP Connection Time: Time to establish TCP connection
 * - TTFB (Time to First Byte): Connection + server processing
 * - TTLB (Time to Last Byte): Total transfer time
 * 
 * Always measure from real users, not synthetic tests.
 */
class PerformanceCollector {
  constructor(endpoint) {
    this.endpoint = endpoint;
    this.buffer = [];
    this.flushInterval = 5000;
    
    // Flush periodically in batches (amortize network overhead)
    setInterval(() => this.flush(), this.flushInterval);
  }
  
  measure(url) {
    const entry = performance.getEntriesByName(url)[0];
    if (!entry) return;
    
    const metrics = {
      url: url,
      timestamp: Date.now(),
      
      // DNS lookup (often cached, but important for cold loads)
      dnsLookup: entry.domainLookupEnd - entry.domainLookupStart,
      
      // TCP connection (XDP/kernel optimization target)
      tcpConnect: entry.connectEnd - entry.connectStart,
      
      // TLS handshake (QUIC eliminates separate TLS RTT)
      tlsHandshake: entry.secureConnectionStart > 0 
        ? entry.connectEnd - entry.secureConnectionStart 
        : 0,
      
      // Time to First Byte (server processing + network)
      ttfb: entry.responseStart - entry.requestStart,
      
      // Content transfer (CDN/edge cache optimization target)
      contentTransfer: entry.responseEnd - entry.responseStart,
      
      // Total time
      total: entry.responseEnd - entry.startTime,
      
      // Protocol (HTTP/2 vs HTTP/3)
      protocol: entry.nextHopProtocol,
      
      // Was this served from cache?
      cached: entry.transferSize === 0,
    };
    
    // Track percentiles, not just averages
    this.buffer.push(metrics);
  }
  
  async flush() {
    if (this.buffer.length === 0) return;
    
    const batch = this.buffer.splice(0, this.buffer.length);
    
    // Use sendBeacon for reliability (survives page unload)
    navigator.sendBeacon(this.endpoint, JSON.stringify(batch));
  }
}
```

## Mental Model

Cloudflare engineers approach performance with:

1. **Where can we avoid work?** The fastest code is code that doesn't run.
2. **How close to the user?** Push computation to the edge.
3. **How early in the stack?** XDP > kernel > userspace.
4. **What do real users see?** RUM over synthetic benchmarks.
5. **What's the tail latency?** p99 matters more than average.

### The Cloudflare Performance Review

```text
1. Where is latency added?
   - Network path (measure RTT, packet loss)
   - Protocol overhead (TLS, TCP handshakes)
   - Processing time (edge vs origin)
   - Queueing (congestion, bufferbloat)

2. What can be eliminated?
   - Unnecessary round trips (connection reuse, 0-RTT)
   - Redundant computation (caching at every layer)
   - Wasteful packet processing (XDP for early filtering)
   
3. What can be moved closer?
   - Compute to edge (Workers)
   - Cache to edge (Tiered Cache)
   - TLS termination to edge (reduces RTT)

4. How do we measure improvement?
   - Real User Measurements (RUM)
   - A/B testing with statistical significance
   - Percentile analysis (p50, p95, p99, p999)
```

## Warning Signs

You're violating Cloudflare's principles if:

- You're measuring performance with synthetic benchmarks only
- You're processing packets in userspace that could be filtered in XDP
- You're allocating memory in the hot path
- You're ignoring tail latency (p99, p999)
- You're running compute in centralized data centers when edge is possible
- You're using containers where isolates would work
- You're not measuring the impact of every change in production
- You're optimizing for average latency instead of percentiles

## Technology Stack

| Layer | Cloudflare Choice | Why |
|-------|------------------|-----|
| Packet filtering | XDP/eBPF | 10-26Mpps, kernel bypass |
| Proxy | Pingora (Rust) | Memory safety, 70% less CPU |
| Edge compute | V8 Isolates | <1ms cold start |
| QUIC/HTTP3 | quiche (Rust) | 0-RTT, better mobile |
| Congestion control | BBR | Better on lossy networks |
| Routing | Anycast + Argo | Automatic failover, smart paths |

## Additional Resources

- Cloudflare Blog: blog.cloudflare.com (technical deep-dives)
- Marek Majkowski's posts on XDP and kernel networking
- Pingora announcement and architecture posts
- Speed Week performance measurement methodology
