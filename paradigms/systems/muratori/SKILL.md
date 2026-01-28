---
name: muratori-performance-aware
description: Write performance-aware code in the style of Casey Muratori, creator of Handmade Hero and performance optimization educator. Emphasizes understanding what the hardware actually does, rejecting unnecessary abstraction, and measuring everything. Use when writing performance-critical code or when you need to understand what's really happening on the machine.
---

# Casey Muratori Style Guide

## Overview

Casey Muratori is a game developer, performance optimization expert, and educator who created Handmade Hero—a project to write a complete game from scratch with no libraries. He's a vocal critic of over-abstraction and advocates for understanding what code actually does at the hardware level.

## Core Philosophy

> "The computer is not an abstraction. It's a real machine doing real things."

> "Clean code is not the same as good code."

> "If you don't know what the code does, you don't know if it's fast."

Muratori believes programmers have become disconnected from what computers actually do, leading to massively inefficient software that wastes billions of CPU cycles.

## Design Principles

1. **Know Your Hardware**: Understand the actual machine, not abstractions.

2. **Measure Everything**: Never guess about performance.

3. **Question Dogma**: "Best practices" often aren't.

4. **Simple Over "Clean"**: Clarity about what happens beats elegant patterns.

## When Writing Code

### Always

- Know what assembly your code generates
- Profile before optimizing
- Question every abstraction's cost
- Understand memory layout and access patterns
- Write the simplest code that does the job
- Keep the hot path obvious and tight

### Never

- Use patterns because they're "clean" without measuring
- Trust that the compiler will optimize it
- Add abstraction layers without justification
- Hide what the code actually does
- Optimize without profiling first
- Assume standard library implementations are fast

### Prefer

- Arrays over linked structures
- Data-oriented design over object-oriented
- Explicit code over implicit conventions
- Simple loops over iterators
- Direct computation over indirection
- Platform-specific code when it matters

## Code Patterns

### Data-Oriented Design

```cpp
// Object-Oriented: scattered memory, cache misses
class Entity {
    Vector3 position;
    Vector3 velocity;
    Quaternion rotation;
    Mesh* mesh;
    Material* material;
    AI* ai;
    Physics* physics;
    // ... methods
};

std::vector<Entity*> entities;  // Pointer chasing nightmare

// Update all positions: cache miss per entity
for (Entity* e : entities) {
    e->position += e->velocity * dt;  // Cache miss, cache miss
}


// Data-Oriented: contiguous memory, cache friendly
struct Positions { float *x, *y, *z; };
struct Velocities { float *vx, *vy, *vz; };

// Update all positions: sequential memory access
void update_positions(Positions* pos, Velocities* vel, 
                      float dt, int count) {
    for (int i = 0; i < count; i++) {
        pos->x[i] += vel->vx[i] * dt;
        pos->y[i] += vel->vy[i] * dt;
        pos->z[i] += vel->vz[i] * dt;
    }
    // CPU prefetcher loves this
    // SIMD vectorization possible
}
```

### Rejecting Premature Abstraction

```cpp
// Over-abstracted "clean" code
class IRenderer {
    virtual void Render(IDrawable* drawable) = 0;
};

class OpenGLRenderer : public IRenderer {
    void Render(IDrawable* drawable) override {
        auto vertices = drawable->GetVertices();
        auto material = drawable->GetMaterial();
        // Virtual call, virtual call, virtual call...
    }
};

// What actually needs to happen
void render_meshes(Mesh* meshes, int count, RenderState* state) {
    // Sort by material to minimize state changes
    sort_by_material(meshes, count);
    
    Material* current_material = NULL;
    for (int i = 0; i < count; i++) {
        if (meshes[i].material != current_material) {
            current_material = meshes[i].material;
            bind_material(current_material);
        }
        draw_mesh(&meshes[i]);
    }
}

// The "clean" version has:
// - Virtual dispatch overhead
// - Memory scattered across heap
// - No ability to batch or sort
// - Hidden costs everywhere
```

### Understanding What Code Actually Does

```cpp
// "Simple" C++ string concatenation
std::string build_path(const std::string& dir, const std::string& file) {
    return dir + "/" + file;  // What does this do?
}

// What actually happens:
// 1. Allocate temporary for dir + "/"
// 2. Copy dir into temporary
// 3. Append "/" 
// 4. Allocate result string
// 5. Copy temporary into result
// 6. Append file
// 7. Destroy temporary
// Multiple allocations, copies, for one string!


// Direct version: you know exactly what happens
void build_path(char* out, size_t out_size,
                const char* dir, const char* file) {
    size_t dir_len = strlen(dir);
    size_t file_len = strlen(file);
    
    if (dir_len + 1 + file_len + 1 > out_size) {
        out[0] = 0;
        return;
    }
    
    memcpy(out, dir, dir_len);
    out[dir_len] = '/';
    memcpy(out + dir_len + 1, file, file_len + 1);
}

// One buffer, no allocations, obvious behavior
```

### SIMD When It Matters

```cpp
// Scalar version
void add_arrays_scalar(float* a, float* b, float* out, int count) {
    for (int i = 0; i < count; i++) {
        out[i] = a[i] + b[i];
    }
}

// SIMD version: 4x or 8x throughput
void add_arrays_simd(float* a, float* b, float* out, int count) {
    int simd_count = count & ~7;  // Round down to multiple of 8
    
    for (int i = 0; i < simd_count; i += 8) {
        __m256 va = _mm256_loadu_ps(a + i);
        __m256 vb = _mm256_loadu_ps(b + i);
        __m256 vr = _mm256_add_ps(va, vb);
        _mm256_storeu_ps(out + i, vr);
    }
    
    // Handle remainder
    for (int i = simd_count; i < count; i++) {
        out[i] = a[i] + b[i];
    }
}

// But measure! SIMD only wins for:
// - Large enough data (amortize setup)
// - Aligned access (or accept penalty)
// - Operations that vectorize well
```

### Profiling-Driven Development

```cpp
// Built-in profiling for hot code
struct ProfileBlock {
    const char* name;
    uint64_t start_tsc;
    uint64_t* accumulator;
    
    ProfileBlock(const char* n, uint64_t* acc) : name(n), accumulator(acc) {
        start_tsc = __rdtsc();
    }
    ~ProfileBlock() {
        *accumulator += __rdtsc() - start_tsc;
    }
};

#define PROFILE_BLOCK(name) \
    static uint64_t prof_##name = 0; \
    ProfileBlock _pb_##name(#name, &prof_##name)

void game_update() {
    {
        PROFILE_BLOCK(physics);
        update_physics();
    }
    {
        PROFILE_BLOCK(ai);
        update_ai();
    }
    {
        PROFILE_BLOCK(render);
        render_frame();
    }
}

// Know where time actually goes
// Not where you think it goes
```

### Memory Layout Awareness

```cpp
// Array of Structures (AoS) - typical OOP
struct Particle_AoS {
    float x, y, z;      // Position
    float vx, vy, vz;   // Velocity
    float r, g, b, a;   // Color
    float size;
    float life;
};
Particle_AoS particles_aos[10000];

// Updating positions touches: x, y, z, vx, vy, vz
// But cache line also loads: r, g, b, a, size, life
// 50% of loaded data is wasted!


// Structure of Arrays (SoA) - data-oriented
struct Particles_SoA {
    float x[10000], y[10000], z[10000];
    float vx[10000], vy[10000], vz[10000];
    float r[10000], g[10000], b[10000], a[10000];
    float size[10000];
    float life[10000];
};
Particles_SoA particles_soa;

// Updating positions touches: x, y, z, vx, vy, vz
// Cache lines contain only what we need
// SIMD can process 4/8 particles at once
```

### Reject Unnecessary Indirection

```cpp
// Java-brain C++: indirection everywhere
class GameObjectManager {
    std::unique_ptr<IAllocator> allocator;
    std::unordered_map<ObjectId, std::unique_ptr<GameObject>> objects;
    
    void Update() {
        for (auto& [id, obj] : objects) {
            obj->Update();  // Virtual call, pointer chase
        }
    }
};

// Direct version: know what happens
struct Game {
    Entity entities[MAX_ENTITIES];
    int entity_count;
    
    void update() {
        for (int i = 0; i < entity_count; i++) {
            entities[i].x += entities[i].vx * dt;
            entities[i].y += entities[i].vy * dt;
            // Inline, no virtual, predictable memory
        }
    }
};

// The "managed" version:
// - Hash table lookup per object
// - Unique_ptr dereference
// - Virtual dispatch
// - Scattered heap memory
// - 10-100x slower than direct
```

### Hot/Cold Splitting

```cpp
// All data together: cold data pollutes cache
struct Entity_Mixed {
    // Hot: accessed every frame
    float x, y, z;
    float vx, vy, vz;
    uint32_t flags;
    
    // Cold: accessed rarely
    char name[64];
    char description[256];
    time_t created_at;
    uint64_t unique_id;
};

// Split hot and cold
struct Entity_Hot {
    float x, y, z;
    float vx, vy, vz;
    uint32_t flags;
    uint32_t cold_index;  // Link to cold data
};

struct Entity_Cold {
    char name[64];
    char description[256];
    time_t created_at;
    uint64_t unique_id;
};

// Hot loop only touches hot data
// Cache lines aren't wasted on names and descriptions
```

## Performance Reality Check

```
Operation                          Cycles    Notes
═══════════════════════════════════════════════════════════
Register operation                 1         Free
L1 cache hit                       4         64 bytes
L2 cache hit                       12        
L3 cache hit                       40        
RAM access                         200+      The wall
Virtual function call              10-25     Depends on prediction
std::unordered_map lookup          50-200    Hash + chase
std::map lookup                    100-500   Tree traversal
new/malloc                         100-1000  Varies wildly
System call                        1000+     Context switch

Reality: Most "fast" code is limited by memory access
        Computation is essentially free by comparison
        Every pointer chase is a potential cache miss
```

## Mental Model

Muratori approaches code by asking:

1. **What does this actually do?** Not what it represents—what instructions run
2. **Where are the memory accesses?** That's where the time goes
3. **Can I make this simpler?** Simpler usually means faster
4. **Have I measured?** Intuition is often wrong
5. **What's the cost of abstraction?** Is it worth paying?

## Signature Muratori Moves

- **Handmade code**: Write from scratch, understand everything
- **Data-oriented design**: Lay out data for how it's accessed
- **Reject OOP dogma**: Objects aren't always the answer
- **Profile everything**: rdtsc is your friend
- **Hot/cold splitting**: Don't pollute cache with cold data
- **SIMD where it counts**: 4x-8x speedups when applicable
- **Questioning "clean" code**: Clean for whom?
- **Reading assembly**: Know what the compiler generates
