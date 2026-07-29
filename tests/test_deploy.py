import pytest
from deploy import parse_md_frontmatter, compile_content_with_model, validate_agent_sources, is_symlink_correct
import os
import tempfile


# --- parse_md_frontmatter ---

def test_parse_md_frontmatter_with_valid_frontmatter():
    content = "---\nname: foo\ndescription: bar\n---\n\nBody text here."
    fm, body, raw_fm = parse_md_frontmatter(content)
    assert fm == {"name": "foo", "description": "bar"}
    assert body == "Body text here."
    assert "name: foo" in raw_fm


def test_parse_md_frontmatter_no_frontmatter():
    content = "Just body text, no frontmatter."
    fm, body, raw_fm = parse_md_frontmatter(content)
    assert fm == {}
    assert body == content
    assert raw_fm == ""


def test_parse_md_frontmatter_strips_quotes():
    content = "---\nmodel: 'high'\n---\n\nBody."
    fm, body, _ = parse_md_frontmatter(content)
    assert fm["model"] == "high"


def test_parse_md_frontmatter_empty_body():
    content = "---\nname: only-frontmatter\n---\n\n"
    fm, body, _ = parse_md_frontmatter(content)
    assert fm["name"] == "only-frontmatter"
    assert body == ""


# --- compile_content_with_model ---

def test_compile_replaces_tier_with_model():
    content = "---\nmodel: high\n---\n\nSkill body."
    model_config = {"high": "claude-opus-5", "mid": "claude-sonnet-5", "low": "claude-haiku-4-5"}
    result = compile_content_with_model(content, model_config)
    assert "model: claude-opus-5" in result
    assert "model: high" not in result


def test_compile_leaves_content_without_model_unchanged():
    content = "---\nname: no-model\n---\n\nBody."
    model_config = {"high": "claude-opus-5"}
    result = compile_content_with_model(content, model_config)
    assert result == content


def test_compile_no_frontmatter_unchanged():
    content = "Plain content with no frontmatter."
    model_config = {"high": "claude-opus-5"}
    result = compile_content_with_model(content, model_config)
    assert result == content


def test_compile_unknown_tier_exits(monkeypatch):
    content = "---\nmodel: unknown-tier\n---\n\nBody."
    model_config = {"high": "claude-opus-5"}
    with pytest.raises(SystemExit):
        compile_content_with_model(content, model_config)


# --- validate_agent_sources ---

def test_validate_agent_sources_no_violations(tmp_path):
    agent_file = tmp_path / "my-agent.md"
    agent_file.write_text("---\nmodel: mid\n---\n\nAgent body.")
    import deploy as d
    original = d.AGENTS_SRC
    d.AGENTS_SRC = str(tmp_path)
    try:
        violations = validate_agent_sources()
        assert violations == []
    finally:
        d.AGENTS_SRC = original


def test_validate_agent_sources_detects_concrete_model(tmp_path):
    agent_file = tmp_path / "bad-agent.md"
    agent_file.write_text("---\nmodel: claude-sonnet-5\n---\n\nAgent body.")
    import deploy as d
    original = d.AGENTS_SRC
    d.AGENTS_SRC = str(tmp_path)
    try:
        violations = validate_agent_sources()
        assert len(violations) == 1
        assert violations[0][0] == "bad-agent.md"
        assert violations[0][1] == "claude-sonnet-5"
    finally:
        d.AGENTS_SRC = original


def test_validate_agent_sources_missing_dir():
    import deploy as d
    original = d.AGENTS_SRC
    d.AGENTS_SRC = "/nonexistent/path"
    try:
        violations = validate_agent_sources()
        assert violations == []
    finally:
        d.AGENTS_SRC = original


# --- is_symlink_correct ---

def test_is_symlink_correct_valid(tmp_path):
    target = tmp_path / "target.md"
    target.write_text("content")
    link = tmp_path / "link.md"
    os.symlink(str(target), str(link))
    assert is_symlink_correct(str(link), str(target)) is True


def test_is_symlink_correct_wrong_target(tmp_path):
    target1 = tmp_path / "target1.md"
    target2 = tmp_path / "target2.md"
    target1.write_text("a")
    target2.write_text("b")
    link = tmp_path / "link.md"
    os.symlink(str(target1), str(link))
    assert is_symlink_correct(str(link), str(target2)) is False


def test_is_symlink_correct_not_a_symlink(tmp_path):
    regular = tmp_path / "regular.md"
    regular.write_text("content")
    assert is_symlink_correct(str(regular), str(regular)) is False


def test_is_symlink_correct_nonexistent():
    assert is_symlink_correct("/nonexistent/link", "/nonexistent/target") is False
