use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    Actor, ActorId, ActorKind, Event, EventPayload, Evidence, ModelInvocation, NewEvent, RiskLevel,
    TaskId, WorkspaceId,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    AwaitingApproval,
    Ready,
    Running,
    Completed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub assignee_id: ActorId,
    pub risk: RiskLevel,
    pub status: TaskStatus,
    pub model_invocations: Vec<ModelInvocation>,
    pub summary: Option<String>,
    pub evidence: Vec<Evidence>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkspaceProjection {
    pub id: Option<WorkspaceId>,
    pub name: Option<String>,
    pub owner_id: Option<ActorId>,
    pub actors: HashMap<ActorId, Actor>,
    pub tasks: HashMap<TaskId, Task>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DomainError {
    #[error("workspace already exists")]
    WorkspaceAlreadyExists,
    #[error("workspace has not been created")]
    WorkspaceNotCreated,
    #[error("event belongs to a different workspace")]
    WrongWorkspace,
    #[error("actor is not a workspace member")]
    UnknownActor,
    #[error("actor already exists")]
    ActorAlreadyExists,
    #[error("operation requires a human actor")]
    HumanRequired,
    #[error("operation requires the workspace owner")]
    OwnerRequired,
    #[error("task already exists")]
    TaskAlreadyExists,
    #[error("task does not exist")]
    UnknownTask,
    #[error("task must be assigned to an agent")]
    AgentAssigneeRequired,
    #[error("only the assigned agent can perform this operation")]
    AssigneeRequired,
    #[error("task is not in the required state")]
    InvalidTaskState,
    #[error("a required text field is empty")]
    EmptyText,
    #[error("task completion requires evidence")]
    EvidenceRequired,
}

impl WorkspaceProjection {
    pub fn validate(&self, event: &NewEvent) -> Result<(), DomainError> {
        match &event.payload {
            EventPayload::WorkspaceCreated { name, owner } => {
                if self.id.is_some() {
                    return Err(DomainError::WorkspaceAlreadyExists);
                }
                if name.trim().is_empty() || owner.display_name.trim().is_empty() {
                    return Err(DomainError::EmptyText);
                }
                if owner.kind != ActorKind::Human || owner.id != event.actor_id {
                    return Err(DomainError::HumanRequired);
                }
                Ok(())
            }
            EventPayload::ActorJoined { actor } => {
                self.require_workspace(event.workspace_id)?;
                self.require_owner(event.actor_id)?;
                if actor.display_name.trim().is_empty() {
                    return Err(DomainError::EmptyText);
                }
                if self.actors.contains_key(&actor.id) {
                    return Err(DomainError::ActorAlreadyExists);
                }
                Ok(())
            }
            EventPayload::TaskAssigned {
                task_id,
                title,
                assignee_id,
                ..
            } => {
                self.require_workspace(event.workspace_id)?;
                self.require_human(event.actor_id)?;
                if title.trim().is_empty() {
                    return Err(DomainError::EmptyText);
                }
                if self.tasks.contains_key(task_id) {
                    return Err(DomainError::TaskAlreadyExists);
                }
                let assignee = self
                    .actors
                    .get(assignee_id)
                    .ok_or(DomainError::UnknownActor)?;
                if assignee.kind != ActorKind::Agent {
                    return Err(DomainError::AgentAssigneeRequired);
                }
                Ok(())
            }
            EventPayload::TaskApproved { task_id } => {
                self.require_workspace(event.workspace_id)?;
                self.require_human(event.actor_id)?;
                let task = self.task(*task_id)?;
                if task.status != TaskStatus::AwaitingApproval {
                    return Err(DomainError::InvalidTaskState);
                }
                Ok(())
            }
            EventPayload::TaskStarted { task_id } => {
                self.require_workspace(event.workspace_id)?;
                let task = self.task(*task_id)?;
                self.require_assignee(task, event.actor_id)?;
                if task.status != TaskStatus::Ready {
                    return Err(DomainError::InvalidTaskState);
                }
                Ok(())
            }
            EventPayload::ModelInvocationRecorded {
                task_id,
                invocation,
            } => {
                self.require_workspace(event.workspace_id)?;
                let task = self.task(*task_id)?;
                self.require_assignee(task, event.actor_id)?;
                if task.status != TaskStatus::Running {
                    return Err(DomainError::InvalidTaskState);
                }
                if invocation.provider.trim().is_empty() || invocation.model.trim().is_empty() {
                    return Err(DomainError::EmptyText);
                }
                Ok(())
            }
            EventPayload::TaskCompleted {
                task_id,
                summary,
                evidence,
            } => {
                self.require_workspace(event.workspace_id)?;
                let task = self.task(*task_id)?;
                self.require_assignee(task, event.actor_id)?;
                if task.status != TaskStatus::Running {
                    return Err(DomainError::InvalidTaskState);
                }
                if summary.trim().is_empty() {
                    return Err(DomainError::EmptyText);
                }
                if evidence.is_empty() {
                    return Err(DomainError::EvidenceRequired);
                }
                if evidence
                    .iter()
                    .any(|item| item.kind.trim().is_empty() || item.uri.trim().is_empty())
                {
                    return Err(DomainError::EmptyText);
                }
                Ok(())
            }
        }
    }

    pub fn apply(&mut self, event: &Event) {
        match &event.payload {
            EventPayload::WorkspaceCreated { name, owner } => {
                self.id = Some(event.workspace_id);
                self.name = Some(name.clone());
                self.owner_id = Some(owner.id);
                self.actors.insert(owner.id, owner.clone());
            }
            EventPayload::ActorJoined { actor } => {
                self.actors.insert(actor.id, actor.clone());
            }
            EventPayload::TaskAssigned {
                task_id,
                title,
                assignee_id,
                risk,
            } => {
                let status = match risk {
                    RiskLevel::Low => TaskStatus::Ready,
                    RiskLevel::High => TaskStatus::AwaitingApproval,
                };
                self.tasks.insert(
                    *task_id,
                    Task {
                        id: *task_id,
                        title: title.clone(),
                        assignee_id: *assignee_id,
                        risk: *risk,
                        status,
                        model_invocations: Vec::new(),
                        summary: None,
                        evidence: Vec::new(),
                    },
                );
            }
            EventPayload::TaskApproved { task_id } => {
                if let Some(task) = self.tasks.get_mut(task_id) {
                    task.status = TaskStatus::Ready;
                }
            }
            EventPayload::TaskStarted { task_id } => {
                if let Some(task) = self.tasks.get_mut(task_id) {
                    task.status = TaskStatus::Running;
                }
            }
            EventPayload::ModelInvocationRecorded {
                task_id,
                invocation,
            } => {
                if let Some(task) = self.tasks.get_mut(task_id) {
                    task.model_invocations.push(invocation.clone());
                }
            }
            EventPayload::TaskCompleted {
                task_id,
                summary,
                evidence,
            } => {
                if let Some(task) = self.tasks.get_mut(task_id) {
                    task.status = TaskStatus::Completed;
                    task.summary = Some(summary.clone());
                    task.evidence.clone_from(evidence);
                }
            }
        }
    }

    pub fn accept(&mut self, event: &Event) -> Result<(), DomainError> {
        self.validate(&event.as_new())?;
        self.apply(event);
        Ok(())
    }

    fn require_workspace(&self, workspace_id: WorkspaceId) -> Result<(), DomainError> {
        let id = self.id.ok_or(DomainError::WorkspaceNotCreated)?;
        if id != workspace_id {
            return Err(DomainError::WrongWorkspace);
        }
        Ok(())
    }

    fn require_owner(&self, actor_id: ActorId) -> Result<(), DomainError> {
        if self.owner_id != Some(actor_id) {
            return Err(DomainError::OwnerRequired);
        }
        Ok(())
    }

    fn require_human(&self, actor_id: ActorId) -> Result<(), DomainError> {
        let actor = self
            .actors
            .get(&actor_id)
            .ok_or(DomainError::UnknownActor)?;
        if actor.kind != ActorKind::Human {
            return Err(DomainError::HumanRequired);
        }
        Ok(())
    }

    fn task(&self, task_id: TaskId) -> Result<&Task, DomainError> {
        self.tasks.get(&task_id).ok_or(DomainError::UnknownTask)
    }

    fn require_assignee(&self, task: &Task, actor_id: ActorId) -> Result<(), DomainError> {
        if task.assignee_id != actor_id {
            return Err(DomainError::AssigneeRequired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, EventPayload, NewEvent};

    fn event(workspace_id: WorkspaceId, actor_id: ActorId, payload: EventPayload) -> Event {
        Event::from_new(NewEvent {
            workspace_id,
            actor_id,
            payload,
        })
    }

    #[test]
    fn high_risk_task_requires_human_approval_and_agent_evidence() {
        let workspace_id = WorkspaceId::new();
        let owner_id = ActorId::new();
        let agent_id = ActorId::new();
        let task_id = TaskId::new();
        let mut projection = WorkspaceProjection::default();

        projection
            .accept(&event(
                workspace_id,
                owner_id,
                EventPayload::WorkspaceCreated {
                    name: "Sovereign workspace".into(),
                    owner: Actor {
                        id: owner_id,
                        display_name: "Owner".into(),
                        kind: ActorKind::Human,
                    },
                },
            ))
            .expect("workspace should be created");
        projection
            .accept(&event(
                workspace_id,
                owner_id,
                EventPayload::ActorJoined {
                    actor: Actor {
                        id: agent_id,
                        display_name: "Release agent".into(),
                        kind: ActorKind::Agent,
                    },
                },
            ))
            .expect("agent should join");
        projection
            .accept(&event(
                workspace_id,
                owner_id,
                EventPayload::TaskAssigned {
                    task_id,
                    title: "Publish a release".into(),
                    assignee_id: agent_id,
                    risk: RiskLevel::High,
                },
            ))
            .expect("task should be assigned");

        let start = event(
            workspace_id,
            agent_id,
            EventPayload::TaskStarted { task_id },
        );
        assert_eq!(
            projection.accept(&start),
            Err(DomainError::InvalidTaskState)
        );

        projection
            .accept(&event(
                workspace_id,
                owner_id,
                EventPayload::TaskApproved { task_id },
            ))
            .expect("human approval should unblock the task");
        projection
            .accept(&start)
            .expect("assigned agent should start");

        let no_evidence = event(
            workspace_id,
            agent_id,
            EventPayload::TaskCompleted {
                task_id,
                summary: "Published".into(),
                evidence: Vec::new(),
            },
        );
        assert_eq!(
            projection.accept(&no_evidence),
            Err(DomainError::EvidenceRequired)
        );

        projection
            .accept(&event(
                workspace_id,
                agent_id,
                EventPayload::TaskCompleted {
                    task_id,
                    summary: "Published".into(),
                    evidence: vec![Evidence {
                        kind: "url".into(),
                        uri: "https://example.invalid/release".into(),
                        digest: None,
                    }],
                },
            ))
            .expect("evidence should close the task");

        assert_eq!(
            projection.tasks.get(&task_id).map(|task| task.status),
            Some(TaskStatus::Completed)
        );
    }
}
