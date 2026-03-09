# Raph Levien's Philosophy⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍​‌​​‌​​‌‍​‌​‌​‌‌‌‍​​​​‌​‌​‍​​‌​​‌‌​‍​​​​‌​​‌‍​​‌‌‌​‌​⁠‍⁠

## The Tree Transformation Model

Levien's central insight is that **reactive UI is a pipeline of tree transformations**. This model unifies seemingly disparate frameworks under one theoretical construct.

### The Pipeline

```text
App State → View Tree → Widget Tree → Render Tree → Draw Commands → Pixels
```

Each stage transforms one tree into the next:

1. **App State → View Tree**: Application logic produces a declarative description of desired UI
2. **View Tree → Widget Tree**: Framework diffs view trees and applies changes to retained widgets
3. **Widget Tree → Render Tree**: Widgets produce render objects with layout information
4. **Render Tree → Draw Commands**: Layout is resolved, paint operations are generated
5. **Draw Commands → Pixels**: GPU (or CPU) rasterizes to the screen

### Two Ways to Represent a Tree

A key observation: trees can be represented in two fundamental ways:

1. **As a data structure**: Nodes in memory, parents own children
2. **As a trace of execution**: A sequence of events from traversing the tree (often a call stack)

These representations are interchangeable. A traversal generates a trace; a trace can be annotated to build a structure. Different UI frameworks make different choices at each pipeline stage—this explains much of their apparent diversity.

### Why This Matters

The pipeline model reveals:

- **Where state lives**: Each stage can have its own state (e.g., splitter position belongs to the widget stage, not app logic)
- **Coupling points**: Clear boundaries between stages allow different implementations, languages, or processes
- **Optimization opportunities**: Each stage can be incrementalized independently

## Why Immediate Mode Falls Short

Immediate mode GUI (imgui, egui) is popular in Rust because it avoids shared mutable state. But Levien identifies fundamental limitations:

### The Accessibility Problem

Screen readers expect a persistent tree with stable identity. Pure immediate mode provides neither:

```rust
// Immediate mode: no retained structure
fn ui(ctx: &mut Context) {
    if button(ctx, "Click me") {
        // handle click
    }
}
// Where does the screen reader get the button from? 
// It's gone after this frame.
```

Accessibility *requires* a retained widget tree. This forces immediate mode frameworks toward hybrid approaches—emulating immediate mode on top of retained structure.

### The Layout Problem

Complex layouts (virtualized scrolling, multi-pass layouts) need retained state:

```rust
// Virtualized list: only materialize visible items
// Immediate mode: must iterate all 100,000 items to know heights
// Retained mode: cache heights, only render visible window
```

### The Verdict

> "For the accessibility part (as opposed to the GPU-drawn part) of the UI, pure immediate mode cannot be used, only a hybrid approach which resembles emulation of an immediate mode API on top of a more traditional retained structure."

Immediate mode is a viable *API* but not a viable *implementation* for production UI.

## Why Elm Architecture Falls Short

The Elm Architecture (TEA) is another popular Rust choice (Iced, Relm). It avoids shared mutable state by using message passing. But Levien finds it wanting:

### The Verbosity Problem

Every interaction requires explicit message types:

```rust
// Elm-style: explicit messages for everything
enum Msg {
    IncrementCount,
    DecrementCount,
    SetUsername(String),
    ToggleDarkMode,
    // ... hundreds more
}

fn update(state: &mut State, msg: Msg) {
    match msg {
        Msg::IncrementCount => state.count += 1,
        // ... handle every message
    }
}
```

This is verbose. Every button needs a message type. Every field needs a setter message.

### The Component Problem

Elm's own documentation warns against components:

> "Actively trying to make components is a recipe for disaster in Elm."

In TEA, nested components require message wrapping and unwrapping at each level. State is either global (bad for encapsulation) or requires explicit message translation (verbose).

### The Alternative: Adapt Nodes

Levien's solution is Adapt nodes—closures that translate between state types:

```rust
// Xilem-style: closure provides mutable access to state slice
Adapt::new(
    |app: &mut AppState, thunk| thunk.call(&mut app.settings),
    settings_component,  // Just takes &mut SettingsState
)
```

No message types. No dispatch. Just scoped mutable access.

## The Xilem Hypothesis

Levien's hypothesis:

> "Xilem is the best known reactive architecture for Rust... more concise, more ergonomic, more efficient, and better integrated with async than comparable work."

### Key Innovations

1. **Typed view trees**: The view tree is statically typed; the compiler infers state and widget tree types via associated types

2. **Id-path event routing**: Events carry an id path; at each tree level, the handler gets mutable access to its state slice

3. **Adapt nodes**: Components define their own state type; Adapt nodes translate between parent and child

4. **Built-in incrementalization**: Memoization is first-class; the framework tracks what changed

### The Test

The hypothesis is testable only by building real systems. If Xilem succeeds, it proves the architecture. If it fails, alternatives (Dioxus, Iced, Sycamore) are "good enough."

## Research as Practice

Levien operates as a researcher-practitioner:

- **Output is code and blog posts**, not academic papers
- **Primary literature is the blog**: raphlinus.github.io
- **Knowledge is "lore"**: UI and 2D graphics lack textbooks and conferences
- **Community is essential**: Weekly office hours, Zulip chat, collaborative PRs

This approach means:

- Ideas are tested by implementation, not proof
- Architecture evolves through building (Druid → Crochet → Xilem)
- Negative results are valuable (Crochet was "mostly negative")

## On GPU Rendering

Levien's Vello project embodies a radical philosophy:

> "The CPU uploads a scene description... then the compute shaders take care of the rest."

### Traditional vs. Vello

| Traditional (Cairo, Skia)       | Vello                |
|---------------------------------|----------------------|
| CPU parses scene                | GPU parses scene     |
| CPU flattens curves             | GPU flattens curves  |
| CPU tiles and bins              | GPU tiles and bins   |
| CPU rasterizes (or limited GPU) | GPU rasterizes fully |

### Why GPU Rendering Matters

The GPU has thousands of cores. Traditional 2D renderers serialize work on the CPU, using the GPU only for final compositing. Vello parallelizes *everything*—parsing, path processing, rasterization.

This requires:

- Designing algorithms for massive parallelism
- Encoding scenes in GPU-friendly binary formats
- Solving hard problems (Euler spirals for strokes, robust path operations)

### The Payoff

When it works, Vello can render complex scenes at 120fps where traditional renderers struggle at 60fps. This unlocks:

- Smooth animations
- Complex vector graphics
- High-DPI displays without compromise

## On Curves and Typography

Levien's PhD work was on font curve fitting. This deep expertise informs:

### Path Geometry

- **Euler spirals** (cornu spirals): Better than Bézier for stroke expansion
- **Parallel curves**: Mathematically correct offset paths
- **Robust boolean operations**: Intersection, union, difference of paths

### Font Rendering

- **Stem darkening**: Making thin fonts readable at small sizes
- **Hinting**: Aligning outlines to pixel grid
- **Variable fonts**: Interpolating between masters

### The Connection to UI

Typography is 90% of UI. Getting text right—legible, beautiful, fast—is essential for native-quality UIs. Levien's font expertise directly serves the UI goal.

## The Happiness Principle

> "My happiness tends to correlate pretty directly with how much code I'm writing."

This is not incidental. Levien's productivity comes from:

1. **Deep puzzle-solving**: Adapting algorithms for GPU parallelism
2. **Teaching through writing**: Blog posts, documentation, talks
3. **Community building**: Not "licking the cookie"—empowering others to contribute

The implication: sustainable open source requires maintainers who genuinely enjoy the work, not martyrs burning out on thankless labor.
