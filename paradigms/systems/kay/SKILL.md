---
name: kay-inventing-the-future
description: Design and build software in the style of Alan Kay, inventor of object-oriented programming, Smalltalk, and the Dynabook concept. Emphasizes message-passing over method-calling, late binding, biological metaphors for system design, and building systems that can evolve. Use when designing extensible architectures, programming environments, or systems meant to outlast their creators.
---

# Alan Kay Style Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​‌‌‌​​​‌‍​​​​‌​​​‍​​​​‌‌‌​‍‌​​‌​‌​‌‍‌‌‌‌​‌​‌‍‌​‌​​​‌‌‍​​​​‌​‌​‍​​​​‌​‌‌⁠‍⁠

## Overview

Alan Kay is a computer scientist who invented Smalltalk, coined the term "object-oriented programming," conceived the Dynabook (decades before the iPad), and led the Xerox PARC team that created the modern GUI. He won the Turing Award in 2003. His work is not about any single technology—it's about how to think about computing itself. Kay sees software as a *medium for human thought*, not a product to be shipped.

## Core Philosophy

> "The best way to predict the future is to invent it."

> "People who are really serious about software should make their own hardware."

> "I invented the term 'object-oriented,' and I can tell you I did not have C++ in mind."

Kay's vision of OOP has almost nothing to do with what most programmers call OOP. His objects are not bags of data with methods. They are autonomous computational agents that communicate through messages—closer to biological cells or networked computers than to C++ classes. The key idea is *late binding*: defer decisions as long as possible so the system can adapt.

## Design Principles

1. **Objects Are Computers in Miniature**: Each object is a self-contained, autonomous entity with its own state and behavior. Objects communicate exclusively through messages. They do not reach into each other's internals.

2. **Messaging Is the Fundamental Mechanism**: The power is not in the objects—it's in the messages between them. Messages can be intercepted, redirected, logged, serialized, delayed. This is where all the flexibility lives.

3. **Late Binding Over Early Binding**: Every decision that can be deferred should be deferred. Compile time < load time < run time < user interaction time. The later a binding happens, the more powerful and adaptable the system.

4. **Systems Should Be Biological, Not Mechanical**: Good systems grow and heal like organisms, not break like machines. Design for evolution, not perfection. Components should be replaceable at runtime.

5. **The Medium Is the Message**: Programming environments should amplify human thought. If your tools constrain what you can think, you need better tools—build them.

## When Writing Code

### Always

- Design components that communicate through well-defined messages
- Make objects autonomous—they decide how to respond to a message
- Defer binding decisions as late as possible
- Build systems that can be modified while running
- Think in terms of protocols and message patterns, not class hierarchies
- Design for the next programmer to extend, not just use
- Consider whether you're building a tool or building a medium

### Never

- Expose internal state to other objects (getters/setters are a design smell)
- Inherit for code reuse—compose through delegation
- Hardcode decisions that could be made at runtime
- Build systems that require stopping to change
- Confuse inheritance hierarchies with object-oriented design
- Let the language limit your thinking about what's possible

### Prefer

- Message-passing over method-calling
- Delegation over inheritance
- Protocols over types
- Live systems over compile-restart cycles
- Simulations over specifications
- Environments over applications

## Code Patterns

### Objects as Message Processors

```smalltalk
"Kay's OOP: objects respond to messages, not method calls.
 The object decides what to do with the message."

"In Smalltalk, everything is a message send:"
3 + 4           "Send message '+' with argument 4 to object 3"
'hello' size    "Send message 'size' to string 'hello'"
array at: 5     "Send message 'at:' with argument 5 to array"

"Even control flow is messages:"
x > 0
    ifTrue: [self doPositive]
    ifFalse: [self doNegative]
"This sends 'ifTrue:ifFalse:' to a Boolean object"

"Even class creation is a message:"
Object subclass: #Animal
    instanceVariableNames: 'name'
    classVariableNames: ''
    poolDictionaries: ''
```

### Message-Oriented Design (In Any Language)

```python
# BAD: Objects reaching into each other's state
class OrderProcessor:
    def process(self, order):
        if order.status == "pending":     # Knows order internals
            order.status = "processing"   # Mutates foreign state
            for item in order.items:      # Knows structure
                warehouse.stock[item.sku] -= item.qty  # Deep coupling

# GOOD: Kay-style message passing
class Order:
    def request_processing(self):
        """Order decides how to process itself."""
        if self._can_process():
            self._status = "processing"
            return ProcessingStarted(self._id, self._items)
        return ProcessingDenied(self._id, self._reason)

class Warehouse:
    def handle(self, message):
        """Warehouse decides how to respond to messages."""
        if isinstance(message, ProcessingStarted):
            return self._attempt_reservation(message.items)
        # Warehouse controls its own internals

# The power: you can now intercept, log, retry, redirect,
# serialize, or replay these messages. Try doing that with
# direct method calls and shared mutable state.
```

### Late Binding

```python
# EARLY BINDING: Decision locked at compile/write time
def save_user(user):
    db = PostgresDatabase("localhost:5432")  # Locked in
    db.insert("users", user.to_dict())       # Locked in

# LATE BINDING: Decision deferred to runtime
def save_user(user, storage=None):
    storage = storage or resolve_storage()  # Decided at runtime
    storage.save("users", user)             # Protocol, not implementation

# LATER BINDING: Decision deferred to configuration
# storage.yml:
#   backend: postgres
#   host: localhost
#   port: 5432

# LATEST BINDING: Decision made by the user at interaction time
# User drags "User" object to "S3 Bucket" icon in a live environment
# The system adapts without recompilation

# Kay's insight: each level of late binding gives
# exponentially more flexibility and power.
```

### Biological Design

```python
# MECHANICAL: One failure kills the system
class Pipeline:
    def run(self, data):
        a = self.step_a(data)    # If this dies, everything dies
        b = self.step_b(a)       # Tightly coupled chain
        c = self.step_c(b)
        return c

# BIOLOGICAL: Components are autonomous, system heals
class Cell:
    """Each cell is autonomous and communicates through messages."""

    def __init__(self, name, handler):
        self.name = name
        self.handler = handler
        self.inbox = queue.Queue()
        self.alive = True

    def receive(self, message):
        self.inbox.put(message)

    def run(self):
        while self.alive:
            try:
                msg = self.inbox.get(timeout=1)
                response = self.handler(msg)
                if response:
                    msg.reply_to.receive(response)
            except queue.Empty:
                continue
            except Exception as e:
                # Cell heals itself, doesn't crash the organism
                self.log_error(e)
                continue

class Organism:
    """The system is a colony of cells.
    Cells can be replaced, added, or removed at runtime."""

    def __init__(self):
        self.cells = {}

    def add_cell(self, cell):
        self.cells[cell.name] = cell
        threading.Thread(target=cell.run, daemon=True).start()

    def replace_cell(self, name, new_cell):
        """Hot-swap a component without stopping the system."""
        old = self.cells[name]
        old.alive = False
        self.add_cell(new_cell)
```

### The Environment Is the Application

```python
# Kay doesn't think in "applications." He thinks in "environments"
# where users and objects interact dynamically.

# APPLICATION THINKING:
#   "Build a todo app with these features"
#   Fixed functionality, fixed UI, ship and done

# ENVIRONMENT THINKING:
#   "Build a world where task objects exist and can be
#    composed, scripted, shared, and extended by users"

# In an environment:
# - Users can create new kinds of objects at runtime
# - Objects can be inspected, modified, composed
# - The system is never "done"—it's always being extended
# - Programming and using are on a continuum, not separate activities

# This is why Smalltalk has a live image, not compiled binaries.
# This is why the Dynabook was envisioned as a medium, not a device.
```

## The Kay Litmus Tests

### Test 1: Can You Replace a Component While Running?

If you have to stop the system to change a component, your binding is too early. In Kay's vision, every component should be hot-swappable.

### Test 2: Can You Add Behavior Without Modifying Existing Code?

If adding a new feature requires changing existing objects, your messaging protocol is too rigid. New behavior should arrive as new message handlers, not code modifications.

### Test 3: Can You Explain the System Without Implementation Details?

If you can't describe the system purely in terms of objects and the messages they exchange—without mentioning databases, frameworks, or languages—your design is coupled to implementation.

### Test 4: Could a Child Extend This?

Kay's Dynabook was designed for children. If your system requires expert knowledge to extend, you've built a tool, not a medium. The question is not whether the code is simple—it's whether the *model* is simple.

## The Internet as Kay's OOP

Kay often points to the Internet as the best example of his OOP vision:

- **Each node is autonomous** (like an object)
- **Communication is via messages** (packets, HTTP requests)
- **Late binding everywhere** (DNS, content negotiation, routing)
- **No central control** (no God object, no main loop)
- **Components can be replaced without stopping the network**
- **It scales to billions of nodes**

If your software architecture doesn't have these properties, ask yourself why the Internet does and your system doesn't.

## Mental Model

Kay approaches every design by asking:

1. **What are the objects?** Not data structures—autonomous agents.
2. **What messages do they exchange?** This defines the system.
3. **What can be deferred?** Bind as late as possible.
4. **Can this evolve?** If not, it will die.
5. **Is this a tool or a medium?** Build media, not applications.

## Signature Kay Moves

- Designing systems as colonies of autonomous message-passing agents
- Making everything inspectable and modifiable at runtime
- Deferring every decision to the latest possible moment
- Building programming environments, not just programs
- Thinking in decades, not sprints
- Treating objects as tiny computers, not data containers
- Asking "What would this look like if we invented it today?"
