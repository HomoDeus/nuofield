#![deny(unsafe_code)]

//! Append-only JSONL storage with a tamper-evident SHA-256 hash chain.

use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use nuofield_core::{Event, WorkspaceId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuditRecord {
    pub sequence: u64,
    pub previous_hash: String,
    pub hash: String,
    pub event: Event,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("stored event JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("audit chain is invalid at sequence {sequence}: {reason}")]
    Integrity { sequence: u64, reason: String },
    #[error("audit sequence overflow")]
    SequenceOverflow,
}

#[derive(Debug)]
pub struct JsonlStore {
    path: PathBuf,
    records: Vec<AuditRecord>,
}

impl JsonlStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        OpenOptions::new().create(true).append(true).open(&path)?;

        let mut records = Vec::new();
        let reader = BufReader::new(File::open(&path)?);
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(&line)?);
        }

        verify_records(&records)?;
        Ok(Self { path, records })
    }

    pub fn append(&mut self, event: Event) -> Result<AuditRecord, StoreError> {
        let sequence = u64::try_from(self.records.len())
            .map_err(|_| StoreError::SequenceOverflow)?
            .checked_add(1)
            .ok_or(StoreError::SequenceOverflow)?;
        let previous_hash = self
            .records
            .last()
            .map_or_else(|| GENESIS_HASH.to_owned(), |record| record.hash.clone());
        let hash = compute_hash(sequence, &previous_hash, &event)?;
        let record = AuditRecord {
            sequence,
            previous_hash,
            hash,
            event,
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        serde_json::to_writer(&mut file, &record)?;
        file.write_all(b"\n")?;
        file.sync_data()?;
        self.records.push(record.clone());
        Ok(record)
    }

    pub fn records(&self) -> &[AuditRecord] {
        &self.records
    }

    pub fn records_for_workspace(&self, workspace_id: WorkspaceId) -> Vec<AuditRecord> {
        self.records
            .iter()
            .filter(|record| record.event.workspace_id == workspace_id)
            .cloned()
            .collect()
    }

    pub fn verify(&self) -> Result<(), StoreError> {
        verify_records(&self.records)
    }
}

#[derive(Serialize)]
struct HashMaterial<'a> {
    sequence: u64,
    previous_hash: &'a str,
    event: &'a Event,
}

fn compute_hash(sequence: u64, previous_hash: &str, event: &Event) -> Result<String, StoreError> {
    let bytes = serde_json::to_vec(&HashMaterial {
        sequence,
        previous_hash,
        event,
    })?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn verify_records(records: &[AuditRecord]) -> Result<(), StoreError> {
    let mut expected_previous = GENESIS_HASH.to_owned();
    for (index, record) in records.iter().enumerate() {
        let expected_sequence = u64::try_from(index)
            .map_err(|_| StoreError::SequenceOverflow)?
            .checked_add(1)
            .ok_or(StoreError::SequenceOverflow)?;
        if record.sequence != expected_sequence {
            return Err(StoreError::Integrity {
                sequence: record.sequence,
                reason: format!("expected sequence {expected_sequence}"),
            });
        }
        if record.previous_hash != expected_previous {
            return Err(StoreError::Integrity {
                sequence: record.sequence,
                reason: "previous hash does not match".into(),
            });
        }
        let expected_hash = compute_hash(record.sequence, &record.previous_hash, &record.event)?;
        if record.hash != expected_hash {
            return Err(StoreError::Integrity {
                sequence: record.sequence,
                reason: "record hash does not match".into(),
            });
        }
        expected_previous.clone_from(&record.hash);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use nuofield_core::{Actor, ActorId, ActorKind, Event, EventPayload, NewEvent, WorkspaceId};
    use tempfile::tempdir;

    use super::*;

    fn workspace_event() -> Event {
        let workspace_id = WorkspaceId::new();
        let owner_id = ActorId::new();
        Event::from_new(NewEvent {
            workspace_id,
            actor_id: owner_id,
            payload: EventPayload::WorkspaceCreated {
                name: "Local workspace".into(),
                owner: Actor {
                    id: owner_id,
                    display_name: "Owner".into(),
                    kind: ActorKind::Human,
                },
            },
        })
    }

    #[test]
    fn persists_and_verifies_records() {
        let directory = tempdir().expect("temp directory");
        let path = directory.path().join("events.jsonl");
        let mut store = JsonlStore::open(&path).expect("store should open");
        let event = workspace_event();

        store.append(event.clone()).expect("event should append");
        store.verify().expect("chain should verify");

        let reopened = JsonlStore::open(&path).expect("store should reopen");
        assert_eq!(reopened.records().len(), 1);
        assert_eq!(reopened.records()[0].event, event);
    }

    #[test]
    fn detects_tampering() {
        let directory = tempdir().expect("temp directory");
        let path = directory.path().join("events.jsonl");
        let mut store = JsonlStore::open(&path).expect("store should open");
        store
            .append(workspace_event())
            .expect("event should append");

        let content = fs::read_to_string(&path).expect("event log should be readable");
        fs::write(
            &path,
            content.replace("Local workspace", "Changed workspace"),
        )
        .expect("tamper write should succeed");

        assert!(matches!(
            JsonlStore::open(&path),
            Err(StoreError::Integrity { .. })
        ));
    }
}
