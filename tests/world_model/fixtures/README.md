# World Model Test Fixtures

This directory intentionally stays empty at runtime to simulate missing resources
referenced by `tests/world_model/sample_missing_graph.json`. The reconciler should
detect that `ghost.txt` is absent and produce a remediation plan.
