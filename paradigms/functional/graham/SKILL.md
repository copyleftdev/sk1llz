---
name: graham-hackers-painters
description: Write code in the style of Paul Graham, essayist, Lisp advocate, and co-founder of Y Combinator. Emphasizes bottom-up programming, expressive power, rapid prototyping, and treating code as a creative medium. Use when building exploratory software, designing DSLs, or writing Lisp that should be as dense and powerful as prose.
tags: lisp, macros, bottom-up, brevity, abstraction, startups, essays, language-design, metaprogramming
---

# Paul Graham Style Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​‌‌​‌‌‌‌‍​‌‌‌​​​​‍​​​​‌‌‌​‍‌​​‌​‌​‌‍‌‌‌‌​‌​‌‍‌​‌​​​‌‌‍​​​​‌​‌​‍​‌‌‌​​​‌⁠‍⁠

## Overview

Paul Graham is a programmer, essayist, and co-founder of Y Combinator. He created Viaweb (the first web-based application, sold to Yahoo), designed the Arc programming language, and wrote *Hackers & Painters*—a collection of essays arguing that programming is a creative art closer to painting than engineering. His work on Bayesian spam filtering, Lisp advocacy, and startup philosophy has shaped an entire generation of builders.

## Core Philosophy

> "A programming language is for thinking of programs, not for expressing programs you've already thought of."

> "The best writing is rewriting. The best code is refactoring."

> "When you're working on something and you have the feeling you're onto something big, you probably are."

Graham sees programming as a medium for thought. The best programs, like the best essays, emerge through exploration—not from specifications handed down from above. The language you think in determines what you can think, so choose the most powerful one available.

## Design Principles

1. **Bottom-Up Programming**: Build languages toward your problem, not programs down from your specification. Write the language you wish you had, then write your program in it.

2. **Brevity Is Power**: The measure of a language is how short your programs are. Shorter programs mean fewer bugs, faster iteration, and clearer thinking. Succinctness is not a nice-to-have—it is power.

3. **Explore, Don't Specify**: Great software is discovered, not planned. Start with something small that works, then grow it. Keep the feedback loop as tight as possible.

4. **Worse Is Better (Strategically)**: Ship the simpler thing. A working prototype that captures the core idea beats a perfect specification that was never built.

5. **Hold Onto the Main Thing**: In any system, there is one idea that matters most. Find it, protect it, and do not let secondary concerns dilute it.

## When Writing Code

### Always

- Write code bottom-up—build abstractions first, then compose
- Use macros to eliminate repetition and create expressive DSLs
- Keep programs short; count lines as a cost metric
- Prototype rapidly; iterate toward quality
- Treat refactoring as the primary creative act
- Use the most powerful language available for the task
- Think in transformations on data, not in object hierarchies

### Never

- Over-plan before coding—discover the design by building
- Write Java-style boilerplate when a macro or higher-order function suffices
- Add abstraction layers you don't need yet
- Mistake verbosity for clarity
- Ignore the feeling that something could be shorter
- Treat programming as a manufacturing process

### Prefer

- Lisp (or the most expressive language available) over mainstream defaults
- Macros over copy-paste patterns
- Closures over objects
- Lists and trees over complex type hierarchies
- Interactive development (REPL) over compile-run-debug cycles
- Small teams of great programmers over large teams of average ones

## Code Patterns

### Bottom-Up Programming

```lisp
;; BAD: Top-down, verbose, one-off
(defun process-orders (orders)
  (let ((results nil))
    (dolist (order orders)
      (when (> (order-total order) 100)
        (let ((discount (calculate-discount order)))
          (push (apply-discount order discount) results))))
    (nreverse results)))

;; GOOD: Build up the language first
(defun filter-map (pred fn lst)
  "Filter by pred, then apply fn to survivors."
  (mapcar fn (remove-if-not pred lst)))

;; Now the program reads like prose
(defun process-orders (orders)
  (filter-map
    (lambda (o) (> (order-total o) 100))
    (lambda (o) (apply-discount o (calculate-discount o)))
    orders))
```

### Macros as Language Extensions

```lisp
;; You find yourself writing this pattern repeatedly:
(let ((start (get-internal-real-time)))
  (progn
    (do-something)
    (do-something-else))
  (format t "Elapsed: ~Dms~%" (- (get-internal-real-time) start)))

;; Graham says: if you see a pattern, write a macro
(defmacro with-timing (label &body body)
  (let ((start (gensym)))
    `(let ((,start (get-internal-real-time)))
       (prog1 (progn ,@body)
         (format t "~A: ~Dms~%" ,label
                 (- (get-internal-real-time) ,start))))))

;; Now timing is part of your language
(with-timing "order processing"
  (process-orders *pending*))
```

### Closures Over Objects

```lisp
;; BAD: Object-oriented counter
(defclass counter ()
  ((value :initform 0 :accessor counter-value)))
(defmethod increment ((c counter))
  (incf (counter-value c)))
(defmethod get-count ((c counter))
  (counter-value c))

;; GOOD: A closure is simpler and more powerful
(defun make-counter (&optional (start 0))
  (let ((n start))
    (lambda (msg)
      (case msg
        (:inc  (incf n))
        (:dec  (decf n))
        (:val  n)
        (:reset (setf n start))))))

(let ((c (make-counter)))
  (funcall c :inc)
  (funcall c :inc)
  (funcall c :val))  ;; => 2
```

### Accumulator Generators (The Graham Litmus Test)

```lisp
;; Graham uses this as a test of language power:
;; "Write a function that generates accumulators—
;;  a function that takes a number n, and returns
;;  a function that takes another number i and
;;  returns n incremented by i."

;; In Lisp, it's trivial:
(defun accumulator (n)
  (lambda (i) (incf n i)))

(let ((acc (accumulator 5)))
  (funcall acc 10)   ;; => 15
  (funcall acc 3)    ;; => 18
  (funcall acc 1))   ;; => 19

;; The length of this in your language tells you
;; something about the language's power.
```

### On Lisp: Anaphoric Macros

```lisp
;; Standard: name the result explicitly every time
(let ((result (find-if #'expensive-p items)))
  (when result
    (process result)))

;; Graham's anaphoric style: 'it' binds automatically
(defmacro aif (test then &optional else)
  `(let ((it ,test))
     (if it ,then ,else)))

;; Now reads like English
(aif (find-if #'expensive-p items)
  (process it)
  (warn "nothing found"))
```

### The Arc Way: Brevity as Design Goal

```lisp
;; Arc is Graham's language, designed for maximum brevity.
;; The philosophy: every character you type is a cost.

;; Common Lisp:
(defun even-positives (lst)
  (remove-if-not #'evenp
    (remove-if-not #'plusp lst)))

;; Arc equivalent (conceptual):
(def even-positives (lst)
  (keep even (keep pos lst)))

;; The principle: if you use something often,
;; it should be short. Naming is compression.
```

### Rapid Prototyping: The Viaweb Method

```lisp
;; Graham built Viaweb by:
;; 1. Writing the simplest thing that could work
;; 2. Putting it in front of users immediately
;; 3. Iterating daily based on feedback

;; In code, this means:
;; - Don't design the database schema first
;; - Build the user-facing feature
;; - Let the data model emerge from what users need

;; Start with in-memory data
(defvar *store* (make-hash-table :test 'equal))

(defun save-page (id content)
  (setf (gethash id *store*) content))

(defun load-page (id)
  (gethash id *store*))

;; Ship it. Add persistence later when you know
;; what data actually matters.
```

## The Blub Paradox

Graham's most famous conceptual framework:

> Every programmer is satisfied with their current language and looks down on less powerful ones—but can't see what they're missing in more powerful ones. They're stuck on the "Blub" continuum.

The test: look at your code. Is there a pattern you keep repeating that a more powerful language could eliminate? That pattern is the ceiling of your current language.

- **If you're writing getters/setters** → you need first-class data
- **If you're writing visitors/strategies** → you need first-class functions
- **If you're copy-pasting with slight variations** → you need macros
- **If you're writing code generators** → you need compile-time metaprogramming

## Mental Model

Graham approaches programming by asking:

1. **What's the shortest program that does this?** Brevity reveals essence.
2. **Am I building up or building down?** Build up—create the language first.
3. **Can I eliminate this pattern?** If you see it twice, abstract it.
4. **What would a Lisp hacker do?** Even in other languages, think in transformations.
5. **Is this an essay or a bureaucratic form?** Code should read like the former.

## Signature Graham Moves

- Bottom-up language construction
- Macros that make patterns disappear
- Closures as the universal abstraction
- Programs shorter than you thought possible
- REPL-driven exploratory development
- Treating code as a creative medium, not an engineering artifact
- Shipping fast, iterating faster
