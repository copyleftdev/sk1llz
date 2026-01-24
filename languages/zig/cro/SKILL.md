---
name: cro-practical-zig
description: Write Zig code in the style of Loris Cro, VP Community at Zig Software Foundation. Emphasizes practical patterns, build system mastery, and teaching Zig effectively. Use when building real applications or learning Zig idioms.
---

# Loris Cro Style Guide

## Overview

Loris Cro is the VP of Community at the Zig Software Foundation, known for explaining Zig concepts clearly and demonstrating practical applications. His focus is on making Zig accessible and showing how to build real software.

## Core Philosophy

> "Zig's build system is one of its killer features."

> "Start simple, add complexity only when needed."

Cro emphasizes practical application—building real things, understanding the build system, and using Zig's unique features to solve actual problems.

## Design Principles

1. **Build System First**: Understand `build.zig` deeply.

2. **Practical Patterns**: Focus on what works in production.

3. **C Interop**: Leverage existing C libraries seamlessly.

4. **Incremental Adoption**: Use Zig where it helps most.

## When Writing Code

### Always

- Master the build system early
- Use `build.zig` for all project configuration
- Leverage C interop for existing libraries
- Write tests alongside code
- Use `std.log` for structured logging
- Profile before optimizing

### Never

- Fight the build system—learn it
- Rewrite working C code without reason
- Ignore the standard library—it's excellent
- Skip writing tests
- Optimize without measurements

### Prefer

- `build.zig` over external build tools
- Standard library over reinvention
- C library bindings over pure Zig rewrites (when sensible)
- Incremental compilation during development
- Cross-compilation from the start

## Code Patterns

### Build System Mastery

```zig
// build.zig - the heart of a Zig project
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Main executable
    const exe = b.addExecutable(.{
        .name = "myapp",
        .root_source_file = .{ .path = "src/main.zig" },
        .target = target,
        .optimize = optimize,
    });

    // Link C library
    exe.linkLibC();
    exe.linkSystemLibrary("sqlite3");

    // Add include paths
    exe.addIncludePath(.{ .path = "vendor/include" });

    b.installArtifact(exe);

    // Run step
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    const run_step = b.step("run", "Run the application");
    run_step.dependOn(&run_cmd.step);

    // Test step
    const unit_tests = b.addTest(.{
        .root_source_file = .{ .path = "src/main.zig" },
        .target = target,
        .optimize = optimize,
    });

    const run_unit_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_unit_tests.step);
}
```

### C Interoperability

```zig
// Import C headers directly
const c = @cImport({
    @cInclude("stdio.h");
    @cInclude("sqlite3.h");
});

pub fn main() void {
    // Call C functions directly
    _ = c.printf("Hello from C!\n");
}

// Wrap C libraries idiomatically
const Database = struct {
    handle: *c.sqlite3,

    pub fn open(path: [*:0]const u8) !Database {
        var db: ?*c.sqlite3 = null;
        const result = c.sqlite3_open(path, &db);
        if (result != c.SQLITE_OK) {
            return error.DatabaseOpenFailed;
        }
        return .{ .handle = db.? };
    }

    pub fn close(self: *Database) void {
        _ = c.sqlite3_close(self.handle);
    }

    pub fn exec(self: *Database, sql: [*:0]const u8) !void {
        var err_msg: ?[*:0]u8 = null;
        const result = c.sqlite3_exec(
            self.handle,
            sql,
            null,
            null,
            &err_msg,
        );
        if (result != c.SQLITE_OK) {
            if (err_msg) |msg| {
                std.log.err("SQL error: {s}", .{msg});
                c.sqlite3_free(msg);
            }
            return error.SqlExecutionFailed;
        }
    }
};
```

### Structured Logging

```zig
const std = @import("std");

// Scoped logging
const log = std.log.scoped(.myapp);

pub fn processRequest(request_id: u64) !void {
    log.info("Processing request {d}", .{request_id});

    const result = doWork() catch |err| {
        log.err("Request {d} failed: {}", .{ request_id, err });
        return err;
    };

    log.debug("Request {d} result: {any}", .{ request_id, result });
}

// Configure log level at build time
pub const std_options = struct {
    pub const log_level: std.log.Level = .debug;

    // Custom log function
    pub fn logFn(
        comptime level: std.log.Level,
        comptime scope: @TypeOf(.enum_literal),
        comptime format: []const u8,
        args: anytype,
    ) void {
        const scope_prefix = if (scope != .default)
            "[" ++ @tagName(scope) ++ "] "
        else
            "";

        const prefix = "[" ++ level.asText() ++ "] " ++ scope_prefix;

        std.debug.print(prefix ++ format ++ "\n", args);
    }
};
```

### Testing Patterns

```zig
const std = @import("std");
const testing = std.testing;

fn add(a: i32, b: i32) i32 {
    return a + b;
}

test "add basic" {
    try testing.expectEqual(@as(i32, 5), add(2, 3));
}

test "add negative" {
    try testing.expectEqual(@as(i32, -1), add(2, -3));
}

// Test with allocator
test "dynamic allocation" {
    const allocator = testing.allocator;  // Detects leaks!

    var list = std.ArrayList(u8).init(allocator);
    defer list.deinit();

    try list.append(42);
    try testing.expectEqual(@as(usize, 1), list.items.len);
}

// Fuzz testing
test "fuzz example" {
    const input = std.testing.fuzzInput(.{});
    // Process fuzz input...
}
```

### Standard Library Gems

```zig
const std = @import("std");

// ArrayList - dynamic arrays
fn arrayListExample(allocator: std.mem.Allocator) !void {
    var list = std.ArrayList(u32).init(allocator);
    defer list.deinit();

    try list.append(1);
    try list.append(2);
    try list.appendSlice(&[_]u32{ 3, 4, 5 });

    for (list.items) |item| {
        std.debug.print("{d} ", .{item});
    }
}

// HashMap
fn hashMapExample(allocator: std.mem.Allocator) !void {
    var map = std.StringHashMap(u32).init(allocator);
    defer map.deinit();

    try map.put("one", 1);
    try map.put("two", 2);

    if (map.get("one")) |value| {
        std.debug.print("one = {d}\n", .{value});
    }
}

// File I/O
fn fileExample() !void {
    const file = try std.fs.cwd().openFile("data.txt", .{});
    defer file.close();

    var buf_reader = std.io.bufferedReader(file.reader());
    var reader = buf_reader.reader();

    var line_buf: [1024]u8 = undefined;
    while (try reader.readUntilDelimiterOrEof(&line_buf, '\n')) |line| {
        std.debug.print("{s}\n", .{line});
    }
}

// JSON parsing
fn jsonExample(allocator: std.mem.Allocator) !void {
    const json_str =
        \\{"name": "Alice", "age": 30}
    ;

    const User = struct {
        name: []const u8,
        age: u32,
    };

    const parsed = try std.json.parseFromSlice(
        User,
        allocator,
        json_str,
        .{},
    );
    defer parsed.deinit();

    std.debug.print("Name: {s}, Age: {d}\n", .{
        parsed.value.name,
        parsed.value.age,
    });
}
```

### Cross-Compilation

```zig
// build.zig - cross-compile easily
pub fn build(b: *std.Build) void {
    // Default to native
    const target = b.standardTargetOptions(.{});

    // Or target specific platforms:
    // zig build -Dtarget=x86_64-linux-gnu
    // zig build -Dtarget=aarch64-macos
    // zig build -Dtarget=x86_64-windows-gnu

    const exe = b.addExecutable(.{
        .name = "myapp",
        .root_source_file = .{ .path = "src/main.zig" },
        .target = target,
        .optimize = b.standardOptimizeOption(.{}),
    });

    b.installArtifact(exe);
}

// Target-specific code
const builtin = @import("builtin");

fn platformSpecific() void {
    switch (builtin.os.tag) {
        .linux => linuxImpl(),
        .macos => macosImpl(),
        .windows => windowsImpl(),
        else => @compileError("Unsupported platform"),
    }
}
```

## Mental Model

Cro approaches Zig projects by asking:

1. **Is the build system set up right?** Start with `build.zig`
2. **Can I use an existing C library?** Don't reinvent the wheel
3. **Is this tested?** Write tests early and often
4. **Will this cross-compile?** Think portable from the start
5. **Is this practical?** Ship working software

## Signature Cro Moves

- Master `build.zig` before deep language features
- C interop for rapid development
- Standard library fluency
- Tests with leak-detecting allocator
- Cross-compilation as default mindset
- Structured logging from the start
