"""Chat workspace endpoints bridging UI and agents."""
from __future__ import annotations

from fastapi import APIRouter
from pydantic import BaseModel

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class ChatMessage(BaseModel):
    """Inbound chat payload."""

    message: str


class ChatResponse(BaseModel):
    """Normalized chat response."""

    reply: str


@router.post("/message", response_model=ChatResponse)
async def send_message(body: ChatMessage) -> ChatResponse:
    await GLOBAL_EVENT_BUS.publish(
        "chat",
        {"type": "chat_command", "message": body.message},
    )
    reply = f"Acknowledged command '{body.message}'."
    return ChatResponse(reply=reply)
