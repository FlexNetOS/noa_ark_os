use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::Serialize_repr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetKind {
    WorkspaceHeader,
    WorkspaceSession,
    WorkspaceBoard,
    WorkspaceAnalytics,
    WorkspaceActivity,
    WorkspaceAssist,
    WorkspaceIntegrations,
    WorkspacePresence,
    LayoutRegion,
    LayoutSection,
    CtaPrimary,
    FormSignin,
}

#[derive(Debug, Clone, Serialize_repr, Deserialize)]
#[repr(u8)]
pub enum LayoutSlot {
    Header = 1,
    Main = 2,
    Footer = 3,
}

impl fmt::Display for LayoutSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutSlot::Header => write!(f, "Header"),
            LayoutSlot::Main => write!(f, "Main"),
            LayoutSlot::Footer => write!(f, "Footer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSchema {
    pub id: String,
    pub kind: WidgetKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRegion {
    pub id: String,
    pub layout: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<LayoutSlot>,
    pub widgets: Vec<WidgetSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tokens_version: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub accessibility_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSchema {
    pub id: String,
    pub version: String,
    pub kind: String,
    pub metadata: PageMetadata,
    pub regions: Vec<LayoutRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeEvent {
    pub event_type: String,
    pub workflow_id: String,
    #[serde(default)]
    pub payload: Value,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeToken {
    pub workflow_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    pub checkpoint: String,
    pub issued_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageEnvelope {
    pub schema: PageSchema,
    #[serde(default)]
    pub realtime: Vec<RealTimeEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_token: Option<ResumeToken>,
}

impl PageEnvelope {
    pub fn with_sample(page_id: &str) -> Self {
        Self {
            schema: PageSchema {
                id: page_id.to_string(),
                version: "2024.05".into(),
                kind: "workspace".into(),
                metadata: PageMetadata {
                    title: "Vibe Kanban Control Hub".into(),
                    description: Some("Server-driven schema delivered by noa_ui_api".into()),
                    tokens_version: "0.1.0".into(),
                    created_at: "2024-05-01T00:00:00.000Z".into(),
                    updated_at: "2024-05-15T00:00:00.000Z".into(),
                    accessibility_notes: vec![
                        "ARIA labels provided for all actionable widgets".into(),
                        "Color contrast adheres to WCAG AA".into(),
                    ],
                },
                regions: vec![
                    LayoutRegion {
                        id: Uuid::new_v4().to_string(),
                        layout: "surface".into(),
                        columns: None,
                        gap: None,
                        surface: Some("surface.glass".into()),
                        slot: Some(LayoutSlot::Header),
                        widgets: vec![WidgetSchema {
                            id: "header.primary".into(),
                            kind: WidgetKind::WorkspaceHeader,
                            variant: None,
                            props: None,
                            component: Some("WorkspaceHeader".into()),
                        }],
                    },
                    LayoutRegion {
                        id: Uuid::new_v4().to_string(),
                        layout: "stack".into(),
                        columns: None,
                        gap: Some("1.5rem".into()),
                        surface: Some("surface.glass".into()),
                        slot: Some(LayoutSlot::Main),
                        widgets: vec![
                            WidgetSchema {
                                id: "workspace.switcher".into(),
                                kind: WidgetKind::WorkspaceSession,
                                variant: None,
                                props: None,
                                component: Some("WorkspaceSwitcher".into()),
                            },
                            WidgetSchema {
                                id: "integrations".into(),
                                kind: WidgetKind::WorkspaceIntegrations,
                                variant: None,
                                props: None,
                                component: Some("IntegrationStatus".into()),
                            },
                        ],
                    },
                    LayoutRegion {
                        id: Uuid::new_v4().to_string(),
                        layout: "stack".into(),
                        columns: None,
                        gap: Some("1.5rem".into()),
                        surface: Some("surface.primary".into()),
                        slot: Some(LayoutSlot::Main),
                        widgets: vec![WidgetSchema {
                            id: "board".into(),
                            kind: WidgetKind::WorkspaceBoard,
                            variant: Some("kanban".into()),
                            props: None,
                            component: Some("BoardShell".into()),
                        }],
                    },
                    LayoutRegion {
                        id: Uuid::new_v4().to_string(),
                        layout: "stack".into(),
                        columns: None,
                        gap: Some("1.5rem".into()),
                        surface: Some("surface.glass".into()),
                        slot: Some(LayoutSlot::Main),
                        widgets: vec![
                            WidgetSchema {
                                id: "presence".into(),
                                kind: WidgetKind::WorkspacePresence,
                                variant: None,
                                props: None,
                                component: Some("PresenceBar".into()),
                            },
                            WidgetSchema {
                                id: "assist".into(),
                                kind: WidgetKind::WorkspaceAssist,
                                variant: None,
                                props: None,
                                component: Some("AssistPanel".into()),
                            },
                            WidgetSchema {
                                id: "analytics".into(),
                                kind: WidgetKind::WorkspaceAnalytics,
                                variant: None,
                                props: None,
                                component: Some("AnalyticsPanel".into()),
                            },
                            WidgetSchema {
                                id: "activity".into(),
                                kind: WidgetKind::WorkspaceActivity,
                                variant: None,
                                props: None,
                                component: Some("ActivityTimeline".into()),
                            },
                        ],
                    },
                ],
            },
            realtime: vec![],
            resume_token: Some(ResumeToken {
                workflow_id: "workspace-sync".into(),
                stage_id: Some("board-load".into()),
                checkpoint: "kanban/snapshot/last".into(),
                issued_at: "2024-05-20T08:00:00.000Z".into(),
                expires_at: "2024-05-20T12:00:00.000Z".into(),
            }),
        }
    }
}
