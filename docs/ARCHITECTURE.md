# NuoField Architecture

## Objective

The first architecture proves one invariant:

> A human can assign work to an independently identified Agent; risky work
> cannot start without human approval; execution, model egress, evidence, and
> audit data remain exportable by the deployment owner.

## Components

```text
Human or Agent
      │
      │ JSON event
      ▼
nuofield-server
      │
      ├─ validate ─────────► nuofield-core
      │                       zero I/O domain policy
      │
      ├─ durable append ───► nuofield-store
      │                       deployment-owned JSONL + hash chain
      │
      └─ apply ────────────► in-memory workspace projection

nuofield-cli ──────────────► HTTP API
```

### `nuofield-core`

The zero-I/O foundation. It defines actors, events, task state, approval
requirements, model invocation records, evidence, and deterministic projection
rules. It does not depend on a filesystem, database, network, or async runtime.

### `nuofield-store`

The initial single-process event store. Every accepted event becomes one JSON
line containing a monotonic sequence, previous hash, record hash, and event.
The file lives under `NUOFIELD_DATA_DIR`, is synced before acknowledgement, and
is verified when opened.

### `nuofield-server`

The orchestration boundary. It owns ordering across policy validation, durable
append, and state application. Cross-component coordination does not occur
inside the core or store crates.

### `nuofield-cli`

The stable automation entry point for agents and operators. It submits events,
reads workspace projections and event streams, and exports deployment data.

## Write pipeline

```text
1. Decode a bounded JSON body.
2. Resolve the workspace projection.
3. Validate actor, membership, assignment, risk, and task state.
4. Add server-owned event identity and timestamp.
5. Append and sync the hash-chained audit record.
6. Apply the accepted event to the in-memory projection.
7. Return the complete audit record.
```

Validation errors do not write. Storage errors do not mutate projections.

## Event model

Events are the durable source of truth. Workspace projections are disposable
views rebuilt by replaying the event log. The current event types are:

- `workspace_created`
- `actor_joined`
- `task_assigned`
- `task_approved`
- `task_started`
- `model_invocation_recorded`
- `task_completed`

New behavior should normally add an event or strengthen a projection invariant,
not mutate stored history.

## Data ownership

The initial deployment stores one file:

```text
${NUOFIELD_DATA_DIR}/events.jsonl
```

The operator can stop the process and copy, inspect, export, back up, or delete
that directory without contacting a hosted control plane.

## Scaling path

The JSONL store is intentionally a single-deployment reference implementation.
A transactional embedded store and PostgreSQL adapter can implement the same
append/replay contract without changing domain policy. Live subscriptions,
search, model routing, and user interfaces remain downstream projections and
adapters.

## Current limitations

- Actor IDs are explicit but not cryptographically authenticated.
- The store assumes one writer process.
- Hash chaining detects modification but does not prevent deletion or rollback.
- Workspace roles are limited to owner, human member, and Agent member.
- No live subscription transport exists yet.
- No model provider is invoked by the server; invocations are auditable events.

These limitations are security boundaries, not implied production capability.
