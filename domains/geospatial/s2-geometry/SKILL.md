---
name: s2-geometry-spatial-indexing
description: Index and query geospatial data using Google's S2 Geometry library. Emphasizes hierarchical cell decomposition, Hilbert curves for locality preservation, and efficient spatial operations on spherical geometry. Use when building location-based services, proximity search, geofencing, or any system that needs to efficiently query geographic data.
---

# S2 Geometry Style Guide

## Overview

S2 is Google's library for spherical geometry and spatial indexing, used internally for Google Maps, Google Earth, and countless geo-aware services. It solves the fundamental problem: **how do you efficiently index and query locations on a sphere?**

The library was developed at Google and open-sourced in 2017. It powers proximity search, geofencing, spatial joins, and geographic sharding at planetary scale.

## Core Philosophy

> "The Earth is not flat. Your spatial index shouldn't pretend it is."

> "A good spatial index turns geometric queries into range queries on integers."

> "Locality in space should mean locality in your index."

S2's insight: project the sphere onto a cube, fill each face with a space-filling Hilbert curve, and encode positions as 64-bit integers. Nearby points get nearby integers. Geometric queries become range scans.

## Design Principles

1. **Spherical First**: No projection distortion. Work on the actual sphere.

2. **Hierarchical Decomposition**: 30 levels from ~85km² cells down to ~1cm² cells.

3. **Locality Preservation**: Hilbert curves ensure nearby points have nearby cell IDs.

4. **Integer Encoding**: Any cell is a single 64-bit integer. Hierarchy is in the bits.

5. **Exact Predicates**: Robust geometric operations that don't fail on edge cases.

## When Writing Code

### Always

- Think in cells, not coordinates
- Choose the right cell level for your precision needs
- Use region coverings for irregular shapes
- Leverage the hierarchical nature for multi-resolution queries
- Remember: cell ID ordering preserves spatial locality

### Never

- Use lat/lng bounding boxes for spherical queries (they distort near poles)
- Ignore the antimeridian (longitude ±180°)
- Assume cells are square (they're quadrilaterals on a sphere)
- Store raw coordinates when you could store cell IDs
- Forget that level 30 is ~1cm², level 12 is ~3km²

### Prefer

- Cell IDs over lat/lng pairs for storage and indexing
- Region coverings over point-in-polygon for containment
- S2 distance calculations over Haversine (more accurate)
- Hierarchical queries (coarse to fine) for performance
- Cell unions for representing complex regions

## The S2 Cell Hierarchy

```
Level    Cell Size (edge)    Use Case
─────────────────────────────────────────────────────
  0      ~7,842 km          Continental regions
  4      ~490 km            Country-scale
  8      ~31 km             Metro area
 12      ~1.9 km            Neighborhood
 14      ~477 m             City block
 16      ~119 m             Building cluster  
 18      ~30 m              Building footprint
 20      ~7.4 m             Room-scale
 24      ~0.46 m            Sub-meter precision
 30      ~0.7 cm            Maximum precision
```

## Code Patterns

### Basic Cell Operations

```python
import s2sphere  # Python wrapper

# Point to cell at specific level
lat, lng = 37.7749, -122.4194  # San Francisco
point = s2sphere.LatLng.from_degrees(lat, lng)
cell_id = s2sphere.CellId.from_lat_lng(point)

# Get cell at level 16 (~119m cells)
cell_level_16 = cell_id.parent(16)
print(f"Cell ID: {cell_level_16.id()}")  # 64-bit integer

# Cell properties
cell = s2sphere.Cell(cell_level_16)
print(f"Level: {cell.level()}")
print(f"Area: {cell.exact_area()} steradians")

# Get neighbors
neighbors = [cell_level_16.get_edge_neighbors()]

# Parent/child traversal
parent = cell_level_16.parent()  # Level 15
children = [cell_level_16.child(i) for i in range(4)]  # 4 children
```

### The 64-bit Cell ID Encoding

```python
# The cell ID encodes the entire hierarchy in its bits
# 
# Bit layout (simplified):
#   - 3 bits: face (0-5, which cube face)
#   - 2 bits per level: quadrant within parent (0-3)
#   - 1 bit: sentinel marking the end
#
# This means:
#   - Parent cell ID is a prefix of child cell ID
#   - Range queries on cell IDs = spatial range queries
#   - Sorting by cell ID clusters nearby cells together

cell_id = s2sphere.CellId.from_lat_lng(
    s2sphere.LatLng.from_degrees(37.7749, -122.4194)
)

# The magic: all descendants share a prefix
parent = cell_id.parent(12)
range_min = parent.range_min()  # Smallest descendant
range_max = parent.range_max()  # Largest descendant

# "Find all points in this region" becomes:
# SELECT * FROM locations 
# WHERE cell_id >= range_min AND cell_id <= range_max
```

### Region Covering Algorithm

```python
# The killer feature: approximate any region with a set of cells
# at varying levels, optimizing for minimal cells

from s2sphere import RegionCoverer, LatLngRect, LatLng

# Define a region (e.g., bounding rectangle)
rect = LatLngRect(
    LatLng.from_degrees(37.7, -122.5),  # SW corner
    LatLng.from_degrees(37.8, -122.4)   # NE corner
)

# Configure the coverer
coverer = RegionCoverer()
coverer.min_level = 8   # Don't go coarser than level 8
coverer.max_level = 16  # Don't go finer than level 16
coverer.max_cells = 20  # Use at most 20 cells

# Get the covering
covering = coverer.get_covering(rect)

# Result: a set of cells at different levels that
# tightly approximate the region
for cell_id in covering:
    print(f"Level {cell_id.level()}: {cell_id.id()}")

# These cell IDs can be used for:
# - Geofencing (is point in any of these cells?)
# - Spatial joins (do cell sets overlap?)
# - Sharding (route requests to shard owning the cell)
```

### Proximity Search Pattern

```python
# "Find all restaurants within 500m of me"

def find_nearby(center_lat, center_lng, radius_meters, max_results=100):
    """
    S2 approach to proximity search:
    1. Create a cap (spherical circle) around the point
    2. Get a covering of that cap
    3. Query the index for all covered cells
    4. Post-filter by exact distance
    """
    from s2sphere import Cap, LatLng, CellId, RegionCoverer
    import math
    
    # Earth radius in meters
    EARTH_RADIUS = 6371000
    
    # Create center point
    center = LatLng.from_degrees(center_lat, center_lng)
    
    # Create a spherical cap (circle on sphere)
    # Cap is defined by axis (center) and chord angle
    angle = radius_meters / EARTH_RADIUS  # radians
    cap = Cap.from_axis_angle(
        center.to_point(),
        s1.Angle.from_radians(angle)
    )
    
    # Get covering cells
    coverer = RegionCoverer()
    coverer.max_cells = 8  # Balance: fewer cells = more false positives
    covering = coverer.get_covering(cap)
    
    # Query database for each cell range
    candidates = []
    for cell_id in covering:
        # This is the key insight: spatial query → range query
        range_min = cell_id.range_min().id()
        range_max = cell_id.range_max().id()
        
        # SELECT * FROM places WHERE cell_id BETWEEN range_min AND range_max
        candidates.extend(db.query_range(range_min, range_max))
    
    # Post-filter by exact distance (covering may include some outside radius)
    results = []
    for place in candidates:
        dist = calculate_distance(center_lat, center_lng, place.lat, place.lng)
        if dist <= radius_meters:
            results.append((place, dist))
    
    # Sort by distance, return top N
    results.sort(key=lambda x: x[1])
    return results[:max_results]
```

### Geofencing with Cell Unions

```python
# "Is this user inside our delivery zone?"

class DeliveryZone:
    def __init__(self, polygon_coords, name):
        """
        Pre-compute a cell covering for the delivery zone.
        Containment checks become cell ID lookups.
        """
        self.name = name
        
        # Build S2 polygon from coordinates
        points = [LatLng.from_degrees(lat, lng) for lat, lng in polygon_coords]
        loop = S2Loop(points)
        polygon = S2Polygon(loop)
        
        # Cover the polygon with cells
        coverer = RegionCoverer()
        coverer.max_cells = 100  # More cells = tighter fit
        self.covering = coverer.get_covering(polygon)
        
        # Store as sorted list for binary search
        self.cell_ids = sorted([c.id() for c in self.covering])
    
    def contains(self, lat, lng):
        """
        Fast containment check:
        1. Get cell ID for point
        2. Check if any ancestor cell is in our covering
        """
        point_cell = CellId.from_lat_lng(LatLng.from_degrees(lat, lng))
        
        # Check this cell and all its ancestors
        cell = point_cell
        while cell.is_valid():
            # Binary search in our covering
            if self._cell_in_covering(cell.id()):
                return True
            cell = cell.parent()
        
        return False
    
    def _cell_in_covering(self, cell_id):
        # Binary search for cell_id in sorted covering
        import bisect
        idx = bisect.bisect_left(self.cell_ids, cell_id)
        return idx < len(self.cell_ids) and self.cell_ids[idx] == cell_id
```

### Geographic Sharding

```python
# Partition data across shards by geography

class GeoShardRouter:
    """
    Route requests to shards based on S2 cell ownership.
    Each shard owns a contiguous range of cell IDs.
    """
    
    def __init__(self, num_shards, replication_factor=3):
        self.num_shards = num_shards
        
        # Divide the full cell ID space among shards
        # Cell IDs range from 0 to 2^64-1
        # Hilbert curve ensures geographic locality within ranges
        self.shard_boundaries = []
        step = (2**64) // num_shards
        for i in range(num_shards):
            self.shard_boundaries.append(i * step)
    
    def get_shard(self, lat, lng, level=16):
        """Get the primary shard for a location."""
        cell_id = CellId.from_lat_lng(
            LatLng.from_degrees(lat, lng)
        ).parent(level).id()
        
        # Binary search for owning shard
        import bisect
        shard = bisect.bisect_right(self.shard_boundaries, cell_id) - 1
        return shard
    
    def get_shards_for_region(self, covering):
        """
        For a region query, return all shards that might have data.
        This is why coverings are powerful: bounded shard fan-out.
        """
        shards = set()
        for cell_id in covering:
            # Add shards for entire cell range
            min_shard = self.get_shard_by_id(cell_id.range_min().id())
            max_shard = self.get_shard_by_id(cell_id.range_max().id())
            for s in range(min_shard, max_shard + 1):
                shards.add(s)
        return shards
```

### Hilbert Curve Intuition

```
Why Hilbert curves? They preserve locality better than alternatives.

Z-order (Morton) curve:         Hilbert curve:
┌───┬───┐                       ┌───┬───┐
│ 0 │ 1 │                       │ 0 │ 1 │
├───┼───┤                       ├───┼───┤
│ 2 │ 3 │                       │ 3 │ 2 │  ← Notice: 2 and 3 are adjacent
└───┴───┘                       └───┴───┘

Z-order: cells 1 and 2 are sequential but not adjacent spatially.
Hilbert: sequential cells are always spatially adjacent.

This matters because:
- Range queries on cell IDs return spatially coherent results
- Cache locality is preserved
- Fewer disk seeks for spatial scans
```

## Mental Model

Think of S2 as giving every location on Earth a **sort key** where:

1. **Nearby locations have similar sort keys** (Hilbert property)
2. **Containment is prefix matching** (hierarchy property)
3. **Resolution is adjustable** (level selection)
4. **Regions are cell sets** (covering algorithm)

When you need to answer "what's near X?" or "what's inside Y?", you're really asking about ranges and sets of these sort keys.

## Common Mistakes

### Wrong Level Selection

```python
# BAD: Using level 30 (1cm) for city-scale queries
cell = CellId.from_lat_lng(point).parent(30)  # Way too precise

# GOOD: Match level to your use case
# Ride-sharing pickup: level 16-18 (~30-120m)
# Neighborhood search: level 12-14 (~500m-2km)
# City-scale analytics: level 8-10 (~10-40km)
cell = CellId.from_lat_lng(point).parent(16)
```

### Ignoring Covering Size

```python
# BAD: Unlimited cells = slow queries
coverer.max_cells = 10000  # Will fan out to too many ranges

# GOOD: Balance precision vs. query cost
coverer.max_cells = 8   # For proximity search
coverer.max_cells = 50  # For geofencing
coverer.max_cells = 200 # For precise region analytics
```

### Forgetting Post-Filtering

```python
# BAD: Assuming covering is exact
results = query_covering(covering)  # May include points outside region

# GOOD: Always post-filter for exactness
results = query_covering(covering)
results = [r for r in results if region.contains(r.point)]
```

## S2 Debugging Questions

When your spatial queries aren't working:

1. **What level am I using?** Is it appropriate for my precision needs?
2. **How many cells in my covering?** Too few = false positives. Too many = slow.
3. **Am I handling the antimeridian?** Regions crossing ±180° need special care.
4. **Am I post-filtering?** Coverings are approximate by design.
5. **Is my data indexed at the right level?** Query level should match index level.

## References

- [S2 Geometry Library](https://s2geometry.io/) — Official documentation
- [S2 Cells Overview](https://s2geometry.io/devguide/s2cell_hierarchy) — Cell hierarchy details
- [Google Open Source Blog: Announcing S2](https://opensource.googleblog.com/2017/12/announcing-s2-library-geometry-on-sphere.html)
- [Hilbert Curves and S2](https://blog.christianperone.com/2015/08/googles-s2-geometry-on-the-sphere-cells-and-hilbert-curve/)
- [S2 at Foursquare](https://engineering.foursquare.com/) — Production usage patterns
- [Uber H3](https://h3geo.org/) — Alternative (hexagonal) for comparison
