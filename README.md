# $K1LLZ

> Capturing the coding philosophies, patterns, and wisdom of legendary software engineers as Anthropic-compatible Skills.

*"The skills of the gods, encoded for mortals."*

## Vision

This repository encapsulates the **essence of how the greatest software engineers think and write code**. Each skill represents not just coding patterns, but the mental models, design philosophies, and hard-won wisdom of masters in their respective domains.

## Structure

```
skill_of_the_gods/
├── README.md
├── CONTRIBUTING.md
├── languages/
│   ├── cpp/                    # C++ Masters
│   │   ├── stroustrup/         # Bjarne Stroustrup - Creator of C++
│   │   ├── meyers/             # Scott Meyers - Effective C++
│   │   ├── sutter/             # Herb Sutter - Exceptional C++
│   │   ├── alexandrescu/       # Andrei Alexandrescu - Modern C++ Design
│   │   ├── stepanov/           # Alexander Stepanov - STL Creator
│   │   ├── parent/             # Sean Parent - No Raw Loops
│   │   └── _shared/            # Shared C++ resources
│   ├── c/                      # C Masters
│   ├── rust/                   # Rust Masters
│   ├── python/                 # Python Masters
│   ├── javascript/             # JavaScript Masters
│   └── go/                     # Go Masters
├── paradigms/
│   ├── functional/             # Functional Programming Masters
│   ├── systems/                # Systems Programming Masters
│   └── distributed/            # Distributed Systems Masters
├── domains/
│   ├── compilers/              # Compiler Writers
│   ├── databases/              # Database Architects
│   ├── graphics/               # Graphics Programming
│   └── security/               # Security Experts
└── meta/
    ├── skill-template/         # Template for creating new skills
    └── validation/             # Skill validation tools
```

## Skill Format

Each engineer's skill follows Anthropic's Agent Skills specification:

```
engineer-name/
├── SKILL.md              # Core skill definition (required)
├── philosophy.md         # Design philosophy & mental models
├── patterns/             # Code patterns & idioms
│   └── *.md
├── anti-patterns/        # What to avoid & why
│   └── *.md
├── examples/             # Canonical code examples
│   └── *.cpp
└── references.md         # Books, talks, papers
```

## Usage

These skills can be used with:
- **Claude Code**: Place in `~/.claude/skills/` or project `.claude/skills/`
- **Claude.ai**: Upload as custom skills
- **Any Agent Skills compatible system**: Per [agentskills.io](https://agentskills.io)

## Philosophy

We don't just capture *what* these engineers write—we capture *how they think*:

1. **Mental Models**: How do they approach problems?
2. **Design Principles**: What invariants do they hold sacred?
3. **Trade-off Analysis**: How do they weigh competing concerns?
4. **Evolution**: How has their thinking changed over time?

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on adding new engineers or languages.

## License

Apache 2.0 - See LICENSE
