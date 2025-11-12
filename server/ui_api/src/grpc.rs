use std::pin::Pin;

use axum::async_trait;
use chrono::{DateTime, Utc};
use futures::Stream;
use prost_types::{ListValue, Struct, Timestamp};
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};

use crate::schema::{LayoutSlot, PageEnvelope, RealTimeEvent};
use crate::server::UiApiState;

pub mod proto {
    tonic::include_proto!("noa.ui");
}

pub struct UiSchemaGrpc {
    state: UiApiState,
}

impl UiSchemaGrpc {
    pub fn new(state: UiApiState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl proto::ui_schema_service_server::UiSchemaService for UiSchemaGrpc {
    async fn get_page(
        &self,
        request: Request<proto::PageRequest>,
    ) -> Result<Response<proto::PageEnvelope>, Status> {
        let page_id = request.into_inner().page_id;
        let envelope = {
            let mut pages = self.state.pages().write().await;
            pages
                .entry(page_id.clone())
                .or_insert_with(|| PageEnvelope::with_sample(&page_id))
                .clone()
        };
        let proto_envelope = page_envelope_to_proto(envelope)?;
        Ok(Response::new(proto_envelope))
    }

    type StreamEventsStream =
        Pin<Box<dyn futures::Stream<Item = Result<proto::RealTimeEvent, Status>> + Send>>;

    async fn stream_events(
        &self,
        request: Request<proto::PageRequest>,
    ) -> Result<Response<Self::StreamEventsStream>, Status> {
        let _page_id = request.into_inner().page_id;
        let bridge = {
            let guard = self
                .state
                .session()
                .lock()
                .map_err(|_| Status::internal("session poisoned"))?;
            guard
                .clone()
                .ok_or(Status::unavailable("streaming disabled"))?
        };

        let stream = bridge.forward_events();
        let output = async_stream::try_stream! {
            let mut stream = Box::pin(stream);
            while let Some(event) = stream.as_mut().next().await {
                yield realtime_to_proto(event)?;
            }
        };

        Ok(Response::new(Box::pin(output) as Self::StreamEventsStream))
    }
}

fn page_envelope_to_proto(envelope: PageEnvelope) -> Result<proto::PageEnvelope, Status> {
    let metadata = Some(proto::PageMetadata {
        title: envelope.schema.metadata.title,
        description: envelope.schema.metadata.description.unwrap_or_default(),
        tokens_version: envelope.schema.metadata.tokens_version,
        created_at: Some(timestamp_from_str(&envelope.schema.metadata.created_at)?),
        updated_at: Some(timestamp_from_str(&envelope.schema.metadata.updated_at)?),
        accessibility_notes: envelope.schema.metadata.accessibility_notes,
    });

    let regions = envelope
        .schema
        .regions
        .into_iter()
        .map(|region| -> Result<proto::LayoutRegion, Status> {
            let widgets = region
                .widgets
                .into_iter()
                .map(|widget| {
                    Ok(proto::WidgetSchema {
                        id: widget.id,
                        kind: format!("{:?}", widget.kind),
                        variant: widget.variant.unwrap_or_default(),
                        props: widget
                            .props
                            .map(value_to_struct)
                            .transpose()?,
                    })
                })
                .collect::<Result<Vec<_>, Status>>()?;

            Ok(proto::LayoutRegion {
                id: region.id,
                layout: region.layout,
                columns: region.columns.unwrap_or_default(),
                gap: region.gap.unwrap_or_default(),
                surface: region.surface.unwrap_or_default(),
                slot: region
                    .slot
                    .map(slot_to_string)
                    .unwrap_or_default(),
                widgets,
            })
        })
        .collect::<Result<Vec<_>, Status>>()?;

    let realtime = envelope
        .realtime
        .into_iter()
        .map(realtime_to_proto)
        .collect::<Result<Vec<_>, _>>()?;

    let resume_token = if let Some(token) = envelope.resume_token {
        Some(proto::ResumeToken {
            workflow_id: token.workflow_id,
            stage_id: token.stage_id.unwrap_or_default(),
            checkpoint: token.checkpoint,
            issued_at: Some(timestamp_from_str(&token.issued_at)?),
            expires_at: Some(timestamp_from_str(&token.expires_at)?),
        })
    } else {
        None
    };

    Ok(proto::PageEnvelope {
        schema: Some(proto::PageSchema {
            id: envelope.schema.id,
            version: envelope.schema.version,
            kind: envelope.schema.kind,
            metadata,
            regions,
        }),
        realtime,
        resume_token,
    })
}

fn realtime_to_proto(event: RealTimeEvent) -> Result<proto::RealTimeEvent, Status> {
    Ok(proto::RealTimeEvent {
        event_type: event.event_type,
        workflow_id: event.workflow_id,
        payload: Some(value_to_struct(event.payload)?),
        timestamp: Some(timestamp_from_str(&event.timestamp)?),
    })
}

fn timestamp_from_str(value: &str) -> Result<Timestamp, Status> {
    let parsed: DateTime<Utc> = value
        .parse()
        .map_err(|_| Status::internal("invalid timestamp"))?;
    Ok(Timestamp {
        seconds: parsed.timestamp(),
        nanos: parsed.timestamp_subsec_nanos() as i32,
    })
}

fn value_to_struct(value: serde_json::Value) -> Result<Struct, Status> {
    match value {
        serde_json::Value::Object(map) => {
            let fields = map
                .into_iter()
                .map(|(key, value)| Ok((key, value_to_prost_value(value)?)))
                .collect::<Result<_, Status>>()?;
            Ok(Struct { fields })
        }
        _ => Err(Status::internal("invalid payload")),
    }
}

fn value_to_prost_value(value: serde_json::Value) -> Result<prost_types::Value, Status> {
    use prost_types::value::Kind;

    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(value) => Kind::BoolValue(value),
        serde_json::Value::Number(number) => Kind::NumberValue(
            number
                .as_f64()
                .ok_or_else(|| Status::internal("invalid number"))?,
        ),
        serde_json::Value::String(value) => Kind::StringValue(value),
        serde_json::Value::Array(values) => Kind::ListValue(ListValue {
            values: values
                .into_iter()
                .map(value_to_prost_value)
                .collect::<Result<_, Status>>()?,
        }),
        serde_json::Value::Object(map) => Kind::StructValue(Struct {
            fields: map
                .into_iter()
                .map(|(key, value)| Ok((key, value_to_prost_value(value)?)))
                .collect::<Result<_, Status>>()?,
        }),
    };

    Ok(prost_types::Value { kind: Some(kind) })
}

fn slot_to_string(slot: LayoutSlot) -> String {
    slot.to_string()
}
