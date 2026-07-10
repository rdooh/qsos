#!/usr/bin/env python3
import os
import re
import json
import shutil

# Paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SKILLS_SRC = os.path.join(SCRIPT_DIR, "skills")
AGENTS_SRC = os.path.join(SCRIPT_DIR, "agents")

# Target Plugin Directory inside Gemini config
GEMINI_PLUGIN_DIR = os.path.expanduser("~/.gemini/config/plugins/qsos")
SKILLS_DST = os.path.join(GEMINI_PLUGIN_DIR, "skills")
AGENTS_DST = os.path.join(GEMINI_PLUGIN_DIR, "agents")

def parse_md_frontmatter(content):
    """
    Parses frontmatter from markdown content.
    Returns (frontmatter_dict, body_content)
    """
    fm_match = re.match(r'^---\s*\n(.*?)\n---\s*\n(.*)$', content, re.DOTALL)
    if fm_match:
        fm_text = fm_match.group(1)
        body = fm_match.group(2)
        fm_dict = {}
        for line in fm_text.split('\n'):
            if ':' in line:
                key, val = line.split(':', 1)
                fm_dict[key.strip()] = val.strip().strip('"').strip("'")
        return fm_dict, body
    return {}, content

def check_and_self_heal(check_only=False):
    """
    Validates the deployed QSOS plugin structure.
    If check_only is False, it heals (creates/overwrites) missing or mismatching files.
    """
    status = {"ok": 0, "fixed": 0, "missing": 0, "mismatch": 0}
    
    # 1. Ensure plugin.json exists
    plugin_json_path = os.path.join(GEMINI_PLUGIN_DIR, "plugin.json")
    plugin_metadata = {
        "name": "qsos",
        "version": "1.0.0",
        "description": "Quality Sauce Operating System — developer-layer quality system for spec-first compliance and zero-drift code.",
        "author": {
            "name": "rob"
        },
        "repository": "https://github.com/rdooh/qsos",
        "license": "MIT"
    }
    
    plugin_json_str = json.dumps(plugin_metadata, indent=2) + "\n"
    
    if not os.path.exists(plugin_json_path):
        status["missing"] += 1
        print(f"[!] Missing plugin.json")
        if not check_only:
            os.makedirs(GEMINI_PLUGIN_DIR, exist_ok=True)
            with open(plugin_json_path, "w", encoding="utf-8") as f:
                f.write(plugin_json_str)
            print(f"  -> Healed: Created plugin.json")
            status["fixed"] += 1
    else:
        with open(plugin_json_path, "r", encoding="utf-8") as f:
            current_content = f.read()
        if current_content != plugin_json_str:
            status["mismatch"] += 1
            print(f"[!] Mismatch in plugin.json content")
            if not check_only:
                with open(plugin_json_path, "w", encoding="utf-8") as f:
                    f.write(plugin_json_str)
                print(f"  -> Healed: Reset plugin.json to standard metadata")
                status["fixed"] += 1
        else:
            status["ok"] += 1

    # 2. Check and deploy Skills
    if os.path.exists(SKILLS_SRC):
        os.makedirs(SKILLS_DST, exist_ok=True)
        for filename in os.listdir(SKILLS_SRC):
            if not filename.endswith(".md"):
                continue
            src_path = os.path.join(SKILLS_SRC, filename)
            skill_name = os.path.splitext(filename)[0]
            
            with open(src_path, "r", encoding="utf-8") as f:
                src_content = f.read()
            
            fm, body = parse_md_frontmatter(src_content)
            description = fm.get("description", f"QSOS skill: {skill_name}")
            
            # Construct expected SKILL.md content
            expected_content = f"---\nname: {skill_name}\ndescription: {description}\n---\n\n{body}"
            
            target_dir = os.path.join(SKILLS_DST, skill_name)
            target_file = os.path.join(target_dir, "SKILL.md")
            
            if not os.path.exists(target_file):
                status["missing"] += 1
                print(f"[!] Skill {skill_name} is missing in plugin")
                if not check_only:
                    os.makedirs(target_dir, exist_ok=True)
                    with open(target_file, "w", encoding="utf-8") as f:
                        f.write(expected_content)
                    print(f"  -> Healed: Created {target_file}")
                    status["fixed"] += 1
            else:
                with open(target_file, "r", encoding="utf-8") as f:
                    target_content = f.read()
                if target_content != expected_content:
                    status["mismatch"] += 1
                    print(f"[!] Skill {skill_name} content has drifted")
                    if not check_only:
                        with open(target_file, "w", encoding="utf-8") as f:
                            f.write(expected_content)
                        print(f"  -> Healed: Updated {target_file} to match source")
                        status["fixed"] += 1
                else:
                    status["ok"] += 1

    # 3. Check and deploy Agents
    if os.path.exists(AGENTS_SRC):
        os.makedirs(AGENTS_DST, exist_ok=True)
        for filename in os.listdir(AGENTS_SRC):
            if not filename.endswith(".md"):
                continue
            src_path = os.path.join(AGENTS_SRC, filename)
            target_file = os.path.join(AGENTS_DST, filename)
            
            with open(src_path, "r", encoding="utf-8") as f:
                src_content = f.read()
                
            if not os.path.exists(target_file):
                status["missing"] += 1
                print(f"[!] Agent {filename} is missing in plugin")
                if not check_only:
                    with open(target_file, "w", encoding="utf-8") as f:
                        f.write(src_content)
                    print(f"  -> Healed: Created {target_file}")
                    status["fixed"] += 1
            else:
                with open(target_file, "r", encoding="utf-8") as f:
                    target_content = f.read()
                if target_content != src_content:
                    status["mismatch"] += 1
                    print(f"[!] Agent {filename} content has drifted")
                    if not check_only:
                        with open(target_file, "w", encoding="utf-8") as f:
                            f.write(src_content)
                        print(f"  -> Healed: Updated {target_file}")
                        status["fixed"] += 1
                else:
                    status["ok"] += 1

    # 4. Clean up any obsolete/deleted files in target directories
    # Clean up obsolete skills
    if os.path.exists(SKILLS_DST):
        for name in os.listdir(SKILLS_DST):
            src_file_name = f"{name}.md"
            if not os.path.exists(os.path.join(SKILLS_SRC, src_file_name)):
                obsolete_dir = os.path.join(SKILLS_DST, name)
                print(f"[!] Obsolete skill folder found: {name}")
                if not check_only:
                    shutil.rmtree(obsolete_dir)
                    print(f"  -> Healed: Removed {obsolete_dir}")
                    status["fixed"] += 1
                else:
                    status["mismatch"] += 1
                    
    # Clean up obsolete agents
    if os.path.exists(AGENTS_DST):
        for filename in os.listdir(AGENTS_DST):
            if not os.path.exists(os.path.join(AGENTS_SRC, filename)):
                obsolete_file = os.path.join(AGENTS_DST, filename)
                print(f"[!] Obsolete agent file found: {filename}")
                if not check_only:
                    os.remove(obsolete_file)
                    print(f"  -> Healed: Removed {obsolete_file}")
                    status["fixed"] += 1
                else:
                    status["mismatch"] += 1

    # Print summary
    print(f"\nQSOS Gemini Integration Status:")
    print(f"  OK: {status['ok']}")
    print(f"  Missing: {status['missing']}")
    print(f"  Mismatched/Drifted: {status['mismatch']}")
    print(f"  Auto-healed: {status['fixed']}")
    
    if check_only and (status['missing'] > 0 or status['mismatch'] > 0):
        return False
    return True

if __name__ == "__main__":
    import sys
    check_mode = "--check" in sys.argv
    success = check_and_self_heal(check_only=check_mode)
    if not success:
        sys.exit(1)
