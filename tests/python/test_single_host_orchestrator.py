from __future__ import annotations

from pathlib import Path

from server.deploy.single_host import SingleHostOrchestrator


def test_single_host_lifecycle(tmp_path: Path) -> None:
    orchestrator = SingleHostOrchestrator(tmp_path)
    inventory = orchestrator.inventory()
    assert inventory[0].id == "kernel"

    statuses = orchestrator.start()
    assert all(status.ready for status in statuses)

    health = orchestrator.health_check()
    assert all(health.values())

    snapshot = orchestrator.snapshot()
    assert snapshot.exists()

    resources = orchestrator.resource_envelopes()
    assert resources["baseline"]["cpuCores"] >= 8

    orchestrator.stop()
    statuses_after_stop = orchestrator.status()
    assert all(not status.ready for status in statuses_after_stop)
