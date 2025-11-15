#!/usr/bin/env python3
"""Restore archived assets from text-armored snapshots.

The loader understands the JSON snapshot created for ``scripts/triage_analyzer.py``.
It keeps the workflow offline-friendly by avoiding binary blobs in git history
while still letting developers reconstruct the original artifact on demand.
"""
from __future__ import annotations

import argparse
import base64
import json
from pathlib import Path
from typing import Any


def _decode_payload(data: dict[str, Any]) -> bytes:
    encoding = data.get("encoding", "utf-8")
    raw = data["content"]
    if encoding == "base64":
        if not isinstance(raw, str):
            raise ValueError(f"Expected string content for {encoding} encoding, got {type(raw).__name__}")
        return base64.b64decode(raw.encode("ascii"))
    if encoding == "utf-8":
        return raw.encode("utf-8")
    raise ValueError(f"Unsupported encoding: {encoding}")


def _apply_format(data: dict[str, Any], payload: bytes) -> bytes:
    fmt = data.get("format", "plain")
    if fmt == "plain":
        return payload
    raise ValueError(
        f"Unsupported archive format: {fmt}. Only 'plain' is currently supported."
    )


def restore(archive_path: Path, destination: Path | None = None) -> Path:
    snapshot = json.loads(archive_path.read_text(encoding="utf-8"))
    output_path = destination or Path(snapshot.get("original_path", archive_path.stem))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    payload = _decode_payload(snapshot)
    contents = _apply_format(snapshot, payload)
    output_path.write_bytes(contents)
    return output_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path, help="Path to the JSON snapshot to restore")
    parser.add_argument(
        "--output",
        type=Path,
        help="Optional destination path. Defaults to the original_path stored in the snapshot.",
    )
    args = parser.parse_args()
    output_path = restore(args.archive, args.output)
    print(f"Restored {args.archive} -> {output_path}")


if __name__ == "__main__":
    main()
