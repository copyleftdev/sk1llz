# Contributing to $K1LLZ

Thank you for helping capture the wisdom of legendary software engineers.

## Adding a New Engineer

### Prerequisites

Before adding an engineer, ensure:

1. **Significant Impact**: The engineer has meaningfully influenced how code is written
2. **Documented Philosophy**: Their principles are documented in books, talks, or papers
3. **Distinct Perspective**: Their approach differs from existing skills

### Structure

Create a directory under the appropriate category:

```
languages/<lang>/<engineer-name>/
├── SKILL.md           # Required: Main skill definition
├── philosophy.md      # Recommended: Deep dive into their thinking
├── references.md      # Recommended: Books, talks, papers
├── patterns/          # Optional: Detailed code patterns
├── anti-patterns/     # Optional: What they advise against
└── examples/          # Optional: Canonical code examples
```

### SKILL.md Format

```yaml
---
name: <lastname>-<brief-descriptor>
description: <200 chars max describing when to use this skill>
---

# <Full Name> Style Guide

## Overview
Brief bio and why this engineer matters.

## Core Philosophy
2-3 key quotes that capture their thinking.

## Design Principles
Numbered list of their main beliefs.

## When Writing Code
### Always
- What they always do

### Never
- What they never do

### Prefer
- X over Y because Z

## Code Patterns
Concrete examples with BAD/GOOD comparisons.

## Mental Model
How they approach problems.

## Additional Resources
Links to supporting files.
```

### Quality Standards

1. **Accuracy**: Principles must be verifiable from primary sources
2. **Actionable**: Guidelines should be specific enough to apply
3. **Balanced**: Include both what to do and what to avoid
4. **Code Examples**: Show, don't just tell

### Style Guidelines

- Use the engineer's own terminology and framing
- Include direct quotes where available
- Provide context for why principles matter
- Show code examples in modern C++ (or appropriate language version)

## Adding a New Language

1. Create `languages/<lang>/README.md` with:
   - Brief language overview
   - Key figures to capture
   - Language-specific patterns

2. Identify founding figures and major contributors

3. Follow the same skill structure

## Adding a New Category

For paradigms, domains, or other groupings:

1. Create directory under appropriate parent
2. Add README explaining the category
3. Identify key engineers to include

## Review Process

1. Fork the repository
2. Create a feature branch
3. Add your skill
4. Ensure it follows the format
5. Submit a pull request

### Review Criteria

- [ ] SKILL.md follows required format
- [ ] Philosophy is accurately represented
- [ ] Code examples compile and demonstrate principles
- [ ] References are verifiable
- [ ] No factual errors about the engineer's positions

## Questions?

Open an issue if you're unsure whether an engineer fits or need guidance on the format.
