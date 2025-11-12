# Session Continuity & Streaming

Multi-device workflows stay in sync through the `workflow` crate's broadcast stream and the shared session clients in `ui/shared/src/session/`.

1. `workflow::WorkflowEngine` emits `WorkflowEvent` updates (state, stage, resume tokens).
2. `server/ui_api` exposes `/ui/pages/:id/events` (WebSocket) and `UiSchemaService.StreamEvents` (gRPC) to relay those updates.
3. Clients instantiate `SessionContinuityClient` to subscribe and push resume tokens back to the server.

## Resume Flow
- When a stage completes the engine emits a `ResumeOffered` event with a checkpoint.
- The server promotes the event to `schema::RealTimeEvent` and forwards it to the client.
- The client displays the resume call-to-action (see `SchemaDrivenRenderer`) and can invoke `resumeWorkflow` to refresh local state.

## Streaming Requirements
- WebSocket payloads must serialize to JSON using the schema's `RealTimeEvent` contract.
- gRPC streaming uses the protobuf definition in `server/ui_api/proto/ui_schema.proto`.
- Resume tokens expire four hours after issuance and must be re-requested from the server when stale.
