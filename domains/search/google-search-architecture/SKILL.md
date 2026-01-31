---
name: google-search-architecture
description: Build search systems informed by Google's internal architecture as revealed in the May 2024 API Content Warehouse leak. Covers Ascorer ranking, NavBoost click signals, Twiddler re-ranking, index tiering, and entity-based retrieval. Use when designing large-scale search engines, ranking systems, or understanding how modern search actually works.
---

# Google Search Architecture Style Guide

## Overview

In May 2024, Google accidentally published internal documentation for their "API Content Warehouse" on GitHub — 2,500+ pages describing 14,014 ranking attributes. This leak provides the most detailed public view into how Google Search actually works, contradicting years of public statements and revealing the true complexity of modern search.

This skill encodes the architectural patterns, ranking systems, and design principles inferred from this documentation.

## Core Philosophy

> "Search is not one algorithm. It's a pipeline of microservices where features are preprocessed and composed at runtime."

> "Clicks are the ground truth. User behavior is the ultimate signal."

> "Trust is earned over time. New sites start in a sandbox until they prove themselves."

Google's search is not a single ranking function with weighted factors. It's a **multi-stage pipeline** where documents pass through retrieval, scoring, re-ranking (Twiddlers), and final composition.

## The Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         QUERY PROCESSING                            │
│  Query Understanding → Intent Classification → Entity Recognition   │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         INDEX TIERS                                 │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────┐              │
│  │   Base   │    │  Zeppelins   │    │  Landfills   │              │
│  │ (Fresh)  │    │  (Standard)  │    │   (Archive)  │              │
│  └──────────┘    └──────────────┘    └──────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         ASCORER (Primary Ranking)                   │
│  PageRank + Content Signals + Entity Scores + Link Quality          │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         NAVBOOST (Click Signals)                    │
│  13 months of click data → CRAPS processing → Engagement signals    │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         TWIDDLERS (Re-ranking)                      │
│  FreshnessTwiddler │ QualityBoost │ RealTimeBoost │ NavBoost        │
│  (Adjust top 20-30 results based on signals)                        │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         SERP COMPOSITION                            │
│  Snippet generation, Featured snippets, Knowledge panels, Ads       │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Systems

### 1. Ascorer — The Primary Ranking Algorithm

The main scoring function that evaluates document relevance. Incorporates:

- **PageRank variants** (rawPagerank, pagerank2) — Link authority
- **Content quality signals** — Originality, depth, E-E-A-T
- **Entity scores** — How well the document covers relevant entities
- **Site authority** (siteAuthority) — Domain-level trust

```python
# Conceptual Ascorer scoring
class Ascorer:
    def score(self, query, document):
        """
        Primary ranking score combining multiple signal families.
        """
        return (
            self.relevance_score(query, document) *
            self.quality_score(document) *
            self.authority_score(document) *
            self.freshness_modifier(query, document)
        )
    
    def authority_score(self, document):
        """
        Combines page-level and site-level authority.
        """
        page_authority = self.get_pagerank(document.url)
        site_authority = self.get_site_authority(document.domain)
        
        # Site authority acts as a ceiling/floor
        return blend(page_authority, site_authority)
```

### 2. NavBoost — Click Signals Are Real

Google denied using clicks for years. The leak confirms **NavBoost** — a system that:

- Collects **13 months of click data**
- Tracks click-through rates, dwell time, bounces
- Feeds into **CRAPS** (Click and Results Prediction System)
- Directly influences rankings

```python
class NavBoost:
    """
    User behavior signals from search interactions.
    """
    
    def __init__(self):
        self.click_window = timedelta(days=395)  # ~13 months
    
    def compute_signals(self, url, query_class):
        """
        Aggregate click signals for a URL within a query class.
        """
        clicks = self.get_clicks(url, query_class, self.click_window)
        
        return {
            'click_through_rate': self.ctr(clicks),
            'long_clicks': self.count_long_clicks(clicks),  # Dwell > 30s
            'short_clicks': self.count_short_clicks(clicks),  # Bounce < 10s
            'last_longest_clicks': self.recent_engagement(clicks),
            'squashed_clicks': self.diminishing_returns(clicks),
        }
    
    def count_long_clicks(self, clicks):
        """
        Long clicks (high dwell time) are strong positive signals.
        Short clicks (quick bounces) are negative signals.
        """
        return sum(1 for c in clicks if c.dwell_time > 30)
```

### 3. Twiddlers — Post-Retrieval Re-ranking

**Twiddlers** are re-ranking functions applied after initial scoring. They adjust the top 20-30 results based on specific signals:

| Twiddler | Function |
|----------|----------|
| **NavBoost** | Boost based on click engagement |
| **QualityBoost** | Boost based on quality signals |
| **FreshnessTwiddler** | Boost newer content for time-sensitive queries |
| **RealTimeBoost** | Boost breaking news/trending content |
| **DemotionTwiddler** | Demote low-quality or policy-violating content |

```python
class TwiddlerPipeline:
    """
    Twiddlers run after Ascorer, adjusting the final ranking.
    They operate on the top N results (typically 20-30).
    """
    
    def __init__(self):
        self.twiddlers = [
            NavBoostTwiddler(),
            QualityBoostTwiddler(),
            FreshnessTwiddler(),
            RealTimeBoostTwiddler(),
            DemotionTwiddler(),
        ]
    
    def apply(self, query, results):
        """
        Apply twiddlers sequentially to re-rank results.
        """
        for twiddler in self.twiddlers:
            if twiddler.should_apply(query):
                results = twiddler.rerank(query, results)
        return results


class FreshnessTwiddler:
    """
    Boost fresh content for queries with freshness intent.
    """
    
    def should_apply(self, query):
        return query.has_freshness_intent()
    
    def rerank(self, query, results):
        for result in results:
            age = now() - result.publish_date
            if age < timedelta(hours=24):
                result.score *= 1.3  # Strong boost for very fresh
            elif age < timedelta(days=7):
                result.score *= 1.1  # Moderate boost for recent
        return sorted(results, key=lambda r: r.score, reverse=True)
```

### 4. Index Tiers — Not All Pages Are Equal

Google maintains multiple index tiers with different freshness and quality thresholds:

| Tier | Description | Crawl Frequency |
|------|-------------|-----------------|
| **Base** | High-quality, fresh content | Frequent |
| **Zeppelins** | Standard content | Moderate |
| **Landfills** | Low-quality, archive content | Rare |

```python
class IndexTierAssignment:
    """
    Assign documents to index tiers based on quality signals.
    """
    
    def assign_tier(self, document):
        quality = self.compute_quality(document)
        freshness_need = self.freshness_importance(document)
        
        if quality > 0.8 and freshness_need > 0.7:
            return 'base'  # Premium tier, frequent updates
        elif quality > 0.4:
            return 'zeppelins'  # Standard tier
        else:
            return 'landfills'  # Archive tier, rarely re-crawled
```

### 5. Site Authority — Domain Trust Is Real

Despite public denials, Google uses **siteAuthority** — a domain-level trust metric:

```python
class SiteAuthority:
    """
    Domain-level authority score used in the Q* system.
    """
    
    def compute(self, domain):
        return {
            'topicality_concentration': self.topic_focus(domain),
            'link_authority': self.aggregate_pagerank(domain),
            'brand_signals': self.brand_recognition(domain),
            'historical_quality': self.quality_history(domain),
            'chrome_data': self.chrome_engagement(domain),
        }
    
    def topic_focus(self, domain):
        """
        Higher concentration on specific topics = higher authority
        in those topics. Generalist sites have diluted authority.
        """
        topics = self.get_topic_distribution(domain)
        return self.concentration_score(topics)
```

### 6. The Sandbox — New Sites Are Held Back

The leak confirms new sites face a **sandbox period**:

```python
class SandboxEvaluator:
    """
    New sites are held back until they establish trust.
    """
    
    def __init__(self):
        self.min_age = timedelta(days=180)  # ~6 months minimum
    
    def apply_sandbox(self, document, site_info):
        site_age = now() - site_info.first_indexed
        
        if site_age < self.min_age:
            # Apply dampening factor to new sites
            dampening = site_age / self.min_age  # 0.0 to 1.0
            document.score *= dampening
        
        return document
```

### 7. Chrome Data Integration

Google uses Chrome browser data for ranking signals:

```python
class ChromeSignals:
    """
    Signals derived from Chrome browser usage.
    """
    
    def get_signals(self, url):
        return {
            'site_engagement': self.engagement_score(url.domain),
            'direct_navigation': self.direct_visit_rate(url.domain),
            'bookmarks': self.bookmark_frequency(url),
            'time_on_site': self.average_session_duration(url.domain),
        }
```

## Design Principles

### 1. Multi-Stage Pipeline

Don't build a single ranking function. Build a **pipeline**:

```
Retrieval → Scoring → Re-ranking → Composition
```

Each stage has different latency budgets and can use different signal types.

### 2. Clicks Are Ground Truth

User behavior is the ultimate signal. Build systems to:
- Collect click data at scale
- Distinguish long clicks (engagement) from short clicks (bounces)
- Use click patterns to validate other signals

### 3. Trust Is Earned Over Time

New content/sites should face a probation period:
- Historical quality matters
- Consistency builds trust
- Sudden changes trigger re-evaluation

### 4. Entities, Not Just Keywords

Modern search is entity-centric:
- Recognize entities in queries and documents
- Score entity coverage and relevance
- Use knowledge graphs to understand relationships

### 5. Quality Has Multiple Dimensions

Quality signals include:
- **Content quality** — Originality, depth, accuracy
- **Site quality** — Authority, trust, history
- **Page quality** — UX, speed, mobile-friendliness
- **Author quality** — E-E-A-T (Experience, Expertise, Authoritativeness, Trustworthiness)

## Code Patterns

### Building a Twiddler System

```python
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import List

@dataclass
class SearchResult:
    url: str
    score: float
    metadata: dict

class Twiddler(ABC):
    """
    Base class for re-ranking functions.
    """
    
    @abstractmethod
    def should_apply(self, query: str, context: dict) -> bool:
        """Determine if this twiddler should run for this query."""
        pass
    
    @abstractmethod
    def rerank(self, query: str, results: List[SearchResult]) -> List[SearchResult]:
        """Re-rank the results."""
        pass


class TwiddlerPipeline:
    def __init__(self, twiddlers: List[Twiddler]):
        self.twiddlers = twiddlers
    
    def apply(self, query: str, results: List[SearchResult], context: dict) -> List[SearchResult]:
        for twiddler in self.twiddlers:
            if twiddler.should_apply(query, context):
                results = twiddler.rerank(query, results)
        return results


# Example: Freshness Twiddler
class FreshnessTwiddler(Twiddler):
    def __init__(self, freshness_queries: set):
        self.freshness_queries = freshness_queries
    
    def should_apply(self, query: str, context: dict) -> bool:
        # Apply for news, events, or queries with time indicators
        return (
            context.get('query_type') == 'news' or
            any(term in query.lower() for term in ['today', 'latest', 'new', '2024'])
        )
    
    def rerank(self, query: str, results: List[SearchResult]) -> List[SearchResult]:
        from datetime import datetime, timedelta
        now = datetime.utcnow()
        
        for result in results:
            pub_date = result.metadata.get('publish_date')
            if pub_date:
                age = now - pub_date
                if age < timedelta(hours=24):
                    result.score *= 1.5
                elif age < timedelta(days=7):
                    result.score *= 1.2
        
        return sorted(results, key=lambda r: r.score, reverse=True)
```

### Building a Click Signal System

```python
from collections import defaultdict
from datetime import datetime, timedelta

class ClickSignalCollector:
    """
    Collect and aggregate click signals (NavBoost-style).
    """
    
    def __init__(self, window_days: int = 395):
        self.window = timedelta(days=window_days)
        self.clicks = defaultdict(list)  # url -> [Click]
    
    def record_click(self, query: str, url: str, dwell_time: float, position: int):
        """Record a click event."""
        self.clicks[url].append({
            'query': query,
            'timestamp': datetime.utcnow(),
            'dwell_time': dwell_time,
            'position': position,
        })
    
    def get_signals(self, url: str) -> dict:
        """Compute aggregate signals for a URL."""
        cutoff = datetime.utcnow() - self.window
        recent_clicks = [c for c in self.clicks[url] if c['timestamp'] > cutoff]
        
        if not recent_clicks:
            return {'has_data': False}
        
        long_clicks = sum(1 for c in recent_clicks if c['dwell_time'] > 30)
        short_clicks = sum(1 for c in recent_clicks if c['dwell_time'] < 10)
        
        return {
            'has_data': True,
            'total_clicks': len(recent_clicks),
            'long_click_rate': long_clicks / len(recent_clicks),
            'short_click_rate': short_clicks / len(recent_clicks),
            'avg_dwell_time': sum(c['dwell_time'] for c in recent_clicks) / len(recent_clicks),
            'position_weighted_ctr': self._position_weighted_ctr(recent_clicks),
        }
    
    def _position_weighted_ctr(self, clicks):
        """
        Weight clicks by position — clicking result #10 is more
        meaningful than clicking result #1.
        """
        weights = {1: 1.0, 2: 1.2, 3: 1.4, 4: 1.6, 5: 1.8}
        weighted_sum = sum(weights.get(c['position'], 2.0) for c in clicks)
        return weighted_sum / len(clicks) if clicks else 0
```

### Index Tier Management

```python
class IndexTierManager:
    """
    Manage document assignment to index tiers.
    """
    
    TIERS = ['base', 'zeppelins', 'landfills']
    
    def __init__(self):
        self.tier_thresholds = {
            'base': {'quality': 0.8, 'freshness_need': 0.7},
            'zeppelins': {'quality': 0.4, 'freshness_need': 0.3},
            'landfills': {'quality': 0.0, 'freshness_need': 0.0},
        }
        self.crawl_intervals = {
            'base': timedelta(hours=1),
            'zeppelins': timedelta(days=7),
            'landfills': timedelta(days=30),
        }
    
    def assign_tier(self, document) -> str:
        quality = self.compute_quality(document)
        freshness = self.compute_freshness_need(document)
        
        for tier in self.TIERS:
            thresholds = self.tier_thresholds[tier]
            if quality >= thresholds['quality']:
                return tier
        
        return 'landfills'
    
    def get_crawl_interval(self, tier: str) -> timedelta:
        return self.crawl_intervals.get(tier, timedelta(days=30))
```

## Mental Model

Think of Google Search as a **trust and relevance machine**:

1. **Retrieval**: Find candidate documents (index tiers matter)
2. **Scoring**: Ascorer computes base relevance + authority
3. **Behavioral validation**: NavBoost adjusts based on user engagement
4. **Re-ranking**: Twiddlers apply query-specific adjustments
5. **Composition**: Build the final SERP with snippets, features, etc.

**Trust flows through the system:**
- Sites earn trust over time (sandbox → established)
- Pages inherit site trust but can exceed or fall below it
- Links from trusted sources transfer more trust
- User behavior (clicks, engagement) validates or contradicts other signals

## Common Mistakes

### Ignoring the Pipeline

```python
# BAD: Single monolithic scoring function
def rank(query, documents):
    return sorted(documents, key=lambda d: compute_score(query, d))

# GOOD: Multi-stage pipeline with different concerns
def rank(query, documents):
    candidates = retrieve(query, documents)      # Fast, recall-focused
    scored = score(query, candidates)            # Relevance + authority
    reranked = apply_twiddlers(query, scored)    # Query-specific adjustments
    return compose_serp(query, reranked)         # Final presentation
```

### Ignoring Click Signals

```python
# BAD: Ranking purely on content signals
score = content_relevance * pagerank

# GOOD: Incorporate behavioral signals
score = content_relevance * pagerank * navboost_modifier(click_signals)
```

### Treating All Content Equally

```python
# BAD: Same crawl frequency for everything
crawl_all_pages(interval=timedelta(days=1))

# GOOD: Tier-based crawling
for page in pages:
    tier = assign_tier(page)
    schedule_crawl(page, interval=tier_intervals[tier])
```

## Debugging Questions

When your search system isn't working:

1. **What stage is failing?** Retrieval? Scoring? Re-ranking?
2. **Do I have behavioral signals?** Click data is the ground truth.
3. **How old is this content/site?** New sites face natural dampening.
4. **What tier is this content in?** Lower tiers get less attention.
5. **Are twiddlers helping or hurting?** Check each re-ranker's impact.

## References

- [Google API Content Warehouse Leak Analysis (iPullRank)](https://ipullrank.com/google-algo-leak)
- [Search Engine Land: Document Leak Reveals Ranking Algorithm](https://searchengineland.com/google-search-document-leak-ranking-442617)
- [Twiddler Framework Deep Dive (Resoneo)](https://www.resoneo.com/google-leak-part-2-understanding-the-twiddler-framework/)
- [NavBoost and CRAPS Analysis (Hobo)](https://www.hobo-web.co.uk/evidence-based-mapping-of-google-updates-to-leaked-internal-ranking-signals/)
- [Google Search Quality Rater Guidelines](https://guidelines.raterhub.com/) — Official human evaluation criteria
