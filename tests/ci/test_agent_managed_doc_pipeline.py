from __future__ import annotations

from pathlib import Path

import pytest

yaml = pytest.importorskip("yaml")

REPO_ROOT = Path(__file__).resolve().parents[2]
DOC_PIPELINE_PATH = REPO_ROOT / "cicd/pipelines/doc-refresh.yaml"
SELF_HEAL_PIPELINE_PATH = REPO_ROOT / "cicd/pipelines/agent-self-heal.yaml"


def _load_yaml(path: Path):
    return yaml.safe_load(path.read_text())


def test_doc_refresh_pipeline_uses_agent_checkpoints():
    pipeline = _load_yaml(DOC_PIPELINE_PATH)

    assert pipeline["agent_roles"]["planner"] == "PlanAgent"
    assert pipeline["agent_roles"]["verifier"] == "ReviewAgent"
    assert "agent_checkpoints" in pipeline
    checkpoint_roles = {entry["role"] for entry in pipeline["agent_checkpoints"]}
    assert {"planner", "verifier"}.issubset(checkpoint_roles)

    jobs = pipeline["jobs"]
    assert "agent-self-heal" in jobs
    heal_script = jobs["agent-self-heal"]["script"]
    assert any("workspace_optimization.sh" in step for step in heal_script)
    assert any("services/gateway/self_heal.py" in step for step in heal_script)

    doc_checkpoints = jobs["doc-refresh"].get("checkpoints", {}).get("required", [])
    verifier_targets = [item["artifact"] for item in doc_checkpoints]
    assert "docs/reports/AGENT_DEPLOYMENT_OUTCOMES.md" in verifier_targets


def test_agent_self_heal_pipeline_declares_roles():
    pipeline = _load_yaml(SELF_HEAL_PIPELINE_PATH)

    assert pipeline["agent_roles"]["operator"] == "OperationsBoardAgent"
    assert len(pipeline.get("agent_checkpoints", [])) >= 3

    for stage in pipeline.get("stages", []):
        for task in stage.get("tasks", []):
            assert "agent_role" in task, f"Task in stage {stage['name']} is missing agent_role"
            assert task["agent"].startswith("role::"), "Tasks should reference agent roles explicitly"
            if task["agent_role"] == "verifier":
                telemetry_path = task.get("parameters", {}).get("telemetry")
                assert (
                    telemetry_path == "build_output/telemetry/self-heal-metrics.json"
                ), "Verifier should review telemetry exported beside the self-heal summary"
