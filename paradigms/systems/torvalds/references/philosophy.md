# Linus Torvalds Philosophy

## Good Taste in Code

From Torvalds' famous TED talk, "good taste" means eliminating special cases.

### The Classic Example: Linked List Deletion

```c
// Bad taste: Special case for head
void remove_list_entry(List *list, Entry *entry) {
    Entry *prev = NULL;
    Entry *curr = list->head;
    
    while (curr != entry) {
        prev = curr;
        curr = curr->next;
    }
    
    // Special case!
    if (prev == NULL) {
        list->head = entry->next;
    } else {
        prev->next = entry->next;
    }
}

// Good taste: No special cases
void remove_list_entry(Entry **indirect, Entry *entry) {
    while (*indirect != entry) {
        indirect = &(*indirect)->next;
    }
    *indirect = entry->next;
}
```

The pointer-to-pointer approach eliminates the special case entirely.

## Simplicity and Clarity

### The Right Data Structure

> "Bad programmers worry about the code. Good programmers worry about data structures and their relationships."

```c
// Design data structures first
struct task_struct {
    volatile long state;
    void *stack;
    struct list_head tasks;
    struct mm_struct *mm;
    pid_t pid;
    // ...
};

// Code follows naturally from good structure
```

### Small Functions

Torvalds advocates for functions that fit on one screen:

```c
// Good: Single purpose, clear
static inline int signal_pending(struct task_struct *p) {
    return unlikely(test_tsk_thread_flag(p, TIF_SIGPENDING));
}

// Bad: Does too many things, hard to follow
```

## Error Handling

### Handle Errors Where They Occur

```c
// Good: Early return on error
int do_something(struct foo *f) {
    int ret;
    
    ret = first_step(f);
    if (ret)
        return ret;
    
    ret = second_step(f);
    if (ret)
        goto undo_first;
    
    ret = third_step(f);
    if (ret)
        goto undo_second;
    
    return 0;

undo_second:
    undo_second_step(f);
undo_first:
    undo_first_step(f);
    return ret;
}
```

### The goto Controversy

Torvalds defends `goto` for error handling:

> "Anybody who tells me I can't use goto is either (a) ignorant, or (b) a fanatic."

The kernel's error unwinding pattern is cleaner with goto than nested conditionals.

## Code Review Philosophy

### Be Direct

Torvalds is famous for blunt code review:

> "Your code is shit" is more useful than polite vagueness.

But context matters—criticize code, not people.

### Maintainability Over Cleverness

```c
// Bad: Clever but obscure
#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]) + \
    sizeof(typeof(int[1 - 2*!!__builtin_types_compatible_p(typeof(arr), typeof(&arr[0]))])) * 0)

// Better: Clear with comment
#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))
// Note: Only works for actual arrays, not pointers
```

## Git Design Philosophy

Torvalds created Git with specific principles:

### 1. Speed

> "If it takes more than a second, it's too slow."

### 2. Simple Model

```
blob    → file contents
tree    → directory listing
commit  → snapshot + metadata + parent(s)
ref     → pointer to commit
```

### 3. Distributed First

No central server is special. Every clone is a full repository.

### 4. Data Integrity

```bash
# Every object is content-addressed
$ git hash-object file.txt
e69de29bb2d1d6434b8b29ae775ad8c2e48c5391

# Can't silently corrupt data
```

## Linux Kernel Coding Style

Key principles from Documentation/CodingStyle:

### Indentation: Tabs, 8 Characters

> "If you need more than 3 levels of indentation, you're screwed anyway."

### Naming

```c
// Good: Descriptive, lowercase with underscores
struct vm_area_struct *find_vma(struct mm_struct *mm, unsigned long addr);

// Bad: Hungarian notation, CamelCase
PVMAREA pVma = FindVMA(pMm, ulAddr);
```

### Braces

```c
// K&R style, but functions are special
int function(int x)
{
    if (condition) {
        do_something();
    } else {
        do_other();
    }
}
```

## Performance Philosophy

### Measure, Don't Guess

```c
// Profile before optimizing
// The kernel has tracing, perf, ftrace

// Wrong assumption: "malloc is slow"
// Reality: Often the cache miss is what's slow
```

### Cache-Friendly Data

```c
// Good: Data used together is stored together
struct point {
    int x;
    int y;
};
struct point points[1000];  // Contiguous in memory

// Bad: Pointer chasing
struct node {
    int value;
    struct node *next;  // Cache miss every access
};
```

## Famous Quotes

> "Talk is cheap. Show me the code."

> "Software is like sex: it's better when it's free."

> "Theory and practice sometimes clash. And when that happens, theory loses. Every single time."

> "Given enough eyeballs, all bugs are shallow." (Linus's Law, named by Raymond)

> "I'm a bastard. I have absolutely no languid."

> "Intelligence is the ability to avoid doing work, yet getting the work done."

## Key Principles Summary

1. **Eliminate special cases** through better design
2. **Data structures first**, algorithms second
3. **Simple, clear code** beats clever code
4. **Handle errors explicitly** where they occur
5. **Performance matters**, but measure first
6. **Distributed** beats centralized
7. **Show the code**, not the theory
