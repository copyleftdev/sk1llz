# Rob Pike Philosophy

## Simplicity Above All

Pike's core principle: Simplicity is the ultimate sophistication.

> "Simplicity is complicated."

### The Go Proverbs

Pike's Go proverbs distill decades of systems programming wisdom:

1. **Don't communicate by sharing memory; share memory by communicating.**
2. **Concurrency is not parallelism.**
3. **Channels orchestrate; mutexes serialize.**
4. **The bigger the interface, the weaker the abstraction.**
5. **Make the zero value useful.**
6. **interface{} says nothing.**
7. **Gofmt's style is no one's favorite, yet gofmt is everyone's favorite.**
8. **A little copying is better than a little dependency.**
9. **Syscall must always be guarded with build tags.**
10. **Cgo must always be guarded with build tags.**
11. **Cgo is not Go.**
12. **With the unsafe package there are no guarantees.**
13. **Clear is better than clever.**
14. **Reflection is never clear.**
15. **Errors are values.**
16. **Don't just check errors, handle them gracefully.**
17. **Design the architecture, name the components, document the details.**
18. **Documentation is for users.**
19. **Don't panic.**

## Concurrency Philosophy

### Communicating Sequential Processes (CSP)

Pike brought Tony Hoare's CSP model to Go:

```go
// Don't do this (sharing memory)
var counter int
var mu sync.Mutex

func increment() {
    mu.Lock()
    counter++
    mu.Unlock()
}

// Do this (communicating)
func counter(ch chan int) {
    count := 0
    for delta := range ch {
        count += delta
    }
}
```

### Concurrency vs Parallelism

> "Concurrency is about dealing with lots of things at once. Parallelism is about doing lots of things at once."

```go
// Concurrency: Structure
func pipeline(in <-chan int) <-chan int {
    out := make(chan int)
    go func() {
        for n := range in {
            out <- process(n)
        }
        close(out)
    }()
    return out
}

// Parallelism: Execution
func parallel(items []int, workers int) {
    var wg sync.WaitGroup
    ch := make(chan int)
    
    for i := 0; i < workers; i++ {
        wg.Add(1)
        go func() {
            defer wg.Done()
            for item := range ch {
                process(item)
            }
        }()
    }
    
    for _, item := range items {
        ch <- item
    }
    close(ch)
    wg.Wait()
}
```

## Interface Philosophy

### Small Interfaces

> "The bigger the interface, the weaker the abstraction."

```go
// Good: Small, focused interfaces
type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}

// Compose when needed
type ReadWriter interface {
    Reader
    Writer
}
```

### Accept Interfaces, Return Structs

```go
// Good: Accept interface
func Process(r io.Reader) error {
    // Can accept any Reader
}

// Good: Return concrete type
func NewBuffer() *Buffer {
    return &Buffer{}
}
```

## Error Handling

### Errors Are Values

```go
// Errors are just values - use them
type ParseError struct {
    Line int
    Col  int
    Msg  string
}

func (e *ParseError) Error() string {
    return fmt.Sprintf("%d:%d: %s", e.Line, e.Col, e.Msg)
}

// Handle errors, don't just check them
func readConfig(path string) (*Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("reading config %s: %w", path, err)
    }
    
    var cfg Config
    if err := json.Unmarshal(data, &cfg); err != nil {
        return nil, fmt.Errorf("parsing config: %w", err)
    }
    
    return &cfg, nil
}
```

### Don't Panic

```go
// Panic for truly unrecoverable situations
func MustCompile(pattern string) *Regexp {
    re, err := Compile(pattern)
    if err != nil {
        panic(err)
    }
    return re
}

// But prefer returning errors
func Compile(pattern string) (*Regexp, error) {
    // ...
}
```

## Zero Value Philosophy

### Make Zero Values Useful

```go
// Good: Zero value is ready to use
var buf bytes.Buffer
buf.WriteString("hello")  // Works immediately

var mu sync.Mutex
mu.Lock()  // Works without initialization

// Bad: Requires initialization
type BadBuffer struct {
    data []byte
}

func (b *BadBuffer) Write(p []byte) {
    if b.data == nil {
        b.data = make([]byte, 0, 1024)  // Extra work needed
    }
    b.data = append(b.data, p...)
}
```

## Naming Philosophy

### Short Names for Short Scopes

```go
// Loop variables: single letter is fine
for i := 0; i < len(items); i++ {
    process(items[i])
}

for _, v := range values {
    fmt.Println(v)
}

// Parameters: context determines clarity
func (s *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
    // w and r are conventional, clear in context
}

// Package-level: be descriptive
var DefaultClient = &http.Client{Timeout: 30 * time.Second}
```

### MixedCaps, Not Underscores

```go
// Good
func parseJSON(data []byte) error
type HTTPServer struct{}
var maxRetryCount = 3

// Bad
func parse_json(data []byte) error
type HTTP_Server struct{}
var max_retry_count = 3
```

## Code Organization

### Package Design

> "A little copying is better than a little dependency."

```go
// Avoid:
import "github.com/someone/tiny-helper"

// Prefer copying 10 lines of code if:
// - The dependency is small
// - It might change
// - It adds significant compile time
```

### Keep Packages Focused

```go
// Good: focused packages
package json    // Just JSON encoding/decoding
package http    // Just HTTP
package strings // Just string operations

// Bad: kitchen sink packages
package util    // What's in here?
package common  // Everything?
package misc    // Who knows?
```

## Famous Quotes

> "Complexity is multiplicative."

> "The purpose of abstraction is not to be vague, but to create a new semantic level in which one can be absolutely precise."

> "Data dominates. If you've chosen the right data structures and organized things well, the algorithms will almost always be self-evident."

> "Rule of Repair: When you must fail, fail noisily and as soon as possible."

## Influences on Go

Pike brought together influences from:
- **C** - Simplicity, efficiency
- **CSP** - Concurrency model
- **Limbo/Newsqueak** - Channels, goroutines
- **Unix** - Composition, text streams
- **Plan 9** - Modern systems thinking
