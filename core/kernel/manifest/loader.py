from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, Iterable, List, Optional

_MANIFEST_PATH = Path(__file__).with_name("kernel_graph.json")


@dataclass(frozen=True)
class HealthCheck:
    type: str
    path: str


@dataclass(frozen=True)
class Compatibility:
    status: str
    notes: Optional[str] = None


@dataclass(frozen=True)
class ArtifactLocation:
    container: Dict[str, str]
    tarball: Dict[str, str]
    ociLayout: Dict[str, str]


@dataclass(frozen=True)
class KernelService:
    id: str
    name: str
    category: str
    version: str
    interfaces: List[Dict[str, str]]
    dependencies: List[str]
    optionalDependencies: List[str]
    artifacts: ArtifactLocation
    healthChecks: List[HealthCheck]
    compatibility: Dict[str, Compatibility]
    boot: Dict[str, object]
    resources: Dict[str, object]

    def requires(self) -> List[str]:
        return list(self.dependencies)

    def optional_requires(self) -> List[str]:
        return list(self.optionalDependencies)

    def supports_target(self, target: str) -> bool:
        comp = self.compatibility.get(target)
        return bool(comp) and comp.status == "supported"


@dataclass(frozen=True)
class Profile:
    description: str
    bootOrder: List[str]
    health: Dict[str, object]


@dataclass(frozen=True)
class KernelManifest:
    version: str
    schemaVersion: str
    generatedAt: str
    targets: List[str]
    services: Dict[str, KernelService] = field(default_factory=dict)
    profiles: Dict[str, Profile] = field(default_factory=dict)

    def services_in_boot_order(self, profile: str) -> List[KernelService]:
        profile_info = self.profiles.get(profile)
        if not profile_info:
            raise KeyError(f"Unknown profile: {profile}")
        return [self.services[service_id] for service_id in profile_info.bootOrder]

    def required_interfaces(self) -> Dict[str, List[str]]:
        interfaces: Dict[str, List[str]] = {}
        for service in self.services.values():
            for interface in service.interfaces:
                interfaces.setdefault(interface["name"], []).append(interface["version"])
        return interfaces

    def validate_targets(self, required_targets: Iterable[str]) -> None:
        missing: Dict[str, List[str]] = {}
        for service in self.services.values():
            for target in required_targets:
                if not service.supports_target(target):
                    missing.setdefault(service.id, []).append(target)
        if missing:
            raise ValueError(f"Services missing target support: {missing}")


def _as_health_checks(items: Iterable[Dict[str, str]]) -> List[HealthCheck]:
    return [HealthCheck(**item) for item in items]


def _as_compatibility(items: Dict[str, Dict[str, str]]) -> Dict[str, Compatibility]:
    return {name: Compatibility(**fields) for name, fields in items.items()}


def _load_manifest(path: Path) -> KernelManifest:
    data = json.loads(path.read_text())
    services: Dict[str, KernelService] = {}
    for service in data["services"]:
        services[service["id"]] = KernelService(
            id=service["id"],
            name=service["name"],
            category=service["category"],
            version=service["version"],
            interfaces=list(service["interfaces"]),
            dependencies=list(service["dependencies"]),
            optionalDependencies=list(service["optionalDependencies"]),
            artifacts=ArtifactLocation(**service["artifacts"]),
            healthChecks=_as_health_checks(service["healthChecks"]),
            compatibility=_as_compatibility(service["compatibility"]),
            boot=dict(service["boot"]),
            resources=dict(service["resources"]),
        )

    profiles = {
        name: Profile(**profile_data) for name, profile_data in data.get("profiles", {}).items()
    }

    return KernelManifest(
        version=data["version"],
        schemaVersion=data["schemaVersion"],
        generatedAt=data["generatedAt"],
        targets=list(data["targets"]),
        services=services,
        profiles=profiles,
    )


def load_manifest(path: Optional[Path] = None) -> KernelManifest:
    """Load the kernel manifest from disk.

    Parameters
    ----------
    path:
        Optional explicit path to the manifest. Defaults to the canonical manifest
        within this package when not supplied.
    """

    manifest_path = path or _MANIFEST_PATH
    manifest = _load_manifest(manifest_path)

    expected_targets = {"linux", "macos", "container"}
    missing_targets = expected_targets.difference(manifest.targets)
    if missing_targets:
        raise ValueError(f"Manifest missing expected targets: {sorted(missing_targets)}")

    manifest.validate_targets(expected_targets)
    return manifest
