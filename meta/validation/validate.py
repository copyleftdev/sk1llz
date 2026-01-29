#!/usr/bin/env python3
import os
import json
import yaml
import sys
from pathlib import Path

# Configuration
ROOT_DIR = Path(__file__).parent.parent.parent
SKILLS_JSON_PATH = ROOT_DIR / "skills.json"
SKILL_ROOTS = ["languages", "domains", "paradigms", "organizations"]
REQUIRED_FIELDS = ["name", "description"]

def load_skills_json():
    with open(SKILLS_JSON_PATH, "r") as f:
        return json.load(f)

def validate_skill_file(file_path):
    issues = []
    try:
        with open(file_path, "r") as f:
            content = f.read()
            
        if not content.startswith("---"):
            return ["Missing YAML frontmatter start (---)"]
            
        parts = content.split("---", 2)
        if len(parts) < 3:
            return ["Invalid frontmatter format"]
            
        try:
            frontmatter = yaml.safe_load(parts[1])
        except yaml.YAMLError as e:
            return [f"YAML parsing error: {e}"]
            
        for field in REQUIRED_FIELDS:
            if field not in frontmatter:
                issues.append(f"Missing required field: {field}")
                
    except Exception as e:
        issues.append(f"Error reading file: {e}")
        
    return issues

def main():
    print(f"Validating skills in {ROOT_DIR}...")
    
    # 1. Load Manifest
    try:
        manifest = load_skills_json()
        manifest_ids = {s['id'] for s in manifest['skills']}
        manifest_paths = {s['path'] for s in manifest['skills']}
    except Exception as e:
        print(f"CRITICAL: Failed to load skills.json: {e}")
        sys.exit(1)

    errors = 0
    skills_found = 0

    # 2. Walk directories
    for root_name in SKILL_ROOTS:
        root_path = ROOT_DIR / root_name
        if not root_path.exists():
            continue
            
        for root, dirs, files in os.walk(root_path):
            if "SKILL.md" in files:
                skill_path = Path(root)
                rel_path = skill_path.relative_to(ROOT_DIR)
                skills_found += 1
                
                # Check 1: Structure validation
                file_issues = validate_skill_file(skill_path / "SKILL.md")
                if file_issues:
                    print(f"\n[FAIL] {rel_path}")
                    for issue in file_issues:
                        print(f"  - {issue}")
                    errors += 1
                    continue

                # Check 2: Manifest consistency
                # Normalize path string for comparison (remove trailing slashes, handle windows seps if needed)
                # The manifest paths use forward slashes and no leading slash
                str_path = str(rel_path).replace("\\", "/")
                
                if str_path not in manifest_paths:
                     print(f"\n[WARN] {rel_path} found on disk but missing from skills.json")
                     # errors += 1 # Treating this as a warning for now
    
    print(f"\nScanned {skills_found} skills.")
    
    if errors > 0:
        print(f"Found {errors} errors.")
        sys.exit(1)
    else:
        print("All checks passed.")
        sys.exit(0)

if __name__ == "__main__":
    main()
