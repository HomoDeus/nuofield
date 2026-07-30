use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(value).map(Self)
            }
        }
    };
}

uuid_id!(WorkspaceId);
uuid_id!(ActorId);
uuid_id!(TaskId);
uuid_id!(EventId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Human,
    Agent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Actor {
    pub id: ActorId,
    pub display_name: String,
    pub kind: ActorKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    High,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelEndpoint {
    Local,
    DomesticCloud,
    External,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ModelInvocation {
    pub provider: String,
    pub model: String,
    pub endpoint: ModelEndpoint,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Evidence {
    pub kind: String,
    pub uri: String,
    pub digest: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    WorkspaceCreated {
        name: String,
        owner: Actor,
    },
    ActorJoined {
        actor: Actor,
    },
    TaskAssigned {
        task_id: TaskId,
        title: String,
        assignee_id: ActorId,
        risk: RiskLevel,
    },
    TaskApproved {
        task_id: TaskId,
    },
    TaskStarted {
        task_id: TaskId,
    },
    ModelInvocationRecorded {
        task_id: TaskId,
        invocation: ModelInvocation,
    },
    TaskCompleted {
        task_id: TaskId,
        summary: String,
        evidence: Vec<Evidence>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewEvent {
    pub workspace_id: WorkspaceId,
    pub actor_id: ActorId,
    pub payload: EventPayload,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Event {
    pub id: EventId,
    pub workspace_id: WorkspaceId,
    pub actor_id: ActorId,
    pub occurred_at: DateTime<Utc>,
    pub payload: EventPayload,
}

impl Event {
    pub fn from_new(event: NewEvent) -> Self {
        Self {
            id: EventId::new(),
            workspace_id: event.workspace_id,
            actor_id: event.actor_id,
            occurred_at: Utc::now(),
            payload: event.payload,
        }
    }

    pub fn as_new(&self) -> NewEvent {
        NewEvent {
            workspace_id: self.workspace_id,
            actor_id: self.actor_id,
            payload: self.payload.clone(),
        }
    }
}
