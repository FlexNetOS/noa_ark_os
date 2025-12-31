"""Workspace discovery helpers wired to gateway-managed configuration."""

from __future__ import annotations

import logging
import os
from pathlib import Path
from typing import Mapping, Optional

__all__ = ["WorkspaceResolutionError", "resolve_workspace_path"]

_ENV_VAR_NAME = "NOA_WORKSPACE_PATH"
_DEFAULT_WORKSPACE_SUFFIX = "workspace"

logger = logging.getLogger(__name__)


class WorkspaceResolutionError(RuntimeError):
    """Raised when the workspace path cannot be resolved or validated."""


def _gateway_override(env: Mapping[str, str]) -> Optional[str]:
    value = env.get(_ENV_VAR_NAME)
    if value:
        trimmed = value.strip()
        if trimmed:
            return trimmed
    return None


def resolve_workspace_path(
    *,
    env: Optional[Mapping[str, str]] = None,
    ensure_exists: bool = True,
) -> Path:
    """Return the configured workspace path.

    The helper looks for ``NOA_WORKSPACE_PATH`` (managed by the gateway
    configuration), falling back to ``Path.home() / "workspace"`` when no
    override is provided. Paths are normalised and validated to prevent callers
    from silently operating on a missing directory.
    """

    env_map = env or os.environ
    raw_path = _gateway_override(env_map)
    if raw_path:
        candidate = Path(raw_path)
    else:
        candidate = Path.home() / _DEFAULT_WORKSPACE_SUFFIX

    resolved = candidate.expanduser().resolve()

    if ensure_exists:
        if not resolved.exists():
            raise WorkspaceResolutionError(
                f"Workspace path '{resolved}' does not exist. "
                f"Set {_ENV_VAR_NAME} via the gateway config or create the directory."
            )
        if not resolved.is_dir():
            raise WorkspaceResolutionError(
                f"Workspace path '{resolved}' is not a directory."
            )

    logger.debug("Resolved workspace path: %s", resolved)
    return resolved
