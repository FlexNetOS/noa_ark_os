"""Lightweight event bus powering WebSocket fan-out."""
from __future__ import annotations

from collections import defaultdict
from typing import Any, AsyncIterator, Dict, List

from asyncio import Queue


class EventBus:
    """Async pub/sub bus used by websocket handlers."""

    def __init__(self) -> None:
        self._subscribers: Dict[str, List[Queue]] = defaultdict(list)

    async def publish(self, channel: str, payload: Any) -> None:
        for queue in self._subscribers.get(channel, []):
            await queue.put(payload)

    async def subscribe(self, channel: str) -> AsyncIterator[Any]:
        queue: Queue = Queue()
        self._subscribers[channel].append(queue)

        try:
            while True:
                payload = await queue.get()
                yield payload
        finally:
            self._subscribers[channel].remove(queue)


GLOBAL_EVENT_BUS = EventBus()
