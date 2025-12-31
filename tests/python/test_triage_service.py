import base64
import json
import subprocess
import sys
from pathlib import Path


def test_event_driven_triage_creates_incident(tmp_path):
    events_dir = tmp_path / "events"
    incidents_dir = tmp_path / "incidents"
    logs_dir = tmp_path / "logs"
    events_dir.mkdir()
    incidents_dir.mkdir()
    logs_dir.mkdir()

    log_path = logs_dir / "run.log"
    log_path.write_text("ERROR lint failure detected\n", encoding="utf-8")

    event = {
        "event_id": "lint-123",
        "log": str(log_path),
        "message": "Lint error in pipeline",
    }
    (events_dir / "lint.json").write_text(json.dumps(event), encoding="utf-8")

    cmd = [
        sys.executable,
        "scripts/triage_analyzer.py",
        "--events",
        str(events_dir),
        "--incident-root",
        str(incidents_dir),
        "--remediation-cli",
        "echo",
        "--run-once",
        "--dry-run",
    ]

    result = subprocess.run(cmd, check=True, capture_output=True, text=True)
    assert "Starting event-driven triage service" in result.stdout

    incident_dirs = list(incidents_dir.glob("*/lint-123"))
    assert incident_dirs, "incident directory not created"
    incident_dir = incident_dirs[0]

    manifest = json.loads((incident_dir / "manifest.json").read_text(encoding="utf-8"))
    assert manifest["classification"]["category"] == "lint"
    assert manifest["policy_decision"]["decision"] == "allow"

    remediation_log = (incident_dir / "remediation.log").read_text(encoding="utf-8")
    assert "echo auto-fix lint" in remediation_log


def test_triage_archive_loader(tmp_path):
    archive_path = Path("archive/2025/11/scripts/triage_analyzer.py.archive.json")
    output_path = tmp_path / "restored_triage.py"

    cmd = [
        sys.executable,
        "tools/archive_utils/autoload.py",
        str(archive_path),
        "--output",
        str(output_path),
    ]
    subprocess.run(cmd, check=True)

    restored = output_path.read_bytes()
    snapshot = json.loads(archive_path.read_text(encoding="utf-8"))
    if snapshot.get("encoding") == "base64":
        expected = base64.b64decode(snapshot["content"].encode("ascii"))
    else:
        expected = snapshot["content"].encode(snapshot.get("encoding", "utf-8"))

    assert restored == expected, "Restored payload does not match archived content"
