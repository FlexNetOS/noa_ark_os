"""Utilities for replaying gateway telemetry in tests and local tooling."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any, Dict, Iterator, List, Optional

_DEFAULT_REPO_ROOT = Path(__file__).resolve().parents[3]
_DEFAULT_TELEMETRY_ROOT = Path("storage/telemetry")
_DEFAULT_METRICS_FILE = _DEFAULT_TELEMETRY_ROOT / "gateway_metrics.json"
_DEFAULT_EVENTS_FILE = _DEFAULT_TELEMETRY_ROOT / "gateway_events.log"


def _resolve_path(relative: Path, repo_root: Optional[Path]) -> Path:
    if repo_root is not None:
        return Path(repo_root) / relative
    env_root = os.environ.get("NOA_ROOT")
    base = Path(env_root) if env_root else _DEFAULT_REPO_ROOT
    return base / relative


def load_gateway_metrics(repo_root: Optional[Path] = None) -> Dict[str, Any]:
    """Return the most recent gateway metrics snapshot as a dictionary."""

    path = _resolve_path(_DEFAULT_METRICS_FILE, repo_root)
    if not path.exists():
        return {}
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError:
        return {}
    if not raw.strip():
        return {}
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError:
        return {}
    if isinstance(payload, dict):
        return payload
    return {"raw": payload}


def iter_gateway_events(
    repo_root: Optional[Path] = None, *, limit: Optional[int] = None
) -> Iterator[Dict[str, Any]]:
    """Yield gateway telemetry events (most recent first by default)."""

    path = _resolve_path(_DEFAULT_EVENTS_FILE, repo_root)
    if not path.exists():
        return iter(())
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return iter(())
    items: List[Dict[str, Any]] = []
    for line in lines:
        if not line.strip():
            continue
        try:
            items.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    if not items:
        return iter(())
    items.reverse()
    if limit is not None:
        items = items[: limit if limit >= 0 else 0]
    return iter(items)


def load_gateway_events(
    repo_root: Optional[Path] = None, *, limit: Optional[int] = None
) -> List[Dict[str, Any]]:
    """Return a list of gateway telemetry events for quick replay."""

    return list(iter_gateway_events(repo_root=repo_root, limit=limit))


__all__ = [
    "load_gateway_metrics",
    "iter_gateway_events",
    "load_gateway_events",
]
