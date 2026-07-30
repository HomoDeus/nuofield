# Event and HTTP Protocol

## Event envelope

Clients submit a `NewEvent`:

```json
{
  "workspace_id": "00000000-0000-4000-8000-000000000001",
  "actor_id": "00000000-0000-4000-8000-000000000002",
  "payload": {
    "type": "workspace_created",
    "name": "Example workspace",
    "owner": {
      "id": "00000000-0000-4000-8000-000000000002",
      "display_name": "Owner",
      "kind": "human"
    }
  }
}
```

After validation, the server adds an event ID and timestamp. The durable
response also contains `sequence`, `previous_hash`, and `hash`.

## HTTP API

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/healthz` | Process liveness |
| `GET` | `/readyz` | Reverify the audit chain |
| `POST` | `/v1/events` | Validate and append one event |
| `GET` | `/v1/workspaces/{id}` | Read the current workspace projection |
| `GET` | `/v1/workspaces/{id}/events` | Export one workspace's audit records |
| `GET` | `/v1/export` | Export all audit records |

## First complete loop

Use stable UUIDs for the workspace, owner, Agent, and task. Generate them with:

```bash
nuofield id
```

Submit these payload types in order:

1. `workspace_created`, signed logically by the owner ID.
2. `actor_joined` with an Agent actor, submitted by the owner.
3. `task_assigned` to that Agent with `risk: "high"`.
4. `task_approved`, submitted by a human member.
5. `task_started`, submitted by the assigned Agent.
6. Optional `model_invocation_recorded`:

```json
{
  "type": "model_invocation_recorded",
  "task_id": "00000000-0000-4000-8000-000000000004",
  "invocation": {
    "provider": "local-runtime",
    "model": "example-model",
    "endpoint": "local"
  }
}
```

7. `task_completed` with at least one evidence item:

```json
{
  "type": "task_completed",
  "task_id": "00000000-0000-4000-8000-000000000004",
  "summary": "Completed with verified output",
  "evidence": [
    {
      "kind": "artifact",
      "uri": "file:///approved/output/report.pdf",
      "digest": "sha256:..."
    }
  ]
}
```

A high-risk task cannot start before step 4. Only the assigned Agent can submit
steps 5–7, and completion without evidence is rejected.

## Error behavior

- Invalid JSON or UUID: `400 Bad Request`
- Domain-policy conflict: `409 Conflict`
- Unknown workspace: `404 Not Found`
- Storage failure: `500 Internal Server Error`

The current transport does not authenticate `actor_id`. Keep it on a trusted
loopback or isolated development network until request signing is implemented.
