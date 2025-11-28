"""Shared utilities for Python-based NOA Ark OS services."""

from .workspace import WorkspaceResolutionError, resolve_workspace_path

__all__ = ["WorkspaceResolutionError", "resolve_workspace_path"]
