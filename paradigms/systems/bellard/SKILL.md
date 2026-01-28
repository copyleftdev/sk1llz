---
name: bellard-minimalist-wizardry
description: Write systems software in the style of Fabrice Bellard, creator of QEMU, FFmpeg, TinyCC, and JSLinux. Emphasizes extreme minimalism, solo mastery of complex domains, and making the impossible seem simple. Use when building emulators, compilers, codecs, or any system where performance and code density matter.
---

# Fabrice Bellard Style Guide

## Overview

Fabrice Bellard created QEMU (the universal machine emulator), FFmpeg (the multimedia framework), TinyCC (a tiny C compiler), JSLinux (Linux in a browser), and computed record digits of pi. He's arguably the most prolific solo systems programmer alive, repeatedly delivering production-quality systems that others would staff entire teams to build.

## Core Philosophy

> "The best code is code you don't write."

> "Understand the problem deeply before writing a single line."

> "Constraints breed creativity."

Bellard believes in deep understanding over brute force—knowing the domain so well that elegant, minimal solutions become obvious.

## Design Principles

1. **Radical Minimalism**: Every line must earn its place.

2. **Deep Domain Mastery**: Understand the spec better than anyone.

3. **Solo Excellence**: One person can build world-class systems.

4. **Performance Through Simplicity**: Simple code is often fastest.

## When Writing Systems Code

### Always

- Read the specification thoroughly before coding
- Start with the simplest possible implementation
- Profile before optimizing
- Keep the entire system in your head
- Write portable C that compiles anywhere
- Release working code, then iterate

### Never

- Over-engineer the first version
- Use frameworks when libraries suffice
- Add abstraction without clear benefit
- Write code you don't fully understand
- Optimize without measurements
- Let code grow without pruning

### Prefer

- C for systems code (maximum control)
- Tables over code (data-driven design)
- Integer math over floating point
- Static allocation over dynamic
- Single-file implementations when possible
- Bitwise operations for flags and state

## Code Patterns

### TinyCC: A C Compiler in 100KB

```c
// TinyCC philosophy: minimal, fast, self-hosting
// Compiles C faster than GCC can parse it

// Token representation: compact and efficient
typedef struct {
    int type;
    int value;
    char *str;
} Token;

// Simple recursive descent parsing
void parse_declaration(void) {
    int type = parse_type();
    while (tok != ';') {
        char *name = parse_declarator(type);
        if (tok == '=') {
            next();
            parse_initializer();
        }
        if (tok == ',') next();
    }
    expect(';');
}

// Code generation: direct to machine code
void gen_op(int op) {
    // Emit x86 directly, no intermediate representation
    switch (op) {
    case '+':
        o(0x01); o(0xc0 | (REG_EAX << 3) | vtop->r);
        break;
    case '*':
        o(0x0f); o(0xaf); o(0xc0 | (REG_EAX << 3) | vtop->r);
        break;
    }
}

// Key insight: for fast compilation, skip optimization passes
// Generate decent code directly, let the programmer optimize
```

### QEMU: Dynamic Binary Translation

```c
// QEMU's core insight: translate guest code to host code dynamically
// Don't interpret—compile on the fly

// Translation block: cached compiled code
typedef struct TranslationBlock {
    target_ulong pc;           // Guest program counter
    void *tc_ptr;              // Host code pointer
    struct TranslationBlock *next;
    // ... minimal metadata
} TranslationBlock;

// TCG (Tiny Code Generator): portable intermediate ops
// Translates to any host architecture

void tcg_gen_add_i32(TCGv_i32 ret, TCGv_i32 arg1, TCGv_i32 arg2) {
    tcg_gen_op3_i32(INDEX_op_add_i32, ret, arg1, arg2);
}

// Hot path: execute translated blocks directly
static inline void cpu_loop_exec_tb(CPUState *cpu, TranslationBlock *tb) {
    // Jump directly into generated host code
    // No interpretation overhead
    tcg_qemu_tb_exec(cpu->env_ptr, tb->tc_ptr);
}

// Brilliant insight: softmmu for memory translation
// Map guest addresses to host addresses with TLB
static inline void *tlb_lookup(CPUState *cpu, target_ulong addr) {
    int index = (addr >> TARGET_PAGE_BITS) & (CPU_TLB_SIZE - 1);
    if (cpu->tlb_table[index].addr_read == (addr & TARGET_PAGE_MASK)) {
        return (void *)(addr + cpu->tlb_table[index].addend);
    }
    return tlb_fill_slowpath(cpu, addr);  // Page fault handling
}
```

### FFmpeg: Multimedia Swiss Army Knife

```c
// FFmpeg: decode anything, encode anything
// Data-driven codec registration

// Codec structure: interface for all codecs
typedef struct AVCodec {
    const char *name;
    enum AVMediaType type;
    enum AVCodecID id;
    int (*init)(AVCodecContext *);
    int (*encode)(AVCodecContext *, AVPacket *, const AVFrame *, int *);
    int (*decode)(AVCodecContext *, AVFrame *, int *, AVPacket *);
    int (*close)(AVCodecContext *);
    // ... capabilities, profiles
} AVCodec;

// Codec registration: simple linked list
static AVCodec *first_avcodec = NULL;

void avcodec_register(AVCodec *codec) {
    codec->next = first_avcodec;
    first_avcodec = codec;
}

// SIMD optimization: hand-written for each architecture
// But with clean C fallbacks

void ff_h264_idct_add_c(uint8_t *dst, int16_t *block, int stride) {
    // Pure C implementation
    for (int i = 0; i < 4; i++) {
        // 1D IDCT on rows
        int a = block[0] + block[2];
        int b = block[0] - block[2];
        int c = (block[1] >> 1) - block[3];
        int d = block[1] + (block[3] >> 1);
        // ...
    }
}

// x86 SIMD version selected at runtime
void ff_h264_idct_add_sse2(uint8_t *dst, int16_t *block, int stride);
```

### Table-Driven Design

```c
// Bellard loves tables: data over code
// Easier to verify, often faster

// H.264 CAVLC tables
static const uint8_t coeff_token_vlc[4][17][4] = {
    // nC < 2
    {{1, 1, 0, 0}, {6, 5, 0, 1}, {8, 7, 1, 1}, ...},
    // nC < 4
    {{2, 2, 0, 0}, {6, 5, 0, 1}, {6, 5, 1, 1}, ...},
    // ...
};

// State machine as table
typedef enum { S_START, S_NUMBER, S_STRING, S_END } State;

static const State transitions[S_END][256] = {
    [S_START] = {
        ['0' ... '9'] = S_NUMBER,
        ['"'] = S_STRING,
        [' '] = S_START,
    },
    // ...
};

State next_state(State current, char c) {
    return transitions[current][(unsigned char)c];
}
```

### Integer Math for Precision

```c
// Avoid floating point when possible
// Integer math is exact and often faster

// Fixed-point arithmetic for audio resampling
#define FRAC_BITS 16
#define FRAC_ONE (1 << FRAC_BITS)

int resample(int16_t *out, int16_t *in, int in_len, int ratio) {
    int64_t pos = 0;  // Fixed-point position
    int out_idx = 0;
    
    while (pos < ((int64_t)in_len << FRAC_BITS)) {
        int idx = pos >> FRAC_BITS;
        int frac = pos & (FRAC_ONE - 1);
        
        // Linear interpolation with fixed-point
        out[out_idx++] = (in[idx] * (FRAC_ONE - frac) + 
                         in[idx + 1] * frac) >> FRAC_BITS;
        pos += ratio;
    }
    return out_idx;
}

// Pi computation: no floating point anywhere
// Uses Chudnovsky algorithm with arbitrary precision integers
```

### Single-File Mastery

```c
// JSLinux: Linux emulator in a single HTML file
// Everything needed to boot Linux in one file

// Minimal PC emulator structure
typedef struct {
    uint8_t *mem;
    uint32_t regs[8];
    uint32_t eip;
    uint32_t eflags;
    // I/O devices
    struct {
        uint8_t data[16];
        int read_pos, write_pos;
    } serial;
    // ...
} PCState;

// x86 instruction decoder: compact table-driven
static void exec_instruction(PCState *s) {
    uint8_t op = fetch_byte(s);
    
    switch (op) {
    case 0x89:  // MOV r/m32, r32
        modrm = fetch_byte(s);
        decode_modrm(s, modrm, &addr, &reg);
        write_mem32(s, addr, s->regs[reg]);
        break;
    case 0x8b:  // MOV r32, r/m32
        modrm = fetch_byte(s);
        decode_modrm(s, modrm, &addr, &reg);
        s->regs[reg] = read_mem32(s, addr);
        break;
    // ... complete x86 instruction set
    }
}
```

### Portable Performance

```c
// Write portable C, optimize hot paths per platform
// Clean abstraction between portable and platform-specific

// Portable interface
void *page_alloc(size_t size);
void page_protect(void *addr, size_t size, int flags);

// Platform implementations
#ifdef _WIN32
void *page_alloc(size_t size) {
    return VirtualAlloc(NULL, size, MEM_COMMIT, PAGE_READWRITE);
}
#else
void *page_alloc(size_t size) {
    return mmap(NULL, size, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
}
#endif

// Runtime CPU feature detection
static int has_sse2 = 0;

void init_cpu_features(void) {
#if defined(__i386__) || defined(__x86_64__)
    uint32_t eax, ebx, ecx, edx;
    __cpuid(1, eax, ebx, ecx, edx);
    has_sse2 = (edx >> 26) & 1;
#endif
}
```

## Project Scope Philosophy

```
Bellard's Project Characteristics
══════════════════════════════════════════════════════════════

Project     Lines of Code    What It Does
────────────────────────────────────────────────────────────
TinyCC      ~30,000         Full C99 compiler + linker
QEMU        ~500,000*       Universal machine emulator
FFmpeg      ~1,000,000*     All multimedia formats
JSLinux     ~10,000         PC emulator in JavaScript

*Grew over time; Bellard's initial versions much smaller

Key insight: Start minimal, prove the concept works,
            then expand based on real needs.
```

## Mental Model

Bellard approaches problems by asking:

1. **What's the essence?** Strip away everything non-essential
2. **What do the specs actually say?** Read them completely
3. **What's the minimal viable implementation?** Start there
4. **Where's the hot path?** Optimize only what matters
5. **Can one person maintain this?** Complexity is the enemy

## Signature Bellard Moves

- **TinyCC's speed**: Compile fast enough to use as a scripting language
- **QEMU's TCG**: Dynamic translation that's portable across hosts
- **FFmpeg's codec zoo**: Support everything through uniform interfaces
- **JSLinux**: Boot Linux in a browser, because why not
- **Pi computation**: World records with algorithms, not hardware
- **Self-hosting compilers**: TCC compiles itself
- **Single-file deployments**: Reduce dependencies to zero
