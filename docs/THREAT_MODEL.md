# Threat Model

## Assets

- Human and Agent identities
- Task intent, approvals, results, and evidence
- Model provider and egress records
- User-owned deployment event history
- Operator configuration and backups

## Trust boundaries

```text
untrusted caller
      │
      ▼
HTTP parsing and body limit
      │
      ▼
domain policy
      │
      ▼
single-writer local storage
      │
      ▼
operator-managed filesystem and backups
```

The operator controls the host, process, data directory, backups, and network
exposure under user authorization. Users retain sovereignty over their data
and intelligent assets. Model providers are outside the trusted storage
boundary.

## Implemented controls

- Human and Agent actor kinds are distinct.
- Workspace creation and Agent invitation require a human owner.
- High-risk tasks fail closed until human approval.
- Task execution and completion require the assigned Agent identity.
- Completion requires explicit evidence.
- Model endpoint class is recorded.
- Durable append precedes projection mutation.
- Sequence and SHA-256 hash chaining detect record modification or reordering.
- Startup replay rejects invalid domain history.
- Readiness rechecks audit integrity.
- Request bodies are limited to 1 MiB.
- The default bind address is loopback.

## Known gaps

| Gap | Consequence | Planned control |
|---|---|---|
| Actor IDs are not signed | Callers can claim another actor ID | Request signing and revocable credentials |
| Plain HTTP | Network observers can read or modify traffic | TLS termination and signed requests |
| One process lock only | Multiple writers can corrupt order | Transactional storage adapter |
| Hash head is local | An operator can roll back the entire file | User-verifiable signed checkpoints and external backup attestations |
| No rate limiting | Trusted-network denial of service remains possible | Per-actor and per-route limits |
| Evidence URI is declarative | Referenced content may disappear or change | Content-addressed attachment store |
| No secret vault | Provider keys have no managed lifecycle | User-controlled encrypted secret store |

Until the first three gaps are closed, this implementation is a trusted-network
technical slice rather than a production multi-user service.
