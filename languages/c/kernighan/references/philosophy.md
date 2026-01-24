# Brian Kernighan Philosophy

## The Elements of Programming Style

Kernighan's approach to programming emphasizes clarity, simplicity, and good taste.

> "Debugging is twice as hard as writing the code in the first place. Therefore, if you write the code as cleverly as possible, you are, by definition, not smart enough to debug it."

## Core Principles

### 1. Write Clearly - Don't Be Too Clever

```c
// Too clever
while (*s++ = *t++);

// Clear
while (*t != '\0') {
    *s = *t;
    s++;
    t++;
}
*s = '\0';
```

Both work. The first is idiomatic C. But for teaching and maintenance, clarity wins.

### 2. Say What You Mean, Simply and Directly

```c
// Indirect
if (!(x <= y))

// Direct
if (x > y)
```

```c
// Roundabout
for (i = 0; i < n; i++)
    if (a[i] == x)
        found = 1;
if (found)
    // ...

// Direct
for (i = 0; i < n; i++)
    if (a[i] == x)
        break;
if (i < n)
    // found
```

### 3. Use Library Functions

> "Don't reinvent the wheel."

```c
// Bad: Rolling your own
int mystrlen(char *s) {
    int n = 0;
    while (*s++)
        n++;
    return n;
}

// Good: Use the library
#include <string.h>
size_t len = strlen(s);
```

### 4. Avoid Temporary Variables Where Not Needed

```c
// Unnecessary temp
temp = a;
a = b;
b = temp;

// But sometimes temps improve clarity
discriminant = b*b - 4*a*c;
root1 = (-b + sqrt(discriminant)) / (2*a);
root2 = (-b - sqrt(discriminant)) / (2*a);
```

### 5. Write First in an Easy-to-Understand Pseudo-Language

Before coding:
```
for each element in the array
    if element matches target
        return its position
return not found
```

Then translate:
```c
int search(int arr[], int n, int target) {
    for (int i = 0; i < n; i++)
        if (arr[i] == target)
            return i;
    return -1;
}
```

## The K&R Style

### Brace Placement

```c
// K&R style (Kernighan & Ritchie)
if (condition) {
    statement;
} else {
    statement;
}

// Functions are different
int function(int x)
{
    return x * 2;
}
```

### Variable Declarations

```c
// Declare at point of first use (modern C)
for (int i = 0; i < n; i++) {
    int squared = i * i;
    // ...
}

// Or at block start (traditional)
{
    int i, squared;
    for (i = 0; i < n; i++) {
        squared = i * i;
        // ...
    }
}
```

## Documentation Philosophy

### Code Should Be Self-Documenting

```c
// Bad: Comment restates code
i = i + 1;  // increment i

// Good: Comment explains why
i = i + 1;  // skip the header row

// Best: Code needs no comment
row++;  // (context makes it obvious)
```

### Comment on the Why, Not the What

```c
// Bad: What
/* Loop through array */
for (int i = 0; i < n; i++)

// Good: Why
/* Process oldest entries first (FIFO order) */
for (int i = 0; i < n; i++)
```

## Error Handling

### Check Return Values

```c
// Bad: Ignoring errors
file = fopen(filename, "r");
fscanf(file, "%d", &value);

// Good: Handle errors
file = fopen(filename, "r");
if (file == NULL) {
    perror(filename);
    exit(EXIT_FAILURE);
}
if (fscanf(file, "%d", &value) != 1) {
    fprintf(stderr, "Error reading from %s\n", filename);
    exit(EXIT_FAILURE);
}
```

### Fail Gracefully

```c
void *safe_malloc(size_t size) {
    void *p = malloc(size);
    if (p == NULL) {
        fprintf(stderr, "Out of memory\n");
        exit(EXIT_FAILURE);
    }
    return p;
}
```

## Data Structure Philosophy

### Let Data Structure Your Program

```c
// Data drives the code
typedef struct {
    char *name;
    int (*func)(int, int);
} Operation;

Operation ops[] = {
    {"add", add},
    {"sub", subtract},
    {"mul", multiply},
    {"div", divide},
    {NULL, NULL}
};

// Simple lookup replaces switch/case
for (int i = 0; ops[i].name != NULL; i++)
    if (strcmp(ops[i].name, input) == 0)
        return ops[i].func(a, b);
```

## Testing Philosophy

### Test Early, Test Often

```c
// Simple test harness
#ifdef TEST
int main(void) {
    assert(factorial(0) == 1);
    assert(factorial(1) == 1);
    assert(factorial(5) == 120);
    printf("All tests passed\n");
    return 0;
}
#endif
```

### Boundary Conditions

Always test:
- Empty input
- Single element
- Maximum values
- Negative values (if applicable)

## The Unix Philosophy (with Thompson)

From their work on Unix:

1. **Make each program do one thing well**
2. **Expect output to become input**
3. **Design for reuse early**
4. **Use tools over manual labor**
5. **Prototype early**

## Famous Quotes

> "Everyone knows that debugging is twice as hard as writing a program in the first place."

> "Controlling complexity is the essence of computer programming."

> "Don't comment bad code — rewrite it."

> "Simplicity and clarity are the ultimate sophistications."

> "Unix is simple. It just takes a genius to understand its simplicity."

## The Practice of Programming (with Pike)

Key lessons from their book:

1. **Simplicity** - Keep it simple
2. **Clarity** - Make it clear
3. **Generality** - Solve the general problem
4. **Automation** - Let the computer do the work
5. **Testing** - Test everything, test early
6. **Performance** - Measure before optimizing
