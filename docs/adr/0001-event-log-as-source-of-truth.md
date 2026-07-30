# ADR 0001: Event log as the source of truth

- Status: Accepted
- Date: 2026-07-30

## Context

Human–Agent collaboration needs an inspectable responsibility chain. Mutable
task rows alone cannot explain who assigned work, when approval happened, which
model endpoint was used, or what evidence closed the task.

## Decision

Accepted domain events are the durable source of truth. Current workspace state
is a disposable projection rebuilt by replay.

The initial adapter writes append-only JSONL records with a monotonic sequence
and SHA-256 hash chain. The server validates policy, appends durably, and only
then applies the projection.

## Consequences

- Export and audit are native operations.
- New projections can be rebuilt without rewriting history.
- Invalid or tampered history prevents startup.
- Schema evolution must preserve replay compatibility.
- The initial adapter supports one writer and is not a production database.
