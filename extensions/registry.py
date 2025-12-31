"""Dynamic extension registry with kernel-guarded capability tokens."""

from __future__ import annotations

import importlib
import json
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType
from typing import Dict, Iterable, Optional, Set

from core.kernel.security import CapabilityTokenError, KernelTokenClaims, verify_capability_token

__all__ = ["ExtensionManifest", "ExtensionRegistry", "ExtensionRegistryError"]


class ExtensionRegistryError(RuntimeError):
    """Raised when extension manifests or adapters cannot be resolved."""


@dataclass(frozen=True)
class ExtensionManifest:
    """Declarative description of an extension adapter."""

    name: str
    version: str
    module: str
    entrypoint: str
    fs_scope: Optional[str]
    network_scope: Optional[str]
    rate_limit_per_minute: Optional[int]

    @classmethod
    def from_path(cls, manifest_path: Path) -> "ExtensionManifest":
        data = json.loads(manifest_path.read_text())
        scopes = data.get("scopes", {})
        rate_limit = scopes.get("rate_limit_per_minute")
        return cls(
            name=data["name"],
            version=data["version"],
            module=data["module"],
            entrypoint=data["entrypoint"],
            fs_scope=scopes.get("fs"),
            network_scope=scopes.get("network"),
            rate_limit_per_minute=int(rate_limit) if rate_limit is not None else None,
        )

    def ensure_authorized(self, token: str) -> KernelTokenClaims:
        claims = verify_capability_token(token)
        if not claims.allows_scope("fs", self.fs_scope):
            raise CapabilityTokenError("extension requires fs scope")
        if not claims.allows_scope("network", self.network_scope):
            raise CapabilityTokenError("extension requires network scope")
        if self.rate_limit_per_minute is not None and not claims.allows_rate(self.rate_limit_per_minute):
            raise CapabilityTokenError("extension requires higher rate allowance")
        return claims


class ExtensionRegistry:
    """Registry that loads extension adapters without restarting the gateway."""

    def __init__(self, root: Optional[Path] = None) -> None:
        self.root = Path(root or "extensions")
        self._manifests: Dict[str, ExtensionManifest] = {}
        self._manifest_mtimes: Dict[str, float] = {}
        self._loaded_modules: Dict[str, ModuleType] = {}
        self.refresh()

    def refresh(self) -> None:
        """Reload manifest definitions from disk."""

        discovered: Dict[str, ExtensionManifest] = {}
        mtimes: Dict[str, float] = {}
        changed: Set[str] = set()
        for manifest_path in sorted(self.root.glob("*/manifest.json")):
            manifest = ExtensionManifest.from_path(manifest_path)
            discovered[manifest.name] = manifest
            mtime = manifest_path.stat().st_mtime
            mtimes[manifest.name] = mtime
            if self._manifest_mtimes.get(manifest.name) != mtime:
                changed.add(manifest.name)

        removed = set(self._manifests) - set(discovered)
        for name in removed | changed:
            self._loaded_modules.pop(name, None)

        self._manifests = discovered
        self._manifest_mtimes = mtimes

    def registered_extensions(self) -> Iterable[str]:
        return self._manifests.keys()

    def manifest_for(self, name: str) -> ExtensionManifest:
        try:
            return self._manifests[name]
        except KeyError as exc:
            raise ExtensionRegistryError(f"extension '{name}' is not registered") from exc

    def load_adapter(self, name: str, token: str, **kwargs) -> object:
        manifest = self.manifest_for(name)
        claims = manifest.ensure_authorized(token)
        module = self._load_module(name, manifest)
        try:
            factory = getattr(module, manifest.entrypoint)
        except AttributeError as exc:
            raise ExtensionRegistryError(
                f"entrypoint '{manifest.entrypoint}' not found in module '{manifest.module}'"
            ) from exc

        adapter = factory(**kwargs)
        if hasattr(adapter, "bind_claims"):
            adapter.bind_claims(claims)
        else:
            setattr(adapter, "kernel_claims", claims)
        return adapter

    def _load_module(self, name: str, manifest: ExtensionManifest) -> ModuleType:
        module = self._loaded_modules.get(name)
        if module and module.__name__ == manifest.module:
            return module

        module = importlib.import_module(manifest.module)
        self._loaded_modules[name] = module
        return module
