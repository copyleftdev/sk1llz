# Go Masters

> "Simplicity is complicated." — Rob Pike

## The Pantheon

### The Creators (The Holy Trinity)

- **[Rob Pike](pike/)** — Co-creator of Go, Plan 9, UTF-8
  - *Focus*: Simplicity, concurrency, composition over inheritance

- **[Ken Thompson](thompson/)** — Co-creator of Go, Unix, C
  - *Focus*: Unix philosophy, minimalism, doing one thing well

- **[Robert Griesemer](griesemer/)** — Co-creator of Go, V8 JavaScript engine
  - *Focus*: Clean syntax, specification precision, type system

### The Stewards

- **[Russ Cox](cox/)** — Go tech lead, module system architect
  - *Focus*: Tooling excellence, correctness, backward compatibility

### The Educators

- **[Dave Cheney](cheney/)** — Community leader, "Practical Go"
  - *Focus*: Practical patterns, performance, clear error handling

- **[Bill Kennedy](kennedy/)** — Author of "Go in Action", Ardan Labs
  - *Focus*: Mechanical sympathy, data-oriented design, idioms

## Shared Principles

Go's creators explicitly rejected complexity:

1. **Simplicity**: If a feature isn't essential, leave it out
2. **Composition**: Interfaces and embedding over inheritance
3. **Concurrency**: Goroutines and channels as first-class citizens
4. **Tooling**: `gofmt`, `go vet`, `go test` are non-negotiable
5. **Readability**: Code is read far more than written

## Go Proverbs (Rob Pike)

```
Don't communicate by sharing memory, share memory by communicating.
Concurrency is not parallelism.
Channels orchestrate; mutexes serialize.
The bigger the interface, the weaker the abstraction.
Make the zero value useful.
interface{} says nothing.
Gofmt's style is no one's favorite, yet gofmt is everyone's favorite.
A little copying is better than a little dependency.
Syscall must always be guarded with build tags.
Cgo must always be guarded with build tags.
Cgo is not Go.
With the unsafe package there are no guarantees.
Clear is better than clever.
Reflection is never clear.
Errors are values.
Don't just check errors, handle them gracefully.
Design the architecture, name the components, document the details.
Documentation is for users.
Don't panic.
```

## How to Use These Skills

1. **Start with Pike** for the philosophical foundation
2. **Add Thompson** for Unix-style minimalism
3. **Layer Griesemer** for type system understanding
4. **Apply Cox** for tooling and module design
5. **Use Cheney** for practical patterns
6. **Consult Kennedy** for performance-critical code

## Additional Resources

- [Effective Go](https://go.dev/doc/effective_go)
- [Go Code Review Comments](https://github.com/golang/go/wiki/CodeReviewComments)
- [Go Proverbs](https://go-proverbs.github.io/)
- [GopherCon Talks](https://www.youtube.com/c/GopherAcademy)
