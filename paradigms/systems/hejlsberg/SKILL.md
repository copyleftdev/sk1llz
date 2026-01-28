---
name: hejlsberg-language-design
description: Design languages and type systems in the style of Anders Hejlsberg, creator of Turbo Pascal, Delphi, C#, and TypeScript. Emphasizes practical type systems, developer productivity, gradual typing, and IDE-driven language design. Use when designing languages, type systems, or developer tools.
---

# Anders Hejlsberg Style Guide

## Overview

Anders Hejlsberg created Turbo Pascal (the fastest compiler of its era), Delphi (RAD with native compilation), C# (managed language with modern features), and TypeScript (typed JavaScript at scale). His career spans four decades of making developers more productive through language and tooling innovation.

## Core Philosophy

> "A language is only as good as its tooling."

> "Types should help you, not get in your way."

> "The best type system is one that understands existing code."

Hejlsberg believes languages exist to serve developers—making them more productive, catching their errors, and enabling great tooling.

## Design Principles

1. **Developer Productivity First**: Every feature should make developers faster.

2. **Gradual Adoption**: Meet developers where they are.

3. **IDE-Driven Design**: Consider tooling from day one.

4. **Pragmatic Type Systems**: Types that work with real code, not against it.

## When Designing Languages

### Always

- Consider the IDE experience for every feature
- Provide escape hatches for edge cases
- Enable gradual adoption of type safety
- Make common patterns easy
- Design for tooling (completion, refactoring, navigation)
- Maintain backward compatibility

### Never

- Sacrifice developer experience for type purity
- Require full type coverage from day one
- Break existing code without migration paths
- Design features that can't be tooled
- Ignore the ecosystem of existing code
- Make simple things verbose

### Prefer

- Structural typing over nominal (for flexibility)
- Type inference over explicit annotations
- Gradual typing over all-or-nothing
- Composition over inheritance
- Async/await over callbacks
- Null safety without verbosity

## Code Patterns

### Structural Typing (TypeScript)

```typescript
// TypeScript: structural typing for flexibility
// Types are shapes, not names

interface Point {
    x: number;
    y: number;
}

function distance(p1: Point, p2: Point): number {
    return Math.sqrt((p2.x - p1.x) ** 2 + (p2.y - p1.y) ** 2);
}

// Any object with x and y works
const a = { x: 0, y: 0 };
const b = { x: 3, y: 4, label: "target" };  // Extra properties OK

distance(a, b);  // Works! Both match Point's shape

// This enables typing existing JavaScript code
// without requiring changes
```

### Union Types and Type Guards

```typescript
// Unions: represent values that could be multiple types
type Result<T> = 
    | { success: true; value: T }
    | { success: false; error: string };

function process<T>(result: Result<T>): T | null {
    // Type narrowing: compiler tracks which branch we're in
    if (result.success) {
        // TypeScript knows: result.value exists here
        return result.value;
    } else {
        // TypeScript knows: result.error exists here
        console.error(result.error);
        return null;
    }
}

// Discriminated unions for state machines
type State = 
    | { status: "idle" }
    | { status: "loading" }
    | { status: "success"; data: string }
    | { status: "error"; message: string };

function render(state: State): string {
    switch (state.status) {
        case "idle": return "Ready";
        case "loading": return "Loading...";
        case "success": return state.data;  // data available
        case "error": return state.message;  // message available
    }
}
```

### Generics with Constraints

```typescript
// Generics that work with real patterns

// Constrain to ensure properties exist
function pluck<T, K extends keyof T>(items: T[], key: K): T[K][] {
    return items.map(item => item[key]);
}

const users = [
    { name: "Alice", age: 30 },
    { name: "Bob", age: 25 }
];

const names = pluck(users, "name");  // string[]
const ages = pluck(users, "age");    // number[]
// pluck(users, "foo");              // Error: "foo" not in User

// Mapped types: transform type shapes
type Readonly<T> = {
    readonly [P in keyof T]: T[P];
};

type Partial<T> = {
    [P in keyof T]?: T[P];
};

type Required<T> = {
    [P in keyof T]-?: T[P];
};

// Conditional types: types that compute
type NonNullable<T> = T extends null | undefined ? never : T;
type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;
```

### Async/Await (C# Innovation)

```csharp
// C#: async/await pioneered by Hejlsberg's team
// Asynchronous code that reads like synchronous

public async Task<User> GetUserWithOrdersAsync(int userId)
{
    // Each await yields control, resumes when ready
    var user = await _userRepository.GetByIdAsync(userId);
    
    if (user == null)
        return null;
    
    // Parallel async operations
    var ordersTask = _orderRepository.GetByUserAsync(userId);
    var preferencesTask = _preferenceRepository.GetAsync(userId);
    
    await Task.WhenAll(ordersTask, preferencesTask);
    
    user.Orders = ordersTask.Result;
    user.Preferences = preferencesTask.Result;
    
    return user;
}

// The compiler transforms this into a state machine
// Developer writes linear code, gets async behavior
```

### LINQ: Language-Integrated Query

```csharp
// LINQ: query syntax integrated into the language
// Type-safe, composable, provider-agnostic

var results = from user in users
              where user.Age >= 18
              orderby user.Name
              select new { user.Name, user.Email };

// Method syntax (equivalent)
var results = users
    .Where(u => u.Age >= 18)
    .OrderBy(u => u.Name)
    .Select(u => new { u.Name, u.Email });

// Works with any data source
var dbResults = from order in db.Orders
                join customer in db.Customers 
                    on order.CustomerId equals customer.Id
                where order.Total > 1000
                select new { customer.Name, order.Total };

// Deferred execution: query builds an expression tree
// Execution happens when results are enumerated
```

### Null Safety Without Pain

```csharp
// C# nullable reference types: gradual null safety

// Enable per-file or per-project
#nullable enable

public class UserService
{
    // Non-nullable: must not be null
    public string GetDisplayName(User user)
    {
        return user.Name;  // user cannot be null
    }
    
    // Nullable: explicitly might be null
    public User? FindUser(string email)
    {
        return _repository.FindByEmail(email);
    }
    
    // Compiler tracks null state
    public void ProcessUser(string email)
    {
        var user = FindUser(email);
        
        // user.Name;  // Warning: possible null reference
        
        if (user != null)
        {
            // Compiler knows user is not null here
            Console.WriteLine(user.Name);  // OK
        }
        
        // Null-coalescing operators
        var name = user?.Name ?? "Unknown";
    }
}
```

### Pattern Matching Evolution

```csharp
// C# pattern matching: evolved over versions

// Type patterns
if (obj is string s)
{
    Console.WriteLine(s.Length);
}

// Switch expressions with patterns
var description = shape switch
{
    Circle { Radius: 0 } => "Point",
    Circle { Radius: var r } => $"Circle with radius {r}",
    Rectangle { Width: var w, Height: var h } when w == h => $"Square {w}x{w}",
    Rectangle { Width: var w, Height: var h } => $"Rectangle {w}x{h}",
    _ => "Unknown shape"
};

// List patterns (C# 11)
var result = numbers switch
{
    [] => "Empty",
    [var single] => $"Single: {single}",
    [var first, .., var last] => $"First: {first}, Last: {last}",
    _ => "Other"
};
```

### Type Inference Done Right

```typescript
// TypeScript: extensive type inference
// Types inferred from usage, not declarations

// Inferred from initializer
const message = "hello";  // string
const count = 42;         // number
const items = [1, 2, 3];  // number[]

// Inferred from context
const doubled = items.map(x => x * 2);  // number[]
// x is inferred as number from items being number[]

// Inferred return types
function createUser(name: string, age: number) {
    return { name, age, createdAt: new Date() };
}
// Return type inferred as { name: string; age: number; createdAt: Date }

// Generic inference
function first<T>(items: T[]): T | undefined {
    return items[0];
}

const n = first([1, 2, 3]);      // number | undefined
const s = first(["a", "b"]);     // string | undefined
// T is inferred from the argument
```

### IDE-First Feature Design

```typescript
// Features designed with IDE support in mind

// 1. Go to definition works through types
interface UserService {
    getUser(id: string): Promise<User>;
    saveUser(user: User): Promise<void>;
}

// Ctrl+Click on getUser goes to interface definition

// 2. Rename works safely
class OrderProcessor {
    processOrder(order: Order) {  // Rename "order" → all usages update
        this.validate(order);
        this.save(order);
    }
}

// 3. Completion knows valid options
type Status = "pending" | "active" | "completed";
const status: Status = "|"  // Autocomplete shows: pending, active, completed

// 4. Quick fixes provide migrations
const config = {
    name: "app",
    // @ts-expect-error - Typing to add later
    untyped: someValue,  // IDE suggests: Add type annotation
};
```

## Language Evolution Philosophy

```
Gradual Typing Adoption Path
══════════════════════════════════════════════════════════════

Phase           Type Coverage    Developer Action
────────────────────────────────────────────────────────────
1. Baseline     0%              Rename .js → .ts, compiles!
2. Implicit     20%             Add types to public APIs
3. Strict       60%             Enable strict null checks
4. Complete     90%+            Full coverage, all strict

Key insight: Each phase provides value
            No phase requires rewriting code
            Migration can be file-by-file
```

## Mental Model

Hejlsberg approaches language design by asking:

1. **What's the developer experience?** Features must be usable
2. **How does this work in the IDE?** Tooling is part of the design
3. **Can this be adopted gradually?** Don't require big-bang rewrites
4. **Does this match real code patterns?** Type systems should model reality
5. **Is there an escape hatch?** Sometimes types are too strict

## Signature Hejlsberg Moves

- **Turbo Pascal speed**: Compilation so fast it changed expectations
- **Delphi's RAD**: Visual design with native performance
- **C# async/await**: Made async mainstream and readable
- **LINQ**: Query syntax integrated into the language
- **TypeScript's structural typing**: Types for JavaScript at scale
- **Gradual typing**: Adopt at your own pace
- **Nullable reference types**: Null safety without rewrites
- **Mapped/conditional types**: Type-level computation
