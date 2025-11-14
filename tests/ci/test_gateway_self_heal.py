from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "services/gateway/self_heal.py"


def test_gateway_self_heal_executes(tmp_path):
    output_path = tmp_path / "gateway-self-heal.json"
    result = subprocess.run(
        [sys.executable, str(SCRIPT_PATH), "--output", str(output_path)],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stdout + result.stderr

    report = json.loads(result.stdout)
    assert report["status"] == "ok"
    assert output_path.exists()

    persisted = json.loads(output_path.read_text())
    assert persisted["results"]
    assert Path(persisted["telemetry"]).name == "self-heal-metrics.json"
