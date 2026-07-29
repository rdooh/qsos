import os
import json
import pytest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


@pytest.fixture
def pytest_ini_content():
    path = os.path.join(REPO_ROOT, "pytest.ini")
    with open(path) as f:
        return f.read()


def test_pytest_ini_exists():
    assert os.path.exists(os.path.join(REPO_ROOT, "pytest.ini")), "pytest.ini not found"


def test_pytest_ini_declares_cov(pytest_ini_content):
    assert "--cov=" in pytest_ini_content, "pytest.ini must declare --cov"


def test_pytest_ini_declares_cov_fail_under(pytest_ini_content):
    assert "--cov-fail-under=" in pytest_ini_content, "pytest.ini must declare --cov-fail-under"


def test_manifest_coverage_threshold_not_null():
    path = os.path.join(REPO_ROOT, "testing", "manifest.json")
    with open(path) as f:
        manifest = json.load(f)
    assert manifest["coverage_threshold"] is not None, "coverage_threshold must not be null"
    assert isinstance(manifest["coverage_threshold"], int), "coverage_threshold must be an integer"
