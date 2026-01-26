# sk1llz CLI

A package manager for AI coding skills. Like Homebrew, but for skills.

## Installation

### From source

```bash
cd cli
cargo build --release
cp target/release/sk1llz ~/.local/bin/
```

### Shell completions

```bash
# Bash
sk1llz completions bash > ~/.bash_completion.d/sk1llz

# Zsh
sk1llz completions zsh > ~/.zfunc/_sk1llz

# Fish
sk1llz completions fish > ~/.config/fish/completions/sk1llz.fish
```

## Usage

```bash
# List all available skills
sk1llz list

# List skills by category
sk1llz list --category languages
sk1llz list --category paradigms
sk1llz list --category domains

# Search skills (fuzzy matching)
sk1llz search distributed
sk1llz search "rust safety"

# Get detailed info about a skill
sk1llz info lamport
sk1llz info stroustrup

# Show where skills would be installed
sk1llz where

# Install a skill (project-local if .claude/ exists, otherwise global)
sk1llz install lamport

# Force install to global ~/.claude/skills/
sk1llz install lamport --global

# Install to a custom location
sk1llz install lamport --target ./my-skills/

# Update the skill index
sk1llz update
```

## Skill Location Resolution

The CLI automatically detects the best installation location:

1. **Project-local** (`./.claude/skills/`): Used if a `.claude/` directory exists in the current working directory
2. **Global** (`~/.claude/skills/`): Fallback when no project-local config exists

This allows you to:
- Install skills globally for use across all projects
- Install skills locally to a specific project (version-controlled, team-shared)

```bash
# Check which location is active
sk1llz where

# Initialize project-local skills
mkdir -p .claude/skills

# Force global even when .claude/ exists locally
sk1llz install lamport --global
```

## How It Works

1. **Index**: The CLI fetches `skills.json` from the repository, which contains metadata about all available skills.

2. **Cache**: The index is cached locally at `~/.cache/sk1llz/skills.json` for fast lookups.

3. **Install**: When you install a skill, the CLI downloads the skill files from GitHub and places them in your Claude skills directory (`~/.claude/skills/` by default).

## Skill Categories

| Category | Description |
|----------|-------------|
| `languages` | Language-specific skills (Python, Rust, Go, etc.) |
| `paradigms` | Programming paradigm skills (functional, distributed, systems) |
| `domains` | Domain-specific skills (security, systems-architecture) |
| `organizations` | Organization methodology skills (Google SRE, Netflix Chaos) |

## Examples

### Find Python skills

```bash
$ sk1llz list --category languages | grep python
  vanrossum [python] Write Python code in the style of Guido van Rossum...
  hettinger [python] Write Python code in the style of Raymond Hettinger...
  beazley [python] Write Python code in the style of David Beazley...
```

### Search for distributed systems expertise

```bash
$ sk1llz search distributed
32 results for 'distributed':
  lamport [distributed] Design distributed systems...
  dean [distributed] Design distributed systems...
  vogels [systems-architecture] Design cloud-native systems...
```

### Install a skill

```bash
$ sk1llz install lamport
✓ Installed lamport to /home/user/.claude/skills/lamport
```

## Development

```bash
# Build debug
cargo build

# Build release (optimized, stripped)
cargo build --release

# Run tests
cargo test

# Regenerate skills.json manifest
python3 ../scripts/generate_manifest.py
```

## License

Apache 2.0
