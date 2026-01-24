# Ken Thompson Philosophy

## The Unix Philosophy (Original)

From the original Unix documentation and Thompson's writings:

### Rule of Modularity
> Write simple parts connected by clean interfaces.

Programs should do one thing well. When a program needs to be extended, write a new program rather than complicate the old one.

### Rule of Clarity
> Clarity is better than cleverness.

Write programs as if the most important communication they do is not to the computer that executes them but to the human beings who will read and maintain the source code.

### Rule of Composition
> Design programs to be connected to other programs.

Expect the output of every program to become the input to another, as yet unknown, program.

### Rule of Separation
> Separate policy from mechanism; separate interfaces from engines.

### Rule of Simplicity
> Design for simplicity; add complexity only where you must.

## Thompson on Design

### On Brute Force
> "When in doubt, use brute force."

Thompson's famous advice acknowledges that clever algorithms often aren't worth their complexity. A simple O(n²) algorithm that's obviously correct beats a complex O(n log n) algorithm with subtle bugs.

### On Code Deletion
> "One of my most productive days was throwing away 1000 lines of code."

The courage to delete code is essential. Code that doesn't serve the current goal is liability, not asset.

### On Trust
> "You can't trust code that you did not totally create yourself."

From his Turing Award lecture "Reflections on Trusting Trust" - a meditation on how deeply we must trust our tools.

## The Evolution of Thompson's Thinking

### 1970s: Unix Creation
- Radical simplicity
- Everything is a file
- Text as universal interface
- Small, sharp tools

### 1980s: Plan 9
- Took Unix ideas further
- Everything really is a file (including network, graphics)
- Per-process namespaces
- UTF-8 encoding (with Rob Pike)

### 2000s-2010s: Go Language
- Simplicity over features
- Fast compilation
- Built-in concurrency (goroutines, channels)
- No inheritance, no generics (initially)
- Explicit error handling

## Key Quotes Collection

On complexity:
> "The key to performance is elegance, not battalions of special cases."

On debugging:
> "Debugging is twice as hard as writing the code in the first place."

On features:
> "When you feel the urge to design a complex binary file format, or a complex binary application protocol, it is generally wise to lie down until the feeling passes."

On design:
> "A little copying is better than a little dependency."

## Recommended Reading

1. **"The Unix Programming Environment"** (1984) - Kernighan & Pike
2. **"Reflections on Trusting Trust"** - Turing Award Lecture
3. **"Plan 9 from Bell Labs"** - Plan 9 papers
4. **"The Go Programming Language"** (2015) - Donovan & Kernighan
