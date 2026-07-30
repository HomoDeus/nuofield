# Security Policy

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private
security advisory flow for this repository.

Include the affected revision, reproduction steps, impact, and any suggested
mitigation. Please do not access data that does not belong to you.

## Current security status

NuoField is in an early technical-slice stage. The current server:

- binds to loopback by default;
- validates task authority and approval state;
- appends before applying projections;
- synchronizes each append to user-controlled, operator-managed storage;
- verifies a tamper-evident audit chain at startup and readiness checks;
- limits request bodies to 1 MiB;
- has no telemetry or model-network dependency.

It does not yet implement cryptographic actor authentication, transport
encryption, secret storage, multi-process write coordination, or production
rate limiting. Do not expose it to an untrusted network. See
[docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) for the full boundary.

## Supported versions

Until the first stable release, only the latest commit on `main` receives
security fixes.
