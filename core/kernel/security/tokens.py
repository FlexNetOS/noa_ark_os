"""Kernel-facing helpers for capability token issuance and verification.

These helpers provide a deterministic reference implementation for capability
scopes. Tokens are signed using an HMAC derived from the kernel secret so that
service layers can rely on the kernel as the source of truth while remaining
language-agnostic.
"""

from __future__ import annotations

import base64
import binascii
import hashlib
import hmac
import json
import os
import time
from dataclasses import dataclass
from typing import Dict, Iterable, Optional, Tuple

__all__ = [
    "CapabilityTokenError",
    "KernelTokenClaims",
    "issue_capability_token",
    "verify_capability_token",
]


class CapabilityTokenError(RuntimeError):
    """Raised when a capability token cannot be verified by the kernel."""


@dataclass(frozen=True)
class KernelTokenClaims:
    """Validated capability token payload."""

    client_id: str
    fs_scopes: Tuple[str, ...]
    network_scopes: Tuple[str, ...]
    rate_limit_per_minute: int
    issued_at: float
    expires_at: float

    def allows_scope(self, scope_type: str, required: Optional[str]) -> bool:
        if required is None:
            return True
        if scope_type == "fs":
            return required in self.fs_scopes
        if scope_type == "network":
            return required in self.network_scopes
        raise ValueError(f"unknown scope type: {scope_type}")

    def allows_rate(self, required: Optional[int]) -> bool:
        if required is None:
            return True
        return self.rate_limit_per_minute >= required


def _secret() -> bytes:
    secret = os.getenv("NOA_CAPABILITY_TOKEN_SECRET", "noa-ark-capability-secret")
    return secret.encode("utf-8")


def _sign(payload: bytes) -> str:
    digest = hmac.new(_secret(), payload, hashlib.sha256)
    return digest.hexdigest()


def _encode_payload(payload: Dict[str, object]) -> str:
    serialised = json.dumps(payload, sort_keys=True).encode("utf-8")
    token_body = base64.urlsafe_b64encode(serialised).decode("ascii").rstrip("=")
    signature = _sign(serialised)
    return f"{token_body}.{signature}"


def _decode_payload(token: str) -> Dict[str, object]:
    try:
        body, signature = token.split(".", 1)
    except ValueError as exc:  # pragma: no cover - defensive guard
        raise CapabilityTokenError("malformed capability token") from exc

    padding = "=" * (-len(body) % 4)
    try:
        decoded = base64.urlsafe_b64decode(body + padding)
    except (ValueError, binascii.Error) as exc:  # pragma: no cover - safety
        raise CapabilityTokenError("invalid base64 payload") from exc

    expected_signature = _sign(decoded)
    if not hmac.compare_digest(expected_signature, signature):
        raise CapabilityTokenError("signature mismatch")

    try:
        payload = json.loads(decoded.decode("utf-8"))
    except json.JSONDecodeError as exc:  # pragma: no cover - defensive guard
        raise CapabilityTokenError("invalid JSON payload") from exc
    return payload


def issue_capability_token(
    *,
    client_id: str,
    fs_scopes: Iterable[str],
    network_scopes: Iterable[str],
    rate_limit_per_minute: int,
    lifetime_seconds: int = 3600,
) -> str:
    """Issue a signed capability token for integration tests and tooling."""

    issued_at = time.time()
    payload = {
        "client_id": client_id,
        "fs_scopes": list(fs_scopes),
        "network_scopes": list(network_scopes),
        "rate_limit_per_minute": int(rate_limit_per_minute),
        "issued_at": issued_at,
        "expires_at": issued_at + float(lifetime_seconds),
    }
    return _encode_payload(payload)


def verify_capability_token(token: str) -> KernelTokenClaims:
    """Validate a capability token and return the kernel-issued claims."""

    payload = _decode_payload(token)
    expires_at = float(payload.get("expires_at", 0))
    if expires_at <= time.time():
        raise CapabilityTokenError("capability token expired")

    return KernelTokenClaims(
        client_id=str(payload["client_id"]),
        fs_scopes=tuple(payload.get("fs_scopes", [])),
        network_scopes=tuple(payload.get("network_scopes", [])),
        rate_limit_per_minute=int(payload.get("rate_limit_per_minute", 0)),
        issued_at=float(payload.get("issued_at", 0.0)),
        expires_at=expires_at,
    )
