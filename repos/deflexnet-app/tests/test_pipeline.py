"""Smoke tests for the Digest pipeline."""

from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from digest_pipeline import PipelineContext
from run_pipeline import build_pipeline


def test_pipeline_runs(tmp_path: Path) -> None:
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "README.md").write_text("# Sample\n", encoding="utf-8")

    output = tmp_path / "output"
    pipeline = build_pipeline()
    context = PipelineContext(repo_path=repo, output_path=output)
    result = pipeline.run(context)

    assert (output / "profile.json").exists()
    assert result.metadata["files"], "intake should collect files"
    assert any("registrar" in log for log in result.logs)
