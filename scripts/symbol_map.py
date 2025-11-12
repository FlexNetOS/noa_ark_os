#!/usr/bin/env python3
"""Generate a trivial symbol map using Tree-sitter placeholders."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def collect_symbols(root: Path) -> dict[str, list[str]]:
    symbols: dict[str, list[str]] = {}
    for path in root.rglob("*.rs"):
        symbols[str(path)] = ["fn_placeholder", "struct_placeholder"]
    return symbols


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--output", type=Path, default=Path("out/ci/symbols.json"))
    args = parser.parse_args()

    symbols = collect_symbols(args.root)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(symbols, indent=2))


if __name__ == "__main__":
    main()
