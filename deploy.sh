#!/usr/bin/env bash
# deploy.sh — install QSOS artifacts to their system locations
#
# Usage:
#   ./deploy.sh           symlink all artifacts (default)
#   ./deploy.sh --check   report status without making changes; exits 1 if issues found
#   ./deploy.sh --copy    copy instead of symlink
#   ./deploy.sh --clean   remove all deployed QSOS artifacts (uninstall)
#
# Artifact types and targets:
#   skills/   → ~/.claude/commands/
#   agents/   → ~/.claude/agents/
#   utilities/ — stub (no deployment yet)
#   extension/ — stub (no deployment yet)
#
# Status lines per artifact (deploy modes):
#   linked      — new symlink created
#   already-ok  — correct symlink already in place, no change
#   cleaned     — stale symlink or file removed (no source)
#   copied      — file copied (--copy mode)
#   removed     — artifact removed (--clean mode)
#
# Status lines per artifact (--check mode):
#   ok           — correct symlink is in place
#   missing      — no file at destination
#   broken       — symlink at destination points to nonexistent target
#   wrong-target — symlink exists but points to wrong source
#   stale        — symlink in dst_dir points into our src_dir but source file is gone

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODE="symlink"
[[ "${1:-}" == "--check" ]] && MODE="check"
[[ "${1:-}" == "--copy" ]]  && MODE="copy"
[[ "${1:-}" == "--clean" ]] && MODE="clean"

SKILLS_SRC="$SCRIPT_DIR/skills"
AGENTS_SRC="$SCRIPT_DIR/agents"
SKILLS_DST="$HOME/.claude/commands"
AGENTS_DST="$HOME/.claude/agents"

linked=0; already=0; cleaned=0; copied=0; removed=0
check_ok=0; check_missing=0; check_broken=0; check_wrong=0; check_stale=0

# ---------------------------------------------------------------------------
# check_artifact <src_file> <dst_dir>
# Reports status without making changes.
# ---------------------------------------------------------------------------
check_artifact() {
  local src="$1"
  local dst_dir="$2"
  local filename
  filename="$(basename "$src")"
  local dst="$dst_dir/$filename"

  if [[ ! -e "$dst" && ! -L "$dst" ]]; then
    printf "  %-14s %s\n" "missing" "$filename"
    ((check_missing++)) || true
    return
  fi

  if [[ -L "$dst" ]]; then
    local target
    target="$(readlink "$dst")"
    if [[ ! -e "$target" ]]; then
      printf "  %-14s %s  (-> %s)\n" "broken" "$filename" "$target"
      ((check_broken++)) || true
      return
    fi
    if [[ "$target" != "$src" ]]; then
      printf "  %-14s %s  (-> %s, expected -> %s)\n" "wrong-target" "$filename" "$target" "$src"
      ((check_wrong++)) || true
      return
    fi
    printf "  %-14s %s\n" "ok" "$filename"
    ((check_ok++)) || true
  else
    printf "  %-14s %s  (copied, not symlinked)\n" "ok" "$filename"
    ((check_ok++)) || true
  fi
}

# ---------------------------------------------------------------------------
# check_stale_links <dst_dir> <src_dir>
# Reports symlinks in dst_dir that point into src_dir but whose target is gone.
# ---------------------------------------------------------------------------
check_stale_links() {
  local dst_dir="$1"
  local src_dir="$2"

  [[ -d "$dst_dir" ]] || return 0

  while IFS= read -r -d '' link; do
    local target
    target="$(readlink "$link" 2>/dev/null || true)"
    if [[ "$target" == "$src_dir"* && ! -e "$target" ]]; then
      printf "  %-14s %s  (-> %s)\n" "stale" "$(basename "$link")" "$target"
      ((check_stale++)) || true
    fi
  done < <(find "$dst_dir" -maxdepth 1 -type l -print0 2>/dev/null)
}

# ---------------------------------------------------------------------------
# deploy_artifact <src_file> <dst_dir>
# ---------------------------------------------------------------------------
deploy_artifact() {
  local src="$1"
  local dst_dir="$2"
  local filename
  filename="$(basename "$src")"
  local dst="$dst_dir/$filename"

  if [[ "$MODE" == "clean" ]]; then
    if [[ -L "$dst" || -f "$dst" ]]; then
      rm "$dst"
      echo "  removed     $filename"
      ((removed++)) || true
    fi
    return
  fi

  mkdir -p "$dst_dir"

  if [[ "$MODE" == "symlink" && -L "$dst" && "$(readlink "$dst")" == "$src" ]]; then
    echo "  already-ok  $filename"
    ((already++)) || true
    return
  fi

  [[ -L "$dst" || -f "$dst" ]] && rm "$dst"

  if [[ "$MODE" == "copy" ]]; then
    cp "$src" "$dst"
    echo "  copied      $filename"
    ((copied++)) || true
  else
    ln -s "$src" "$dst"
    echo "  linked      $filename"
    ((linked++)) || true
  fi
}

# ---------------------------------------------------------------------------
# clean_stale <dst_dir> <src_dir>
# Remove symlinks in dst_dir that point into src_dir but have no source file.
# ---------------------------------------------------------------------------
clean_stale() {
  local dst_dir="$1"
  local src_dir="$2"

  [[ -d "$dst_dir" ]] || return 0

  while IFS= read -r -d '' link; do
    local target
    target="$(readlink "$link" 2>/dev/null || true)"
    if [[ "$target" == "$src_dir"* ]]; then
      if [[ ! -f "$target" ]]; then
        rm "$link"
        echo "  cleaned     $(basename "$link") (stale -> $target)"
        ((cleaned++)) || true
      fi
    fi
  done < <(find "$dst_dir" -maxdepth 1 -type l -print0 2>/dev/null)
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

echo ""

if [[ "$MODE" == "check" ]]; then
  echo "QSOS health check"
  echo ""

  if [[ -d "$SKILLS_SRC" ]]; then
    echo "Skills ($SKILLS_DST):"
    check_stale_links "$SKILLS_DST" "$SKILLS_SRC"
    for f in "$SKILLS_SRC"/*.md; do
      [[ -f "$f" ]] && check_artifact "$f" "$SKILLS_DST"
    done
    echo ""
  fi

  if [[ -d "$AGENTS_SRC" ]]; then
    echo "Agents ($AGENTS_DST):"
    check_stale_links "$AGENTS_DST" "$AGENTS_SRC"
    for f in "$AGENTS_SRC"/*.md; do
      [[ -f "$f" ]] && check_artifact "$f" "$AGENTS_DST"
    done
    echo ""
  fi

  issues=$((check_missing + check_broken + check_wrong + check_stale))
  echo "Health: $check_ok ok, $check_missing missing, $check_broken broken, $check_wrong wrong-target, $check_stale stale."

  if [[ $issues -gt 0 ]]; then
    echo "Issues found: $issues — run ./deploy.sh to fix."
    exit 1
  else
    echo "All artifacts healthy."
    exit 0
  fi
fi

echo "QSOS deploy — mode: $MODE"
echo ""

if [[ -d "$SKILLS_SRC" ]]; then
  echo "Skills -> $SKILLS_DST"
  clean_stale "$SKILLS_DST" "$SKILLS_SRC"
  if [[ "$MODE" != "clean" ]]; then
    for f in "$SKILLS_SRC"/*.md; do
      [[ -f "$f" ]] && deploy_artifact "$f" "$SKILLS_DST"
    done
  fi
  echo ""
fi

if [[ -d "$AGENTS_SRC" ]]; then
  echo "Agents -> $AGENTS_DST"
  clean_stale "$AGENTS_DST" "$AGENTS_SRC"
  if [[ "$MODE" != "clean" ]]; then
    for f in "$AGENTS_SRC"/*.md; do
      [[ -f "$f" ]] && deploy_artifact "$f" "$AGENTS_DST"
    done
  fi
  echo ""
fi

if [[ "$MODE" != "clean" ]]; then
  echo "Utilities -> (no deployment yet — see utilities/README.md)"
  echo "Extension -> (no deployment yet — see extension/README.md)"
  echo ""
fi

total=$((linked + already + cleaned + copied + removed))
echo "Done. $linked linked, $already already-ok, $cleaned cleaned, $copied copied, $removed removed."
