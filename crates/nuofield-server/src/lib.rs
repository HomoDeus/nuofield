#![deny(unsafe_code)]

//! HTTP runtime for the first NuoField collaboration loop.

use std::{collections::HashMap, path::Path, sync::Arc};

use axum::{
    extract::{DefaultBodyLimit, Path as AxumPath, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use nuofield_core::{DomainError, Event, NewEvent, WorkspaceId, WorkspaceProjection};
use nuofield_store::{AuditRecord, JsonlStore, StoreError};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<RuntimeState>>,
}

#[derive(Debug)]
struct RuntimeState {
    store: JsonlStore,
    workspaces: HashMap<WorkspaceId, WorkspaceProjection>,
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("stored domain event is invalid: {0}")]
    Domain(#[from] DomainError),
}

impl AppState {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StateError> {
        let store = JsonlStore::open(path)?;
        let mut workspaces = HashMap::<WorkspaceId, WorkspaceProjection>::new();

        for record in store.records() {
            workspaces
                .entry(record.event.workspace_id)
                .or_default()
                .accept(&record.event)?;
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(RuntimeState { store, workspaces })),
        })
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/readyz", get(ready))
        .route("/v1/events", post(append_event))
        .route("/v1/export", get(export_all))
        .route("/v1/workspaces/{workspace_id}", get(get_workspace))
        .route(
            "/v1/workspaces/{workspace_id}/events",
            get(get_workspace_events),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn ready(State(state): State<AppState>) -> Result<StatusCode, ApiError> {
    let runtime = state.inner.lock().await;
    runtime.store.verify()?;
    Ok(StatusCode::NO_CONTENT)
}

async fn append_event(
    State(state): State<AppState>,
    Json(new_event): Json<NewEvent>,
) -> Result<(StatusCode, Json<AuditRecord>), ApiError> {
    let mut runtime = state.inner.lock().await;
    let projection = runtime
        .workspaces
        .get(&new_event.workspace_id)
        .cloned()
        .unwrap_or_default();
    projection.validate(&new_event)?;

    let event = Event::from_new(new_event);
    let record = runtime.store.append(event.clone())?;
    runtime
        .workspaces
        .entry(event.workspace_id)
        .or_default()
        .apply(&event);

    Ok((StatusCode::CREATED, Json(record)))
}

async fn get_workspace(
    State(state): State<AppState>,
    AxumPath(workspace_id): AxumPath<String>,
) -> Result<Json<WorkspaceProjection>, ApiError> {
    let workspace_id = parse_workspace_id(&workspace_id)?;
    let runtime = state.inner.lock().await;
    let workspace = runtime
        .workspaces
        .get(&workspace_id)
        .cloned()
        .ok_or(ApiError::NotFound)?;
    Ok(Json(workspace))
}

async fn get_workspace_events(
    State(state): State<AppState>,
    AxumPath(workspace_id): AxumPath<String>,
) -> Result<Json<Vec<AuditRecord>>, ApiError> {
    let workspace_id = parse_workspace_id(&workspace_id)?;
    let runtime = state.inner.lock().await;
    if !runtime.workspaces.contains_key(&workspace_id) {
        return Err(ApiError::NotFound);
    }
    Ok(Json(runtime.store.records_for_workspace(workspace_id)))
}

async fn export_all(State(state): State<AppState>) -> Json<Vec<AuditRecord>> {
    let runtime = state.inner.lock().await;
    Json(runtime.store.records().to_vec())
}

fn parse_workspace_id(value: &str) -> Result<WorkspaceId, ApiError> {
    value
        .parse()
        .map_err(|_| ApiError::BadRequest("workspace_id must be a UUID".into()))
}

#[derive(Debug, Error)]
enum ApiError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("{0}")]
    BadRequest(String),
    #[error("resource not found")]
    NotFound,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Domain(error) => (StatusCode::CONFLICT, error.to_string()),
            Self::Store(error) => {
                tracing::error!(%error, "storage operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage operation failed".into(),
                )
            }
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFound => (StatusCode::NOT_FOUND, "resource not found".into()),
        };
        (status, Json(ErrorBody { error: message })).into_response()
    }
}
