#![deny(unsafe_code)]

//! Zero-I/O domain model and policy enforcement for NuoField.

mod event;
mod projection;

pub use event::{
    Actor, ActorId, ActorKind, Event, EventId, EventPayload, Evidence, ModelEndpoint,
    ModelInvocation, NewEvent, RiskLevel, TaskId, WorkspaceId,
};
pub use projection::{DomainError, Task, TaskStatus, WorkspaceProjection};
