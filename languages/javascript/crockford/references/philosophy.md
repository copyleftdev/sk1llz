# Douglas Crockford Philosophy

## The Good Parts

Crockford's central insight: JavaScript has good parts buried under bad parts. Use the good, avoid the bad.

> "JavaScript is the only language that people feel they don't need to learn before using."

### The Good

- First-class functions
- Dynamic objects
- Loose typing (when used well)
- Object literal notation
- Prototypal inheritance

### The Bad (Avoid)

- Global variables
- `==` (use `===` instead)
- `with` statement
- `eval()`
- `new` keyword (controversial)
- `++` and `--` (controversial)

## Code Quality Principles

### JSLint: The Tool

Crockford created JSLint to enforce good practices:

> "JSLint will hurt your feelings."

Key JSLint rules:
- Always use `===` and `!==`
- Declare all variables at top of function
- Always use braces for blocks
- No trailing commas
- Use strict mode

### The Principle of Least Authority (POLA)

Give code only the authority it needs:

```javascript
// Bad: Too much access
function process(data, db, logger, config) {
    // Has access to everything
}

// Good: Minimal access
function process(data, saveResult) {
    // Only knows how to process and save
    const result = transform(data);
    saveResult(result);
}
```

## Object Creation Patterns

### The Module Pattern

Crockford's preferred pattern for encapsulation:

```javascript
const counter = (function () {
    let count = 0;  // Private
    
    return {
        increment: function () {
            count += 1;
            return count;
        },
        decrement: function () {
            count -= 1;
            return count;
        },
        value: function () {
            return count;
        }
    };
}());

counter.increment();  // 1
counter.value();      // 1
// count is inaccessible
```

### Power Constructor

Creating objects without `new`:

```javascript
function constructor(spec) {
    const that = {};  // or Object.create(prototype)
    
    // Private state
    let value = spec.value || 0;
    
    // Public methods
    that.getValue = function () {
        return value;
    };
    
    that.setValue = function (v) {
        value = v;
    };
    
    return that;
}

const obj = constructor({ value: 42 });
```

## JSON

Crockford discovered/popularized JSON (JavaScript Object Notation):

> "I discovered JSON. I do not claim to have invented it, because it already existed in nature."

### Why JSON Won

- Simpler than XML
- Native to JavaScript
- Human readable
- Easy to parse
- Minimal syntax

```json
{
    "name": "Douglas Crockford",
    "role": "JSON Discoverer",
    "languages": ["JavaScript", "JSON"]
}
```

## Security Philosophy

### The Principle of Least Capability

Crockford advocates for capability-based security:

```javascript
// Bad: Ambient authority
function sendEmail(to, body) {
    emailService.send(to, body);  // Where did emailService come from?
}

// Good: Explicit capability
function makeSender(emailService) {
    return function sendEmail(to, body) {
        emailService.send(to, body);
    };
}
```

### ADsafe

Crockford's safe JavaScript subset for advertising:
- No `this` keyword
- No `eval()`
- No global access
- Widget sandbox

## Key Talks and Books

1. **"JavaScript: The Good Parts"** (Book, 2008)
   - The definitive guide to good JavaScript

2. **"Crockford on JavaScript"** (Video series)
   - Complete history and deep dive

3. **"The JSON Saga"** (Talk)
   - How JSON came to be

## Famous Quotes

> "JavaScript is the world's most misunderstood programming language."

> "The good parts are so good that they make up for the bad parts."

> "Always code as if the guy who ends up maintaining your code will be a violent psychopath who knows where you live."

> "There are two ways of constructing a software design: One way is to make it so simple that there are obviously no deficiencies, and the other way is to make it so complicated that there are no obvious deficiencies."

> "In JavaScript, there is a beautiful, elegant, highly expressive language that is buried under a steaming pile of good intentions and blunders."

## Evolution of Thinking

### Early Crockford (2000s)
- Strict avoidance of `new`
- Module pattern everywhere
- JSLint strictness

### Later Crockford (2010s+)
- Embraced ES6 classes cautiously
- Arrow functions accepted
- Still skeptical of `this`

## The Crockford Conventions

1. Always use `'use strict';`
2. Declare variables at function top
3. One `var` statement per function
4. Use `===` and `!==` exclusively
5. Avoid `eval()` always
6. Avoid `with` always
7. Use `{}` for all blocks
8. Put `{` on same line as control statement
