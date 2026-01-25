#!/usr/bin/env python3
"""Generate skills.json manifest from repository structure."""

import json
import re
from pathlib import Path
from datetime import datetime, timezone

REPO_ROOT = Path(__file__).parent.parent
SKILL_FILE = "SKILL.md"

def parse_frontmatter(content: str) -> dict:
    """Extract YAML frontmatter from markdown."""
    match = re.match(r'^---\s*\n(.*?)\n---', content, re.DOTALL)
    if not match:
        return {}
    
    frontmatter = {}
    for line in match.group(1).strip().split('\n'):
        if ':' in line:
            key, value = line.split(':', 1)
            frontmatter[key.strip()] = value.strip()
    return frontmatter

def extract_category_and_tags(path: Path) -> tuple[str, list[str]]:
    """Extract category and generate tags from path."""
    parts = path.relative_to(REPO_ROOT).parts
    
    # e.g., ('languages', 'python', 'vanrossum', 'SKILL.md')
    # or    ('domains', 'systems-architecture', 'lamport', 'SKILL.md')
    tags = []
    category = parts[0] if parts else "unknown"
    
    for part in parts[:-1]:  # Exclude SKILL.md
        if part not in ('SKILL.md',):
            tags.append(part.replace('-', '_'))
    
    return category, tags

def get_skill_files(skill_dir: Path) -> list[str]:
    """Get list of files in a skill directory."""
    files = []
    for f in skill_dir.iterdir():
        if f.is_file() and not f.name.startswith('.'):
            files.append(f.name)
    return sorted(files)

def generate_manifest() -> dict:
    """Generate the complete skills manifest."""
    skills = []
    
    for skill_path in sorted(REPO_ROOT.rglob(SKILL_FILE)):
        # Skip template
        if 'skill-template' in str(skill_path):
            continue
            
        skill_dir = skill_path.parent
        rel_path = skill_dir.relative_to(REPO_ROOT)
        
        content = skill_path.read_text()
        frontmatter = parse_frontmatter(content)
        category, tags = extract_category_and_tags(skill_path)
        
        # Extract engineer name from path
        engineer = skill_dir.name
        
        # Get subcategory (e.g., 'python' from 'languages/python/vanrossum')
        parts = rel_path.parts
        subcategory = parts[1] if len(parts) > 2 else None
        
        skill = {
            "id": frontmatter.get("name", engineer),
            "name": engineer,
            "description": frontmatter.get("description", ""),
            "category": category,
            "subcategory": subcategory,
            "path": str(rel_path),
            "files": get_skill_files(skill_dir),
            "tags": tags,
        }
        skills.append(skill)
    
    manifest = {
        "version": "1.0.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "repository": "https://github.com/copyleftdev/sk1llz",
        "raw_base_url": "https://raw.githubusercontent.com/copyleftdev/sk1llz/master",
        "skill_count": len(skills),
        "skills": skills,
    }
    
    return manifest

def main():
    manifest = generate_manifest()
    
    output_path = REPO_ROOT / "skills.json"
    with open(output_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"Generated {output_path} with {manifest['skill_count']} skills")

if __name__ == "__main__":
    main()
