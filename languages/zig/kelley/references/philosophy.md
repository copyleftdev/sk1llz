# Andrew Kelley Philosophy

## The Zig Way

Kelley designed Zig to fix what he saw as fundamental problems in systems programming languages.

> "Zig is a general-purpose programming language and toolchain designed to be used to maintain reliable, low-latency, real-time software and systems at scale."

## Core Design Principles

### 1. No Hidden Control Flow

Unlike C++, Zig has no:
- Hidden function calls (operators, destructors)
- Hidden allocations
- Hidden exceptions

```zig
// What you see is what you get
const result = try doSomething();  // Explicit error handling
defer cleanup();                    // Explicit deferred execution
```

### 2. No Hidden Memory Allocations

Every allocation is explicit:

```zig
// Allocator is always passed explicitly
pub fn init(allocator: std.mem.Allocator) !Self {
    const buffer = try allocator.alloc(u8, 1024);
    return Self{ .buffer = buffer, .allocator = allocator };
}

pub fn deinit(self: *Self) void {
    self.allocator.free(self.buffer);
}
```

### 3. No Undefined Behavior (by default)

Zig detects UB at compile time and runtime:

```zig
// Integer overflow is detected
var x: u8 = 255;
x += 1;  // Panic in safe modes, wrapping in ReleaseFast

// Explicit wrapping when you want it
x +%= 1;  // Always wraps

// Out of bounds detected
var arr = [_]u8{ 1, 2, 3 };
const val = arr[index];  // Bounds checked
```

### 4. Communicate Intent to Compiler and Humans

```zig
// comptime tells both compiler and reader this is compile-time
fn fibonacci(comptime n: u32) comptime_int {
    if (n < 2) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

const fib10 = fibonacci(10);  // Computed at compile time
```

## Comptime Philosophy

Kelley's key insight: Replace metaprogramming with computation.

### No Macros, No Templates

```zig
// Generic function - just a function
fn max(comptime T: type, a: T, b: T) T {
    return if (a > b) a else b;
}

// Generic type - just returns a type
fn ArrayList(comptime T: type) type {
    return struct {
        items: []T,
        capacity: usize,
        allocator: std.mem.Allocator,
        
        pub fn append(self: *@This(), item: T) !void {
            // ...
        }
    };
}
```

### Compile-Time Reflection

```zig
fn dumpFields(comptime T: type) void {
    const info = @typeInfo(T);
    inline for (info.Struct.fields) |field| {
        @compileLog(field.name, field.type);
    }
}
```

## Error Handling Philosophy

### Errors Are Values, Not Exceptions

```zig
// Error set is a type
const FileError = error{
    NotFound,
    PermissionDenied,
    IsDirectory,
};

fn openFile(path: []const u8) FileError!File {
    // ...
}

// Explicit handling
const file = openFile("test.txt") catch |err| switch (err) {
    error.NotFound => return fallback(),
    error.PermissionDenied => return error.AccessDenied,
    else => return err,
};
```

### Error Return Traces

Unlike stack traces that hide the problem, error return traces show exactly where errors originated and propagated.

## Memory Safety Philosophy

### Safe by Default, Unsafe When Needed

```zig
// Safe: bounds-checked, null-checked
var slice: []u8 = buffer[0..10];
const value = slice[index];

// Unsafe: explicit opt-out for performance-critical code
const ptr: [*]u8 = @ptrCast(raw_memory);
const value = ptr[index];  // No bounds check
```

### No Garbage Collection

> "Zig is not a memory-safe language in the sense that Rust is, but it gives you tools to catch bugs that C doesn't."

## C Interop Philosophy

### C ABI Compatibility is a Feature

```zig
// Call C directly, no FFI ceremony
const c = @cImport({
    @cInclude("stdio.h");
});

pub fn main() void {
    _ = c.printf("Hello from Zig!\n");
}
```

### Zig as a Better C Compiler

```bash
# Compile C code with Zig
zig cc -o myprogram main.c

# Cross-compile trivially
zig cc -target aarch64-linux-gnu main.c
```

## Build System Philosophy

### Build System in the Language

```zig
// build.zig
const std = @import("std");

pub fn build(b: *std.Build) void {
    const exe = b.addExecutable(.{
        .name = "myapp",
        .root_source_file = .{ .path = "src/main.zig" },
        .target = b.standardTargetOptions(.{}),
        .optimize = b.standardOptimizeOption(.{}),
    });
    b.installArtifact(exe);
}
```

## Performance Philosophy

### Zero-Cost Abstractions (Actually Zero)

```zig
// Optional types compile to nullable pointers
const maybe_ptr: ?*u32 = null;  // Same size as *u32

// Slices are fat pointers (ptr + len)
const slice: []u8 = buffer;  // 16 bytes on 64-bit

// No hidden vtables unless you ask
const Interface = struct {
    ptr: *anyopaque,
    vtable: *const VTable,  // Explicit
};
```

### You Can Always Drop Down

```zig
// Start high-level
for (items) |item| {
    process(item);
}

// Drop to low-level when needed
var i: usize = 0;
while (i < items.len) : (i += 1) {
    @prefetch(items.ptr + i + 16);
    process(items[i]);
}
```

## Famous Quotes

> "Zig is not trying to be a 'better C++'. It's trying to be a better C."

> "There's no good reason why C programmers can't have nice things."

> "The goal is to make it so that doing the right thing is the easy thing."

> "If you need to do something at compile time, do it at compile time. Don't invent a separate language for it."

## Key Talks

1. **"The Road to Zig 1.0"** - Design decisions explained
2. **"Zig's Approach to Memory Safety"** - Safety without GC
3. **"Comptime: Zig's Killer Feature"** - Compile-time computation
