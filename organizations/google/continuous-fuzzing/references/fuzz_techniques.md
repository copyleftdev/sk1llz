# Google Continuous Fuzzing Techniques

## Coverage-Guided Fuzzing

The foundation of modern fuzzing: use code coverage to guide input generation.

### How It Works

```
┌─────────────────────────────────────────────────────────┐
│                   Fuzzing Engine                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │   Corpus    │───►│   Mutate    │───►│  Execute    │ │
│  │  (Seeds)    │    │   Input     │    │   Target    │ │
│  └─────────────┘    └─────────────┘    └──────┬──────┘ │
│         ▲                                      │        │
│         │                                      ▼        │
│  ┌──────┴──────┐                      ┌─────────────┐  │
│  │  Add if     │◄─────────────────────│  Collect    │  │
│  │  New Path   │                      │  Coverage   │  │
│  └─────────────┘                      └─────────────┘  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Coverage Types

| Type | Description | Granularity |
|------|-------------|-------------|
| Edge | Transitions between basic blocks | High |
| Block | Basic block execution | Medium |
| Function | Function entry | Low |
| Line | Source line execution | Very High |

## Mutation Strategies

### Bit Flipping

```c
// Flip random bits
void bit_flip(uint8_t *data, size_t len) {
    size_t pos = rand() % len;
    size_t bit = rand() % 8;
    data[pos] ^= (1 << bit);
}
```

### Byte Replacement

```c
// Replace with interesting values
static const uint8_t interesting_8[] = {
    0x00, 0x01, 0x7f, 0x80, 0xff
};

void byte_replace(uint8_t *data, size_t len) {
    size_t pos = rand() % len;
    data[pos] = interesting_8[rand() % sizeof(interesting_8)];
}
```

### Arithmetic Mutations

```c
// Add/subtract small values
static const int32_t arith_vals[] = {
    -35, -34, -33, /* ... */ 33, 34, 35
};

void arithmetic_mutate(uint8_t *data, size_t len) {
    size_t pos = rand() % (len - 3);
    int32_t *val = (int32_t *)(data + pos);
    *val += arith_vals[rand() % sizeof(arith_vals)/sizeof(int32_t)];
}
```

### Dictionary-Based

```c
// Insert known tokens
const char *dict[] = {
    "GET", "POST", "HTTP/1.1",
    "Content-Length:", "Host:",
    "\r\n", "\r\n\r\n"
};

void dict_insert(uint8_t *data, size_t *len, size_t max_len) {
    const char *token = dict[rand() % (sizeof(dict)/sizeof(char*))];
    size_t token_len = strlen(token);
    size_t pos = rand() % *len;
    
    if (*len + token_len <= max_len) {
        memmove(data + pos + token_len, data + pos, *len - pos);
        memcpy(data + pos, token, token_len);
        *len += token_len;
    }
}
```

## Structure-Aware Fuzzing

### Protobuf Mutator (libprotobuf-mutator)

Define structure, mutate intelligently:

```protobuf
// config.proto
message Config {
    string hostname = 1;
    uint32 port = 2;
    repeated string options = 3;
    enum Mode {
        NORMAL = 0;
        DEBUG = 1;
    }
    Mode mode = 4;
}
```

```cpp
// Fuzz harness using protobuf mutator
#include "libprotobuf-mutator/src/libfuzzer/libfuzzer_macro.h"
#include "config.pb.h"

DEFINE_PROTO_FUZZER(const Config& config) {
    process_config(config.hostname(), 
                   config.port(),
                   config.mode());
}
```

### Grammar-Based (libFuzzer custom mutator)

```cpp
extern "C" size_t LLVMFuzzerCustomMutator(
    uint8_t *Data, size_t Size, 
    size_t MaxSize, unsigned int Seed) {
    
    // Parse as JSON
    json parsed = json::parse(Data, Data + Size, nullptr, false);
    if (parsed.is_discarded()) {
        // Generate valid JSON skeleton
        parsed = {{"key", "value"}, {"num", 0}};
    }
    
    // Mutate while maintaining structure
    mutate_json(parsed, Seed);
    
    // Serialize back
    std::string out = parsed.dump();
    if (out.size() > MaxSize) out.resize(MaxSize);
    memcpy(Data, out.data(), out.size());
    return out.size();
}
```

## OSS-Fuzz Integration

### Project Configuration

```yaml
# project.yaml
homepage: "https://github.com/example/project"
language: c++
primary_contact: "maintainer@example.com"
auto_ccs:
  - "security@example.com"
fuzzing_engines:
  - libfuzzer
  - afl
  - honggfuzz
sanitizers:
  - address
  - memory
  - undefined
```

### Build Script

```bash
#!/bin/bash
# build.sh

# Build project with fuzzing instrumentation
mkdir build && cd build
cmake .. -DCMAKE_C_COMPILER=$CC \
         -DCMAKE_CXX_COMPILER=$CXX \
         -DCMAKE_C_FLAGS="$CFLAGS" \
         -DCMAKE_CXX_FLAGS="$CXXFLAGS"
make -j$(nproc)

# Copy fuzzers to output
cp *_fuzzer $OUT/

# Copy seed corpus
cp -r ../corpus/* $OUT/

# Copy dictionaries
cp ../dictionaries/*.dict $OUT/
```

### Dockerfile

```dockerfile
FROM gcr.io/oss-fuzz-base/base-builder

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config

RUN git clone --depth 1 https://github.com/example/project

WORKDIR project
COPY build.sh $SRC/
```

## Sanitizers

### AddressSanitizer (ASan)

Detects:
- Heap/stack/global buffer overflow
- Use-after-free
- Double-free
- Memory leaks

```c
// Detected by ASan
void bug() {
    char *buf = malloc(10);
    buf[10] = 'x';  // Heap buffer overflow
    free(buf);
    buf[0] = 'y';   // Use after free
}
```

### MemorySanitizer (MSan)

Detects uninitialized memory reads:

```c
// Detected by MSan
void bug() {
    int x;
    if (x) {  // Uninitialized read
        do_something();
    }
}
```

### UndefinedBehaviorSanitizer (UBSan)

Detects:
- Integer overflow
- Null pointer dereference
- Invalid shifts

```c
// Detected by UBSan
void bug(int x) {
    int y = x << 40;     // Shift overflow
    int z = INT_MAX + 1; // Signed overflow
}
```

## Metrics and Reporting

### Coverage Metrics

```
Function coverage: 85.2%
Line coverage: 72.4%
Region coverage: 68.9%
Branch coverage: 61.3%
```

### Crash Triage

```
Unique crashes: 15
  - SEGV: 8
  - ABRT (ASan): 5
  - Timeout: 2

Root causes:
  - Buffer overflow in parse_header(): 3 crashes
  - Use-after-free in connection_close(): 2 crashes
  - Integer overflow in calc_size(): 1 crash
```

## Best Practices

1. **Start with good seeds** - Valid inputs that exercise different paths
2. **Use dictionaries** - Protocol tokens, magic values
3. **Enable all sanitizers** - Different sanitizers find different bugs
4. **Continuous integration** - Fuzz on every commit
5. **Triage quickly** - Fix bugs before they multiply
6. **Measure coverage** - Track progress over time
