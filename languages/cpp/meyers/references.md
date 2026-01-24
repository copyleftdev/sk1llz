# Scott Meyers References

## Books

### The "Effective" Trilogy

1. **"Effective C++" (3rd Edition, 2005)**
   - 55 specific ways to improve programs and designs
   - Foundational C++ wisdom
   - Start here for classic C++

2. **"More Effective C++" (1996)**
   - 35 additional items
   - Deeper into advanced topics
   - Exceptions, efficiency, techniques

3. **"Effective STL" (2001)**
   - 50 specific ways to improve STL usage
   - Container selection, algorithms, iterators
   - Essential for anyone using the standard library

### Modern C++

4. **"Effective Modern C++" (2014)**
   - 42 items for C++11/14
   - Move semantics, smart pointers, lambdas
   - The modern C++ bible

## Talks

### Essential Viewing

1. **"Effective C++ in an Embedded Environment" (2012)**
   - C++ in resource-constrained systems
   - What still applies, what changes

2. **"The Last Thing D Needs" (DConf 2014)**
   - Interesting meta-perspective on language design
   - Why some C++ decisions were made

3. **"CPU Caches and Why You Care" (2014)**
   - Performance beyond algorithms
   - Hardware-aware programming

4. **"Type Deduction and Why You Care" (CppCon 2014)**
   - `auto`, `decltype`, template deduction
   - Modern C++ type system mastery

## Key Items to Internalize

### From Effective C++

- **Item 1**: View C++ as a federation of languages
- **Item 3**: Use `const` wherever possible
- **Item 4**: Make sure objects are initialized before use
- **Item 13**: Use objects to manage resources (RAII)
- **Item 18**: Make interfaces easy to use correctly, hard to use incorrectly

### From Effective Modern C++

- **Item 1**: Understand template type deduction
- **Item 5**: Prefer `auto` to explicit type declarations
- **Item 7**: Distinguish `()` and `{}` when creating objects
- **Item 21**: Prefer `std::make_unique` and `std::make_shared`
- **Item 41**: Consider pass by value for copyable parameters that are cheap to move

## Blog & Website

- **[aristeia.com](http://aristeia.com/)**
  - Scott's personal site
  - Articles, errata, presentations
  - Book information and updates

## Style Notes

Meyers' distinctive approach:
- Numbered items make reference easy
- Each item has clear rationale
- Exceptions and caveats explicitly noted
- "Consider" vs "Always" distinction
