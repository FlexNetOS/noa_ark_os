"""Simple adapter used to verify extension hot swapping."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from core.kernel.security import KernelTokenClaims


class SampleAdapter:
    """In-memory adapter that captures invocations for verification."""

    def __init__(self) -> None:
        self.kernel_claims: Optional[KernelTokenClaims] = None
        self.invocations: List[Dict[str, Any]] = []

    def bind_claims(self, claims: KernelTokenClaims) -> None:
        self.kernel_claims = claims

    def invoke(self, payload: Dict[str, Any]) -> Dict[str, Any]:
        self.invocations.append(payload)
        return {"status": "ok", "payload": payload, "fs_scopes": self.kernel_claims.fs_scopes if self.kernel_claims else ()}
