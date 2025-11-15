from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional

from core.kernel.manifest import KernelManifest, load_manifest
from services.gateway.service import Gateway, build_default_config

NAVIGATION_FILE = Path("ui/shared/src/shell/navigation.ts")


def load_entry_points(path: Path = NAVIGATION_FILE) -> List[str]:
    ids = []
    pattern = re.compile(r"id:\s*\"([^\"]+)\"")
    for line in path.read_text().splitlines():
        match = pattern.search(line)
        if match:
            ids.append(match.group(1))
    return ids


@dataclass
class FeatureToggle:
    name: str
    license_required: bool
    dependencies: List[str]
    enabled: bool = False
    readiness_signal: Optional[str] = None


class FeatureToggleManager:
    def __init__(self) -> None:
        self._toggles: Dict[str, FeatureToggle] = {}
        self._licenses: Dict[str, str] = {}

    def register(self, toggle: FeatureToggle) -> None:
        self._toggles[toggle.name] = toggle

    def grant_license(self, toggle_name: str, license_id: str) -> None:
        self._licenses[toggle_name] = license_id

    def enable(self, toggle_name: str, readiness_signal: Optional[str] = None) -> None:
        toggle = self._toggles[toggle_name]
        if toggle.license_required and toggle_name not in self._licenses:
            raise PermissionError(f"License missing for feature {toggle_name}")
        toggle.enabled = True
        toggle.readiness_signal = readiness_signal

    def is_enabled(self, toggle_name: str) -> bool:
        return self._toggles.get(toggle_name, FeatureToggle(toggle_name, False, [])).enabled

    def export(self) -> Dict[str, Dict[str, object]]:
        return {
            name: {
                "enabled": toggle.enabled,
                "license_required": toggle.license_required,
                "dependencies": toggle.dependencies,
                "readiness_signal": toggle.readiness_signal,
            }
            for name, toggle in self._toggles.items()
        }


@dataclass
class EcosystemBundle:
    manifest: KernelManifest
    toggle_manager: FeatureToggleManager
    gateway: Gateway
    entry_points: List[str]
    features: List[FeatureToggle] = field(default_factory=list)

    def register_feature(self, feature: FeatureToggle) -> None:
        self.features.append(feature)
        self.toggle_manager.register(feature)

    def validate_dependencies(self) -> List[str]:
        required_services = {"ui-shell", "runtime-manager", "gateway"}
        missing = [service for service in required_services if service not in self.manifest.services]
        return missing

    def install(self, license_grants: Optional[Dict[str, str]] = None) -> None:
        missing = self.validate_dependencies()
        if missing:
            raise RuntimeError(f"Missing dependencies: {missing}")

        for feature in self.features:
            if license_grants and feature.name in license_grants:
                self.toggle_manager.grant_license(feature.name, license_grants[feature.name])
            readiness = f"{feature.name}-ready"
            self.toggle_manager.enable(feature.name, readiness_signal=readiness)

    def export_state(self, destination: Path) -> None:
        destination.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "features": self.toggle_manager.export(),
            "entry_points": self.entry_points,
        }
        destination.write_text(json.dumps(payload, indent=2, sort_keys=True))


def create_default_bundle(manifest_path: Optional[Path] = None) -> EcosystemBundle:
    manifest = load_manifest(manifest_path)
    config = build_default_config(manifest_path)
    gateway = Gateway(config=config)
    toggle_manager = FeatureToggleManager()
    entry_points = load_entry_points()
    bundle = EcosystemBundle(manifest, toggle_manager, gateway, entry_points)
    bundle.register_feature(FeatureToggle("workflow-library", True, ["ui-shell", "runtime-manager"]))
    bundle.register_feature(FeatureToggle("partner-integrations", True, ["gateway", "runtime-manager"]))
    bundle.register_feature(FeatureToggle("analytics-pack", False, ["observability"]))
    return bundle
