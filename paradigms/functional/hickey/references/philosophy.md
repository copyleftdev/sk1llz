# Rich Hickey Philosophy

## Simple Made Easy

The cornerstone of Hickey's philosophy, from his 2011 Strange Loop talk.

### Simple vs Easy

| Simple | Easy |
|--------|------|
| One fold/braid | Near at hand |
| One role | Familiar |
| One concept | Near our capabilities |
| Objective | Relative |

**Simple** is about lack of interleaving, not about familiarity.
**Easy** is relative to the observer—what's easy for you may not be easy for me.

### Complect

Hickey's key term: "to interleave, entwine, braid together"

Things that complect:
- State and identity
- Value and time
- Syntax and semantics
- Objects: state + identity + value

### The Simplicity Toolkit

| Complex | Simple |
|---------|--------|
| State, objects | Values |
| Methods | Functions, namespaces |
| Variables | Managed refs |
| Inheritance | Polymorphism à la carte |
| Locks, mutexes | Queues |
| ORM | Declarative data manipulation |

## The Value of Values

Values are immutable. This single property enables:

1. **Sharing without coordination**
2. **Reproducibility**
3. **Testing simplicity**
4. **Caching**
5. **Structural sharing**

```clojure
;; Values can be freely shared
(def person {:name "Alice" :age 30})

;; "Change" creates new value
(def older-person (update person :age inc))

;; Original unchanged
person  ; => {:name "Alice" :age 30}
```

## Programming with Data

### Code is Data

Homoiconicity: code has the same structure as data.

```clojure
;; This is both code AND data
(defn greet [name]
  (str "Hello, " name))

;; Can manipulate code as data
(def code '(+ 1 2 3))
(eval code)  ; => 6
```

### Data > Objects

Prefer plain data over objects with behavior:

```clojure
;; Bad: Hiding data in objects
(defprotocol Person
  (get-name [this])
  (get-age [this]))

;; Good: Plain data
{:name "Alice" :age 30}
```

### Schema at the Edges

Validate data at system boundaries, not everywhere:

```
[External World] → Validate → [Internal: Plain Data] → Validate → [External World]
```

## Decomplecting State

### The Problem with State

State complects:
- Value (what)
- Identity (who)
- Time (when)

### Hickey's Solution: Managed References

```clojure
;; Atom: Uncoordinated, synchronous
(def counter (atom 0))
(swap! counter inc)

;; Ref: Coordinated, synchronous (STM)
(def account-a (ref 100))
(def account-b (ref 0))
(dosync
  (alter account-a - 50)
  (alter account-b + 50))

;; Agent: Uncoordinated, asynchronous
(def logger (agent []))
(send logger conj "message")
```

## Polymorphism Without Inheritance

### Protocols

Type-based dispatch without inheritance hierarchy:

```clojure
(defprotocol Speakable
  (speak [this]))

(extend-type String
  Speakable
  (speak [s] (str "String says: " s)))

(extend-type Long
  Speakable
  (speak [n] (str "Number says: " n)))
```

### Multimethods

Arbitrary dispatch:

```clojure
(defmulti area :shape)

(defmethod area :circle [{:keys [radius]}]
  (* Math/PI radius radius))

(defmethod area :rectangle [{:keys [width height]}]
  (* width height))
```

## Key Talks

1. **"Simple Made Easy"** (Strange Loop 2011)
   - The foundational talk on simplicity
   
2. **"The Value of Values"** (JaxConf 2012)
   - Why immutability matters

3. **"Are We There Yet?"** (JVM Summit 2009)
   - Time, identity, and state

4. **"Hammock Driven Development"** (Clojure/conj 2010)
   - The importance of thinking before coding

5. **"The Language of the System"** (2013)
   - System-level design principles

## Quotes

> "Simplicity is a prerequisite for reliability."

> "Programmers know the benefits of everything and the tradeoffs of nothing."

> "State is never simple. State complects value and time."

> "If you want everything to be familiar, you will never learn anything new."

> "Information is simple. Don't ruin it."

> "Simplicity is the ultimate sophistication." (quoting da Vinci)
