#!/usr/bin/env python3
import os
import sys
import re
import shutil

# Paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SKILLS_SRC = os.path.join(SCRIPT_DIR, "skills")
AGENTS_SRC = os.path.join(SCRIPT_DIR, "agents")

# Claude Target Paths
CLAUDE_DIR = os.path.expanduser("~/.claude")
CLAUDE_SKILLS_DST = os.path.join(CLAUDE_DIR, "commands")
CLAUDE_AGENTS_DST = os.path.join(CLAUDE_DIR, "agents")

# Gemini Target Paths
GEMINI_DIR = os.path.expanduser("~/.gemini")
GEMINI_PLUGIN_DIR = os.path.join(GEMINI_DIR, "config", "plugins", "qsos")
GEMINI_SKILLS_DST = os.path.join(GEMINI_PLUGIN_DIR, "skills")
GEMINI_AGENTS_DST = os.path.join(GEMINI_PLUGIN_DIR, "agents")

# Cursor Target Paths
CURSOR_DIR = os.path.expanduser("~/.cursor")
CURSOR_SKILLS_DST = os.path.join(CURSOR_DIR, "skills")


def load_model_config():
    """
    Loads model mapping from qsos.config.yml or config.yml.
    Returns a dict of tier -> concrete model ID.
    """
    config_paths = [
        os.path.join(SCRIPT_DIR, "qsos.config.yml"),
        os.path.join(SCRIPT_DIR, "config.yml")
    ]
    
    config = {}
    config_found_path = None
    for path in config_paths:
        if os.path.exists(path):
            config_found_path = path
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            # Simple YAML parser for key-value pairs
            for line in content.splitlines():
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if ":" in line:
                    key, val = line.split(":", 1)
                    config[key.strip()] = val.strip().strip('"').strip("'")
            break
            
    return config, config_found_path


def parse_md_frontmatter(content):
    """
    Parses frontmatter from markdown content.
    Returns (frontmatter_dict, body_content, raw_frontmatter_text)
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
        return fm_dict, body, fm_text
    return {}, content, ""


def compile_content_with_model(content, model_config):
    """
    Replaces model tier with concrete model ID in frontmatter.
    """
    fm, body, raw_fm = parse_md_frontmatter(content)
    if not fm or "model" not in fm:
        return content
        
    model_tier = fm["model"]
    if model_tier in model_config:
        resolved_model = model_config[model_tier]
        # Replace the model line inside the raw frontmatter
        new_raw_fm = re.sub(
            rf"^model:\s*{re.escape(model_tier)}\s*$",
            f"model: {resolved_model}",
            raw_fm,
            flags=re.MULTILINE
        )
        return f"---\n{new_raw_fm}\n---\n\n{body}"
    else:
        print(f"Error: Model tier '{model_tier}' is not mapped in your config.")
        sys.exit(1)


VALID_TIERS = {"low", "mid", "high"}


def validate_agent_sources():
    """
    Scans all source agent files and hard-fails if any model: field
    contains a concrete model ID instead of a valid abstract tier.
    Returns list of violations.
    """
    violations = []
    if not os.path.exists(AGENTS_SRC):
        return violations
    for filename in sorted(os.listdir(AGENTS_SRC)):
        if not filename.endswith(".md"):
            continue
        path = os.path.join(AGENTS_SRC, filename)
        with open(path, "r", encoding="utf-8") as f:
            content = f.read()
        fm, _, _ = parse_md_frontmatter(content)
        model_val = fm.get("model", "").strip()
        if model_val and model_val not in VALID_TIERS:
            violations.append((filename, model_val))
    return violations


def print_tier_table(model_config):
    """Prints the resolved tier→model mapping."""
    print("\nModel tiers resolved from config:")
    for tier in ("low", "mid", "high"):
        resolved = model_config.get(tier, "(not mapped)")
        print(f"  {tier:<6}→  {resolved}")
    print("")


def confirm(prompt):
    """Prompt for explicit confirmation. Exits on anything other than 'y'."""
    answer = input(f"{prompt} [y/N] ").strip().lower()
    if answer != "y":
        print("Aborted.")
        sys.exit(0)


def is_symlink_correct(link_path, target_path):
    if not os.path.islink(link_path):
        return False
    try:
        return os.readlink(link_path) == target_path
    except OSError:
        return False


def build_skill_md_content(src_content, skill_name, default_description=None):
    """Build directory-style SKILL.md content for Gemini/Cursor targets."""
    fm, body, _ = parse_md_frontmatter(src_content)
    name = fm.get("name", skill_name)
    description = fm.get("description", default_description or f"QSOS skill: {skill_name}")
    return f"---\nname: {name}\ndescription: {description}\n---\n\n{body}"


class ClaudeTarget:
    name = "claude"
    
    def __init__(self, model_config):
        self.model_config = model_config
        
    def is_installed(self):
        return os.path.exists(CLAUDE_DIR)
        
    def deploy(self, mode="symlink"):
        print(f"Deploying to Claude (mode: {mode})...")
        os.makedirs(CLAUDE_SKILLS_DST, exist_ok=True)
        os.makedirs(CLAUDE_AGENTS_DST, exist_ok=True)
        
        # 1. Clean stale links/files
        self.clean_stale()
        
        # 2. Deploy Skills (symlink or copy)
        if os.path.exists(SKILLS_SRC):
            for filename in os.listdir(SKILLS_SRC):
                if not filename.endswith(".md"):
                    continue
                src = os.path.join(SKILLS_SRC, filename)
                dst = os.path.join(CLAUDE_SKILLS_DST, filename)
                
                if os.path.lexists(dst):
                    os.remove(dst)
                
                if mode == "copy":
                    shutil.copy(src, dst)
                    print(f"  copied      {filename}")
                else:
                    os.symlink(src, dst)
                    print(f"  linked      {filename}")
                    
        # 3. Deploy Agents (MUST compile/copy to replace model tiers)
        if os.path.exists(AGENTS_SRC):
            for filename in os.listdir(AGENTS_SRC):
                if not filename.endswith(".md"):
                    continue
                src = os.path.join(AGENTS_SRC, filename)
                dst = os.path.join(CLAUDE_AGENTS_DST, filename)
                
                with open(src, "r", encoding="utf-8") as f:
                    content = f.read()
                    
                compiled = compile_content_with_model(content, self.model_config)
                
                if os.path.lexists(dst):
                    os.remove(dst)
                with open(dst, "w", encoding="utf-8") as f:
                    f.write(compiled)
                print(f"  compiled    {filename}")
                
    def check(self):
        print(f"Skills ({CLAUDE_SKILLS_DST}):")
        skills_ok = 0
        skills_missing = 0
        skills_broken = 0
        skills_wrong = 0
        skills_stale = 0
        
        # Check stale
        if os.path.exists(CLAUDE_SKILLS_DST):
            for filename in os.listdir(CLAUDE_SKILLS_DST):
                dst = os.path.join(CLAUDE_SKILLS_DST, filename)
                if os.path.islink(dst):
                    target = os.readlink(dst)
                    if target.startswith(SKILLS_SRC) and not os.path.exists(target):
                        print(f"  stale          {filename}  (-> {target})")
                        skills_stale += 1
                        
        if os.path.exists(SKILLS_SRC):
            for filename in os.listdir(SKILLS_SRC):
                if not filename.endswith(".md"):
                    continue
                src = os.path.join(SKILLS_SRC, filename)
                dst = os.path.join(CLAUDE_SKILLS_DST, filename)
                
                if not os.path.lexists(dst):
                    print(f"  missing        {filename}")
                    skills_missing += 1
                elif os.path.islink(dst):
                    target = os.readlink(dst)
                    if not os.path.exists(target):
                        print(f"  broken         {filename}  (-> {target})")
                        skills_broken += 1
                    elif target != src:
                        print(f"  wrong-target   {filename}  (-> {target}, expected -> {src})")
                        skills_wrong += 1
                    else:
                        print(f"  ok             {filename}")
                        skills_ok += 1
                else:
                    # Regular file (copied)
                    print(f"  ok             {filename}  (copied, not symlinked)")
                    skills_ok += 1
                    
        print(f"\nAgents ({CLAUDE_AGENTS_DST}):")
        agents_ok = 0
        agents_missing = 0
        agents_mismatch = 0
        
        if os.path.exists(AGENTS_SRC):
            for filename in os.listdir(AGENTS_SRC):
                if not filename.endswith(".md"):
                    continue
                src = os.path.join(AGENTS_SRC, filename)
                dst = os.path.join(CLAUDE_AGENTS_DST, filename)
                
                if not os.path.exists(dst):
                    print(f"  missing        {filename}")
                    agents_missing += 1
                else:
                    with open(src, "r", encoding="utf-8") as f:
                        src_content = f.read()
                    with open(dst, "r", encoding="utf-8") as f:
                        dst_content = f.read()
                        
                    expected = compile_content_with_model(src_content, self.model_config)
                    if dst_content != expected:
                        print(f"  mismatch       {filename}  (content has drifted/needs compilation)")
                        agents_mismatch += 1
                    else:
                        print(f"  ok             {filename}")
                        agents_ok += 1
                        
        total_ok = skills_ok + agents_ok
        total_missing = skills_missing + agents_missing
        total_broken = skills_broken
        total_wrong = skills_wrong
        total_stale = skills_stale
        total_mismatch = agents_mismatch
        
        return {
            "ok": total_ok,
            "missing": total_missing,
            "broken": total_broken,
            "wrong-target": total_wrong,
            "stale": total_stale,
            "mismatch": total_mismatch
        }
        
    def clean(self):
        print("Cleaning Claude artifacts...")
        removed = 0
        if os.path.exists(CLAUDE_SKILLS_DST) and os.path.exists(SKILLS_SRC):
            for filename in os.listdir(SKILLS_SRC):
                dst = os.path.join(CLAUDE_SKILLS_DST, filename)
                if os.path.lexists(dst):
                    os.remove(dst)
                    print(f"  removed     {filename}")
                    removed += 1
        if os.path.exists(CLAUDE_AGENTS_DST) and os.path.exists(AGENTS_SRC):
            for filename in os.listdir(AGENTS_SRC):
                dst = os.path.join(CLAUDE_AGENTS_DST, filename)
                if os.path.lexists(dst):
                    os.remove(dst)
                    print(f"  removed     {filename}")
                    removed += 1
        return removed

    def clean_stale(self):
        if os.path.exists(CLAUDE_SKILLS_DST):
            for filename in os.listdir(CLAUDE_SKILLS_DST):
                dst = os.path.join(CLAUDE_SKILLS_DST, filename)
                if os.path.islink(dst):
                    target = os.readlink(dst)
                    if target.startswith(SKILLS_SRC) and not os.path.exists(target):
                        os.remove(dst)
                        print(f"  cleaned     {filename} (stale link -> {target})")


class GeminiTarget:
    name = "gemini"
    
    def __init__(self, model_config):
        self.model_config = model_config
        
    def is_installed(self):
        return os.path.exists(GEMINI_DIR)
        
    def deploy(self, mode="copy"):
        print("Deploying to Gemini...")
        os.makedirs(GEMINI_PLUGIN_DIR, exist_ok=True)
        os.makedirs(GEMINI_SKILLS_DST, exist_ok=True)
        os.makedirs(GEMINI_AGENTS_DST, exist_ok=True)
        
        # 1. Write plugin.json
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
        import json
        plugin_json_str = json.dumps(plugin_metadata, indent=2) + "\n"
        with open(plugin_json_path, "w", encoding="utf-8") as f:
            f.write(plugin_json_str)
        print("  created     plugin.json")
        
        # 2. Deploy Skills
        if os.path.exists(SKILLS_SRC):
            for filename in os.listdir(SKILLS_SRC):
                if not filename.endswith(".md"):
                    continue
                src_path = os.path.join(SKILLS_SRC, filename)
                skill_name = os.path.splitext(filename)[0]
                
                with open(src_path, "r", encoding="utf-8") as f:
                    src_content = f.read()
                
                fm, body, _ = parse_md_frontmatter(src_content)
                description = fm.get("description", f"QSOS skill: {skill_name}")
                
                expected_content = build_skill_md_content(src_content, skill_name, description)
                
                target_dir = os.path.join(GEMINI_SKILLS_DST, skill_name)
                os.makedirs(target_dir, exist_ok=True)
                target_file = os.path.join(target_dir, "SKILL.md")
                
                with open(target_file, "w", encoding="utf-8") as f:
                    f.write(expected_content)
                print(f"  created     skills/{skill_name}/SKILL.md")
                
        # 3. Deploy Agents (with model resolution)
        if os.path.exists(AGENTS_SRC):
            for filename in os.listdir(AGENTS_SRC):
                if not filename.endswith(".md"):
                    continue
                src_path = os.path.join(AGENTS_SRC, filename)
                target_file = os.path.join(GEMINI_AGENTS_DST, filename)
                
                with open(src_path, "r", encoding="utf-8") as f:
                    content = f.read()
                    
                compiled = compile_content_with_model(content, self.model_config)
                with open(target_file, "w", encoding="utf-8") as f:
                    f.write(compiled)
                print(f"  compiled    agents/{filename}")
                
        # 4. Clean obsolete
        self.clean_obsolete()

    def check(self):
        print(f"Gemini integration ({GEMINI_PLUGIN_DIR}):")
        import json
        
        ok = 0
        missing = 0
        mismatch = 0
        
        # 1. plugin.json
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
            print("  missing        plugin.json")
            missing += 1
        else:
            with open(plugin_json_path, "r", encoding="utf-8") as f:
                content = f.read()
            if content != plugin_json_str:
                print("  mismatch       plugin.json")
                mismatch += 1
            else:
                ok += 1
                
        # 2. Skills
        if os.path.exists(SKILLS_SRC):
            for filename in os.listdir(SKILLS_SRC):
                if not filename.endswith(".md"):
                    continue
                skill_name = os.path.splitext(filename)[0]
                src_path = os.path.join(SKILLS_SRC, filename)
                target_file = os.path.join(GEMINI_SKILLS_DST, skill_name, "SKILL.md")
                
                if not os.path.exists(target_file):
                    print(f"  missing        skills/{skill_name}/SKILL.md")
                    missing += 1
                else:
                    with open(src_path, "r", encoding="utf-8") as f:
                        src_content = f.read()
                    fm, body, _ = parse_md_frontmatter(src_content)
                    description = fm.get("description", f"QSOS skill: {skill_name}")
                    expected_content = build_skill_md_content(src_content, skill_name, description)
                    
                    with open(target_file, "r", encoding="utf-8") as f:
                        target_content = f.read()
                        
                    if target_content != expected_content:
                        print(f"  mismatch       skills/{skill_name}/SKILL.md")
                        mismatch += 1
                    else:
                        ok += 1
                        
        # 3. Agents
        if os.path.exists(AGENTS_SRC):
            for filename in os.listdir(AGENTS_SRC):
                if not filename.endswith(".md"):
                    continue
                src_path = os.path.join(AGENTS_SRC, filename)
                target_file = os.path.join(GEMINI_AGENTS_DST, filename)
                
                if not os.path.exists(target_file):
                    print(f"  missing        agents/{filename}")
                    missing += 1
                else:
                    with open(src_path, "r", encoding="utf-8") as f:
                        src_content = f.read()
                    with open(target_file, "r", encoding="utf-8") as f:
                        target_content = f.read()
                        
                    expected = compile_content_with_model(src_content, self.model_config)
                    if target_content != expected:
                        print(f"  mismatch       agents/{filename}")
                        mismatch += 1
                    else:
                        ok += 1
                        
        # 4. Check obsolete
        obsolete_count = 0
        if os.path.exists(GEMINI_SKILLS_DST):
            for name in os.listdir(GEMINI_SKILLS_DST):
                if not os.path.exists(os.path.join(SKILLS_SRC, f"{name}.md")):
                    print(f"  stale          skills/{name}")
                    obsolete_count += 1
        if os.path.exists(GEMINI_AGENTS_DST):
            for filename in os.listdir(GEMINI_AGENTS_DST):
                if not os.path.exists(os.path.join(AGENTS_SRC, filename)):
                    print(f"  stale          agents/{filename}")
                    obsolete_count += 1
                    
        return {
            "ok": ok,
            "missing": missing,
            "broken": 0,
            "wrong-target": 0,
            "stale": obsolete_count,
            "mismatch": mismatch
        }

    def clean(self):
        print("Cleaning Gemini artifacts...")
        if os.path.exists(GEMINI_PLUGIN_DIR):
            shutil.rmtree(GEMINI_PLUGIN_DIR)
            print(f"  removed     {GEMINI_PLUGIN_DIR}")
            return 1
        return 0
        
    def clean_obsolete(self):
        if os.path.exists(GEMINI_SKILLS_DST):
            for name in os.listdir(GEMINI_SKILLS_DST):
                if not os.path.exists(os.path.join(SKILLS_SRC, f"{name}.md")):
                    shutil.rmtree(os.path.join(GEMINI_SKILLS_DST, name))
                    print(f"  cleaned obsolete skill {name}")
        if os.path.exists(GEMINI_AGENTS_DST):
            for filename in os.listdir(GEMINI_AGENTS_DST):
                if not os.path.exists(os.path.join(AGENTS_SRC, filename)):
                    os.remove(os.path.join(GEMINI_AGENTS_DST, filename))
                    print(f"  cleaned obsolete agent {filename}")


class CursorTarget:
    name = "cursor"

    def __init__(self, model_config):
        self.model_config = model_config

    def is_installed(self):
        return os.path.exists(CURSOR_DIR)

    def deploy(self, mode="copy"):
        print("Deploying to Cursor...")
        os.makedirs(CURSOR_SKILLS_DST, exist_ok=True)
        self.clean_obsolete()

        if os.path.exists(SKILLS_SRC):
            for filename in os.listdir(SKILLS_SRC):
                if not filename.endswith(".md"):
                    continue
                src_path = os.path.join(SKILLS_SRC, filename)
                skill_name = os.path.splitext(filename)[0]

                with open(src_path, "r", encoding="utf-8") as f:
                    src_content = f.read()

                fm, _, _ = parse_md_frontmatter(src_content)
                description = fm.get("description", f"QSOS skill: {skill_name}")
                expected_content = build_skill_md_content(src_content, skill_name, description)

                target_dir = os.path.join(CURSOR_SKILLS_DST, skill_name)
                os.makedirs(target_dir, exist_ok=True)
                target_file = os.path.join(target_dir, "SKILL.md")
                marker_file = os.path.join(target_dir, ".qsos-deploy")

                with open(target_file, "w", encoding="utf-8") as f:
                    f.write(expected_content)
                with open(marker_file, "w", encoding="utf-8") as f:
                    f.write("qsos\n")
                print(f"  created     skills/{skill_name}/SKILL.md")

    def check(self):
        print(f"Cursor integration ({CURSOR_SKILLS_DST}):")
        ok = 0
        missing = 0
        mismatch = 0
        obsolete_count = 0

        if os.path.exists(SKILLS_SRC):
            for filename in os.listdir(SKILLS_SRC):
                if not filename.endswith(".md"):
                    continue
                skill_name = os.path.splitext(filename)[0]
                src_path = os.path.join(SKILLS_SRC, filename)
                target_file = os.path.join(CURSOR_SKILLS_DST, skill_name, "SKILL.md")

                if not os.path.exists(target_file):
                    print(f"  missing        skills/{skill_name}/SKILL.md")
                    missing += 1
                else:
                    with open(src_path, "r", encoding="utf-8") as f:
                        src_content = f.read()
                    fm, _, _ = parse_md_frontmatter(src_content)
                    description = fm.get("description", f"QSOS skill: {skill_name}")
                    expected_content = build_skill_md_content(src_content, skill_name, description)

                    with open(target_file, "r", encoding="utf-8") as f:
                        target_content = f.read()

                    if target_content != expected_content:
                        print(f"  mismatch       skills/{skill_name}/SKILL.md")
                        mismatch += 1
                    else:
                        ok += 1

        if os.path.exists(CURSOR_SKILLS_DST):
            for name in os.listdir(CURSOR_SKILLS_DST):
                marker = os.path.join(CURSOR_SKILLS_DST, name, ".qsos-deploy")
                if not os.path.isfile(marker):
                    continue
                if not os.path.exists(os.path.join(SKILLS_SRC, f"{name}.md")):
                    print(f"  stale          skills/{name}")
                    obsolete_count += 1

        return {
            "ok": ok,
            "missing": missing,
            "broken": 0,
            "wrong-target": 0,
            "stale": obsolete_count,
            "mismatch": mismatch
        }

    def clean(self):
        print("Cleaning Cursor artifacts...")
        removed = 0
        if os.path.exists(CURSOR_SKILLS_DST) and os.path.exists(SKILLS_SRC):
            for filename in os.listdir(SKILLS_SRC):
                skill_name = os.path.splitext(filename)[0]
                target_dir = os.path.join(CURSOR_SKILLS_DST, skill_name)
                marker = os.path.join(target_dir, ".qsos-deploy")
                if os.path.isfile(marker) and os.path.isdir(target_dir):
                    shutil.rmtree(target_dir)
                    print(f"  removed     skills/{skill_name}")
                    removed += 1
        return removed

    def clean_obsolete(self):
        if not os.path.exists(CURSOR_SKILLS_DST):
            return
        for name in os.listdir(CURSOR_SKILLS_DST):
            marker = os.path.join(CURSOR_SKILLS_DST, name, ".qsos-deploy")
            if not os.path.isfile(marker):
                continue
            if not os.path.exists(os.path.join(SKILLS_SRC, f"{name}.md")):
                target_dir = os.path.join(CURSOR_SKILLS_DST, name)
                if os.path.isdir(target_dir):
                    shutil.rmtree(target_dir)
                    print(f"  cleaned obsolete skill {name}")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="QSOS Multi-Runtime Deployer")
    parser.add_argument("--target", choices=["claude", "gemini", "cursor", "all"], default="all",
                        help="Deployment target runtime (default: all detected)")
    parser.add_argument("--check", action="store_true", help="Report status without making changes")
    parser.add_argument("--copy", action="store_true", help="Copy files instead of symlinking for Claude target")
    parser.add_argument("--clean", action="store_true", help="Remove all deployed artifacts")
    parser.add_argument("--fix", action="store_true", help="Heal/redeploy target runtime(s)")
    parser.add_argument("--yes", action="store_true", help="Skip confirmation prompts")
    
    args = parser.parse_args()
    
    # 1. Load configuration (only skip if cleaning)
    model_config = {}
    if not args.clean:
        model_config, config_path = load_model_config()
        if not model_config:
            print("Error: No 'qsos.config.yml' or 'config.yml' mapping file found.")
            print("Please create 'qsos.config.yml' mapping model tiers (low, mid, high) to concrete model names.")
            print("See qsos.config.yml.example for a template.")
            sys.exit(1)
        else:
            print(f"Loaded model config from: {config_path}")
            
    # 2. Initialize targets
    targets = [ClaudeTarget(model_config), GeminiTarget(model_config), CursorTarget(model_config)]
    
    # 3. Filter targets by detection and CLI flag
    active_targets = []
    detected_str = []
    
    for t in targets:
        installed = t.is_installed()
        if installed:
            detected_str.append(t.name)
        if args.target == "all":
            if installed:
                active_targets.append(t)
        elif args.target == t.name:
            active_targets.append(t)
            
    if args.target == "all" and not active_targets:
        # If none detected, fall back to compiling/processing both if specified explicitly
        # but otherwise warn and default to both
        print("No installed runtimes detected. Defaulting to deploying to all targets.")
        active_targets = targets
    else:
        print(f"Targets detected: {', '.join(detected_str)} — deploying to {', '.join([t.name for t in active_targets])}.")
        print("Use --target <name> to limit scope.")
        
    if args.clean:
        print("\nQSOS Clean mode")
        if not args.yes:
            confirm(f"Remove all deployed artifacts for: {', '.join([t.name for t in active_targets])}?")
        for t in active_targets:
            t.clean()
        print("\nDone cleaning.")
        sys.exit(0)

    if args.check:
        print("\nQSOS health check\n")

        # Source validation — always runs, even in check mode
        violations = validate_agent_sources()
        if violations:
            print("Source validation FAILED — concrete model IDs found in agent source files:")
            for filename, model_val in violations:
                print(f"  agents/{filename}: 'model: {model_val}' is a concrete model ID. Use a tier (low/mid/high) instead.")
            print("\nFix the source files before deploying.")
            sys.exit(1)
        else:
            print("Source validation passed — all agent model fields use abstract tiers.\n")

        issues = 0
        totals = {"ok": 0, "missing": 0, "broken": 0, "wrong-target": 0, "stale": 0, "mismatch": 0}

        for t in active_targets:
            print(f"--- Checking {t.name} target ---")
            res = t.check()
            for k in totals:
                totals[k] += res[k]
            print("")

        issues = totals["missing"] + totals["broken"] + totals["wrong-target"] + totals["stale"] + totals["mismatch"]
        print(f"Health summary: {totals['ok']} ok, {totals['missing']} missing, {totals['broken']} broken, {totals['wrong-target']} wrong-target, {totals['stale']} stale, {totals['mismatch']} content-mismatch.")

        if issues > 0:
            print(f"\nIssues found: {issues} — run deploy.py (without --check) to fix.")
            sys.exit(1)
        else:
            print("\nAll targets healthy.")
            sys.exit(0)

    # Default Deployment/Fix mode

    # Source validation — hard fail before any writes
    violations = validate_agent_sources()
    if violations:
        print("\nDeploy BLOCKED — concrete model IDs found in agent source files:")
        for filename, model_val in violations:
            print(f"  agents/{filename}: 'model: {model_val}' is a concrete model ID. Use a tier (low/mid/high) instead.")
        print("\nFix the source files before deploying.")
        sys.exit(1)

    # Print tier resolution table and require confirmation before any writes
    print_tier_table(model_config)
    targets_str = ', '.join([t.name for t in active_targets])
    if not args.yes:
        confirm(f"Deploy to {targets_str} using the model mapping above?")

    print("\nQSOS deploy")
    mode = "copy" if args.copy else "symlink"
    for t in active_targets:
        t.deploy(mode=mode)
    print("\nDeployment complete.")


if __name__ == "__main__":
    main()
