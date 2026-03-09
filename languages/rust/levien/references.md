# Raph Levien References⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍‌‌‌​‌‌‌‌‍‌​‌‌​​‌‌‍​​‌‌‌​​​‍‌​​‌​‌​‌‍​​​​‌​‌​‍‌‌​‌‌​​‌⁠‍⁠

## Primary Sources

### Blog (Canonical)

- **raphlinus.github.io** — The authoritative source for Levien's thinking
  - [Xilem: an architecture for UI in Rust](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html) — The Xilem architecture proposal
  - [Towards a unified theory of reactive UI](https://raphlinus.github.io/ui/druid/2019/11/22/reactive-ui.html) — Tree transformation model
  - [Towards principled reactive UI](https://raphlinus.github.io/rust/druid/2020/09/25/principled-reactive-ui.html) — Crochet experiment and learnings
  - [Advice for the next dozen Rust GUIs](https://raphlinus.github.io/rust/gui/2022/07/15/next-dozen-guis.html) — Ecosystem survey and recommendations
  - [Raph's reflections and wishes for 2023](https://raphlinus.github.io/personal/2022/12/31/raph-2023.html) — Project priorities and philosophy

### Talks

- **Xilem: Let's Build High Performance Rust UI** (RustLab 2022) — Architecture deep dive
- **High Performance Rust UI** — Goals, motivations, and Q&A
- **RustLab 2020**: Immutable data structures for UI — Sparse diffing for collections

### Repositories

| Repository | Description |
|------------|-------------|
| [linebender/xilem](https://github.com/linebender/xilem) | Next-gen reactive UI framework |
| [linebender/vello](https://github.com/linebender/vello) | GPU-accelerated 2D renderer |
| [linebender/druid](https://github.com/linebender/druid) | Current-gen UI toolkit (research) |
| [linebender/kurbo](https://github.com/linebender/kurbo) | 2D geometry primitives |
| [linebender/piet](https://github.com/linebender/piet) | 2D graphics abstraction trait |
| [AccessKit/accesskit](https://github.com/AccessKit/accesskit) | Cross-platform accessibility |

### Community

- **Zulip**: [xi.zulipchat.com](https://xi.zulipchat.com/) — Primary discussion venue
- **Mastodon**: [@raph@mastodon.online](https://mastodon.online/@raph)
- **GitHub**: [raphlinus](https://github.com/raphlinus)
- **Weekly Office Hours**: Check Zulip for schedule

## Key Concepts by Source

### Xilem Architecture (2022 blog post)

- View trees as typed, ephemeral descriptions
- Widget trees as retained, stateful implementations
- Adapt nodes for component composition
- Id-path event routing with mutable state access
- Memoization for incremental updates

### Unified Theory of Reactive UI (2019 blog post)

- UI as a pipeline of tree transformations
- Trees as data structures vs. traces of execution
- Push vs. pull interfaces
- Incremental transformations and diffing

### Principled Reactive UI (2020 blog post)

- Observable objects vs. future-like polling
- Tree mutation patterns
- Stable identity in view trees
- The Crochet experiment (negative result)

### Advice for Rust GUIs (2022 blog post)

- Tradeoff space analysis (native feel, platform integration, GPU rendering)
- Winit as windowing layer
- Accessibility requirements (AccessKit)
- Architecture must evolve—no One True Solution yet

## Related Reading

### On GPU Rendering

- [I want a good parallel computer](https://raphlinus.github.io/gpu/2025/03/21/good-parallel-computer.html) — GPU compute philosophy
- [Parallel curves of cubic Béziers](https://raphlinus.github.io/curves/2022/09/09/parallel-beziers.html) — Path geometry

### On Curves and Typography

- Levien's PhD thesis on spline curve fitting
- Euler spiral (cornu spiral) applications to stroke expansion
- Font rendering: stem darkening, hinting, variable fonts

### Influences

- **SwiftUI**: Typed view trees, declarative syntax
- **Flutter**: Widget tree, render object tree separation
- **React**: Virtual DOM diffing, component model
- **Elm**: Unidirectional data flow (but not message passing)
- **FRP**: Functional reactive programming foundations

## Ecosystem Alternatives

Levien acknowledges these as viable alternatives while Xilem matures:

| Toolkit    | Architecture    | Notes                                      |
|------------|-----------------|--------------------------------------------|
| **egui**   | Immediate mode  | Simple, pragmatic, popular                 |
| **Iced**   | Elm-inspired    | Clean, well-designed                       |
| **Slint**  | Declarative DSL | Commercial, polished                       |
| **Dioxus** | React-like      | Interior mutability, credible alternative  |

## How to Learn More

1. **Start with the blog**: Read the Xilem and Unified Theory posts
2. **Watch the talks**: RustLab presentations give visual explanations
3. **Join Zulip**: Ask questions, see discussions
4. **Read the code**: Xilem repo has examples and documentation
5. **Contribute**: Many tasks don't require "rocket science"
