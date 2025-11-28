"""Tests covering the shared workspace resolution helper."""

from __future__ import annotations

from pathlib import Path

import pytest

from server.python.common.workspace import (
    WorkspaceResolutionError,
    resolve_workspace_path,
)


def test_env_override_is_respected(tmp_path, monkeypatch):
    custom = tmp_path / "custom"
    custom.mkdir()
    monkeypatch.setenv("NOA_WORKSPACE_PATH", custom.as_posix())

    resolved = resolve_workspace_path()

    assert resolved == custom.resolve()


def test_home_workspace_is_used_when_env_missing(tmp_path, monkeypatch):
    default_workspace = tmp_path / "workspace"
    default_workspace.mkdir()
    monkeypatch.delenv("NOA_WORKSPACE_PATH", raising=False)
    monkeypatch.setattr(Path, "home", lambda: tmp_path)

    resolved = resolve_workspace_path()

    assert resolved == default_workspace.resolve()


def test_missing_workspace_raises_error(tmp_path, monkeypatch):
    missing = tmp_path / "missing"
    monkeypatch.setenv("NOA_WORKSPACE_PATH", missing.as_posix())

    with pytest.raises(WorkspaceResolutionError):
        resolve_workspace_path()
