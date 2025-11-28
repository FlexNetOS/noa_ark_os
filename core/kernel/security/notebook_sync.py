"""Capability helpers for the notebook sync automation."""

from __future__ import annotations

from .tokens import issue_capability_token

NOTEBOOK_SYNC_FS_SCOPE = "fs.notebooks.sync"
NOTEBOOK_SYNC_NETWORK_SCOPE = "net.notebooks.loopback"


def issue_notebook_sync_token(
    *,
    client_id: str,
    rate_limit_per_minute: int = 60,
    lifetime_seconds: int = 3600,
) -> str:
    """Issue a pre-scoped capability token for notebook synchronisation."""

    return issue_capability_token(
        client_id=client_id,
        fs_scopes=[NOTEBOOK_SYNC_FS_SCOPE],
        network_scopes=[NOTEBOOK_SYNC_NETWORK_SCOPE],
        rate_limit_per_minute=rate_limit_per_minute,
        lifetime_seconds=lifetime_seconds,
    )
