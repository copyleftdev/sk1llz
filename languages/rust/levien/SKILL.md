---
name: levien-native-ui-mastery
description: Build native UIs in the style of Raph Levien, architect of Druid, Xilem, and Vello. Emphasizes declarative reactive architecture, synchronized tree transformations, GPU-accelerated rendering, and idiomatic Rust patterns. Use when designing responsive, beautiful native UIs or 2D graphics systems.
tags: gui, native-ui, rendering, gpu, graphics, 2d, vector, canvas, widget, reactive, layout, vello, druid
---

# Raph Levien Style Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍​​​​​​‌​‍‌‌​​‌‌​​‍‌‌‌‌‌​‌​‍​‌‌‌‌‌‌‌‍​​​​‌​‌​‍‌​‌‌​​​‌⁠‍⁠

## Overview

Raph Levien is a Principal Software Engineer at Canva (formerly Google Fonts) and the architect of the Linebender ecosystem: Druid, Xilem, Vello, Piet, and Kurbo. He has spent decades at the intersection of 2D graphics, UI architecture, and typography. His blog "raphlinus.github.io" is the canonical source for modern thinking about native UI in Rust.

## Core Philosophy

> "Architectures that work well in other languages generally don't adapt well to Rust, mostly because they rely on shared mutable state."
>
> "Hidden inside of every UI framework is some kind of incrementalization framework."
>
> "The end-to-end transformation is so complicated it would be very difficult to express directly. So it's best to break it down into smaller chunks, stitched together in a pipeline."

Levien sees UI as a **pipeline of tree transformations**. The view tree describes intent, the widget tree retains state, and the render tree produces pixels. Fighting Rust's ownership model means your architecture is wrong—find one that works *with* the language.

## Design Principles

1. **Declarative Over Imperative**: UI should describe *what*, not *how*. Application logic produces a view tree; the framework handles the rest.

2. **Synchronized Trees**: View tree (ephemeral, typed) → Widget tree (retained, stateful) → Render tree (layout, paint). Each stage has clear responsibilities.

3. **Incremental by Default**: Memoize aggressively. Diff sparsely. Fine-grained change propagation beats wholesale re-rendering.

4. **Statically Typed, Ergonomically Used**: Leverage Rust's type system to catch errors at compile time, but don't burden the developer with excessive annotations.

5. **GPU-First Rendering**: The CPU describes the scene; the GPU does the work. Compute shaders can handle parsing, flattening, and rasterization.

6. **Composition via Adapt Nodes**: Components own a slice of state, not the global state. Adapt nodes translate between parent and child state types.

7. **Accessibility Is Architecture**: Screen reader support requires retained structure and stable identity. This is not an afterthought—it shapes the design.

8. **Performance Is Research**: Willing to solve hard problems (Euler spirals, parallel curves, GPU compute pipelines) rather than accept mediocre solutions.

## When Building UI

### Always

- Model UI as a pipeline of tree transformations
- Use declarative view descriptions that produce typed trees
- Design for incremental updates from the start
- Provide stable identity for widgets (id paths)
- Consider accessibility requirements early—they affect architecture
- Separate view logic (ephemeral) from widget state (retained)
- Route events through the tree with mutable access at each stage

### Never

- Rely on shared mutable state for UI coordination
- Use `Rc<RefCell<T>>` as a first resort—it's a sign of architectural mismatch
- Assume immediate mode can handle complex UI (accessibility, virtualized scroll)
- Create explicit message types for every interaction (Elm-style verbosity)
- Couple rendering tightly to the CPU—GPUs are massively parallel
- Ignore the borrow checker—restructure instead

### Prefer

- View trees over imperative widget construction
- Adapt nodes over global message dispatch
- Id-path event routing over callback spaghetti
- Retained widget trees over pure immediate mode
- GPU compute shaders over CPU rendering loops
- Sparse collection diffing over full re-renders
- Typed erasure escape hatches (`AnyView`) over runtime type chaos

## Architecture Patterns

### The Synchronized Tree Model

```rust
// UI as a pipeline of tree transformations
//
// 1. App produces View tree (ephemeral, describes intent)
// 2. Framework diffs View tree against previous version
// 3. Diff is applied to Widget tree (retained, holds state)
// 4. Widget tree produces Render tree (layout, paint)
// 5. Render tree is drawn to screen (GPU)

// View trait: the core abstraction
trait View {
    type State;           // View-specific state (persists across cycles)
    type Widget;          // Associated widget type
    
    fn build(&self, cx: &mut Cx) -> (Self::State, Self::Widget);
    fn rebuild(&self, cx: &mut Cx, state: &mut Self::State, widget: &mut Self::Widget);
    fn event(&self, state: &mut Self::State, event: &Event) -> EventResult;
}

// The view tree is statically typed—compiler knows the full structure
// State and widget trees are derived via associated types
```

### Adapt Nodes for Composition

```rust
// BAD: Global app state threaded everywhere
struct AppState {
    user: UserState,
    settings: SettingsState,
    // Every component sees everything
}

fn settings_panel(state: &mut AppState) -> impl View {
    // Has access to user state it doesn't need
    // ...
}

// GOOD: Adapt nodes slice state for components
fn app_view(state: &mut AppState) -> impl View {
    VStack::new((
        // Adapt translates between parent and child state
        Adapt::new(
            |state: &mut AppState, thunk| {
                // Child only sees SettingsState
                thunk.call(&mut state.settings)
            },
            settings_panel,  // Receives &mut SettingsState
        ),
        Adapt::new(
            |state: &mut AppState, thunk| thunk.call(&mut state.user),
            user_panel,  // Receives &mut UserState
        ),
    ))
}

// Components are decoupled—they don't know about global state
fn settings_panel(state: &mut SettingsState) -> impl View {
    // Only has access to what it needs
    Toggle::new("Dark Mode", &mut state.dark_mode)
}
```

### Id-Path Event Routing

```rust
// Events are routed via id paths, providing mutable access at each stage
//
// When a button is clicked:
// 1. Event enters at root with full id path: [root, container, button]
// 2. Root receives event, can mutate app state, forwards to container
// 3. Container receives event, can mutate its state, forwards to button
// 4. Button handles the click, mutates its state
// 5. Callbacks fire with mutable access to appropriate state slice

struct IdPath(Vec<Id>);

impl View for Button {
    fn event(&self, state: &mut Self::State, id_path: &IdPath, event: &Event) -> EventResult {
        if id_path.is_empty() && matches!(event, Event::Click) {
            // This event is for us
            (self.on_click)(state);
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    }
}

// Key insight: mutable state access at each level of the tree
// No need for message passing or global dispatch
```

### Memoization for Incremental Updates

```rust
// Fine-grained change propagation: only rebuild what changed

fn item_list(items: &[Item]) -> impl View {
    VStack::new(
        items.iter().map(|item| {
            // Memoize: only rebuild if item changed
            Memoize::new(
                item.id,           // Stable identity
                item.clone(),      // Data to compare
                |item| item_row(item),
            )
        })
    )
}

// The framework tracks:
// - Which items are new (build)
// - Which items changed (rebuild)  
// - Which items are gone (destroy)
// - Which items are unchanged (skip)

// Ron Minsky: "hidden inside of every UI framework 
// is some kind of incrementalization framework"
```

### GPU Scene Description

```rust
// Vello model: CPU describes, GPU renders
//
// The CPU uploads a scene in a simplified binary format
// Compute shaders handle:
// - Parsing the scene graph
// - Path flattening (curves → line segments)
// - Tiling and binning
// - Rasterization
// - Compositing

struct Scene {
    // Scene description: shapes, transforms, clips, blends
    encoding: Vec<u8>,
}

impl Scene {
    fn fill(&mut self, path: &Path, brush: &Brush) {
        // Encode fill command into binary format
        self.encoding.extend(encode_fill(path, brush));
    }
    
    fn stroke(&mut self, path: &Path, style: &Stroke, brush: &Brush) {
        // Stroke is expanded on GPU via Euler spiral approximation
        self.encoding.extend(encode_stroke(path, style, brush));
    }
    
    fn push_transform(&mut self, transform: Affine) {
        self.encoding.extend(encode_transform(transform));
    }
}

// Key insight: the GPU is massively parallel
// Traditional 2D APIs (Cairo, Skia) serialize work on CPU
// Vello parallelizes across thousands of GPU cores
```

### Sparse Collection Diffing

```rust
// Efficient updates for large collections using immutable data structures

use std::sync::Arc;

// Immutable collection with structural sharing
struct ImList<T> {
    root: Option<Arc<Node<T>>>,
}

impl<T: Clone + Eq> ImList<T> {
    fn diff(&self, other: &Self) -> CollectionDiff<T> {
        // O(changed) not O(n) comparison
        // Structural sharing means unchanged subtrees are pointer-equal
        diff_trees(&self.root, &other.root)
    }
}

// In the view layer:
fn list_view(items: &ImList<Item>) -> impl View {
    // Framework diffs against previous items
    // Only changed items trigger widget updates
    VirtualList::new(items, |item| item_row(item))
}

// This solves the UI collection problem:
// - Complex incremental updates → error-prone
// - Full diffing every frame → slow for large collections
// - Immutable + structural sharing → best of both
```

## Mental Model

Levien approaches UI by asking:

1. **What trees are involved?** — View, widget, render, draw—each has a role
2. **How does state flow?** — Props down, events up, through Adapt nodes
3. **Where is the incrementalization?** — What can be memoized? What must be diffed?
4. **Can this be parallelized?** — GPU compute? Multi-threaded reconciliation?
5. **What does the type system encode?** — Compile-time structure vs runtime flexibility
6. **Is accessibility possible?** — Retained structure and stable identity are required

## Raph's Design Questions

When designing UI architecture:

1. Is this declarative? Can app logic just describe what it wants?
2. Where is the retained state? Who owns it?
3. How do events flow back to state? With what granularity of access?
4. What happens when the collection has 10,000 items?
5. Can a screen reader traverse this? Is identity stable?
6. Where is work happening—CPU or GPU? Can it be parallelized?

## Signature Moves

- **Synchronized tree diffing**: View tree is ephemeral, widget tree persists
- **Adapt nodes**: State slicing for component composition
- **Id-path event dispatch**: Mutable access at each tree level
- **GPU scene upload**: CPU describes, GPU renders everything
- **Euler spiral strokes**: Mathematically correct parallel curves
- **Sparse collection diffing**: Immutable structures with structural sharing
- **Type-driven architecture**: Associated types derive state and widget trees
