#!/usr/bin/env python3
"""Validate registry documents under .workspace/registry.

The script performs lightweight schema enforcement without relying on
external packages so it can run inside pre-commit hooks and CI jobs.
"""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Tuple

REGISTRY_RELATIVE_PATH = Path(".workspace/registry")
SCHEMA_FILENAME = "registry.schema.json"
REQUIRED_FIELDS = {"id", "name", "version", "files", "dependencies", "owners"}
SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$")


@dataclass
class RegistryDocument:
    path: Path
    data: dict


class ValidationError(Exception):
    pass


def load_documents(root: Path) -> List[RegistryDocument]:
    registry_dir = root / REGISTRY_RELATIVE_PATH
    if not registry_dir.exists():
        raise ValidationError(f"registry directory '{registry_dir}' does not exist")

    documents: List[RegistryDocument] = []
    for json_path in sorted(registry_dir.glob("*.json")):
        if json_path.name == SCHEMA_FILENAME:
            continue
        try:
            data = json.loads(json_path.read_text())
        except json.JSONDecodeError as exc:  # pragma: no cover - surfaced in CLI output
            raise ValidationError(f"{json_path}: invalid JSON ({exc})") from exc
        documents.append(RegistryDocument(json_path, data))

    if not documents:
        raise ValidationError(f"no registry documents found in '{registry_dir}'")

    return documents


def validate_documents(documents: List[RegistryDocument], repo_root: Path) -> Tuple[int, int]:
    owners: Dict[str, Tuple[Path, dict]] = {}
    components: Dict[str, Tuple[Path, dict]] = {}

    for doc in documents:
        for owner in doc.data.get("owners", []):
            owner_id = owner.get("id")
            if not owner_id:
                raise ValidationError(f"{doc.path}: owner entry missing 'id'")
            previous = owners.get(owner_id)
            if previous and previous[1] != owner:
                raise ValidationError(
                    f"{doc.path}: conflicting owner definition for '{owner_id}' (first defined in {previous[0]})"
                )
            owners[owner_id] = (doc.path, owner)

    for doc in documents:
        for component in doc.data.get("components", []):
            missing = REQUIRED_FIELDS - component.keys()
            if missing:
                raise ValidationError(
                    f"{doc.path}: component '{component.get('id', '<unknown>')}' missing fields: {sorted(missing)}"
                )

            component_id = component["id"]
            previous = components.get(component_id)
            if previous:
                raise ValidationError(
                    f"{doc.path}: duplicate component id '{component_id}' also defined in {previous[0]}"
                )

            if not SEMVER_RE.match(component["version"]):
                raise ValidationError(
                    f"{doc.path}: component '{component_id}' has non-semver version '{component['version']}'"
                )

            for owner_id in component["owners"]:
                if owner_id not in owners:
                    raise ValidationError(
                        f"{doc.path}: component '{component_id}' references unknown owner '{owner_id}'"
                    )

            for file_path in component["files"]:
                candidate = repo_root / file_path
                if not candidate.exists():
                    raise ValidationError(
                        f"{doc.path}: component '{component_id}' references missing file '{file_path}'"
                    )

            components[component_id] = (doc.path, component)

    for component_id, (doc_path, component) in components.items():
        for dependency in component.get("dependencies", []):
            if not dependency:
                raise ValidationError(
                    f"{doc_path}: component '{component_id}' has an empty or missing dependency entry"
                )
            if dependency not in components:
                raise ValidationError(
                    f"{doc_path}: component '{component_id}' depends on unknown component '{dependency}'"
                )

    return len(components), len(owners)


def main() -> int:
    repo_root = Path(__file__).resolve().parents[2]
    try:
        documents = load_documents(repo_root)
        component_count, owner_count = validate_documents(documents, repo_root)
    except ValidationError as exc:
        print(f"registry validation failed: {exc}", file=sys.stderr)
        return 1

    print(
        "registry validation passed: "
        f"{component_count} component(s), {owner_count} owner(s) checked"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
