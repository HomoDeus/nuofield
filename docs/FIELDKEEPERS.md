# Fieldkeepers（守场人）

## Product promise

NuoField should not install as an empty agent framework.

> A deployment opens with a useful digital employee team. Users can put it to
> work immediately, then teach, modify, or extend it over time.

The default team is called the **Fieldkeepers**. It belongs to the user, lives
inside the workspace, and remains subject to the same identity, permission,
approval, evidence, and audit rules as every other Agent.

This is an approved product direction. The current M0 technical slice provides
identity, task, approval, evidence, and audit primitives; it does not yet ship
the complete Fieldkeeper runtime or customization system.

## Default responsibilities

The first team provides four logical responsibilities:

- **Lead:** receive intent, break down work, delegate, and report status.
- **Engineer:** execute approved research, documentation, code, and workflows.
- **Reviewer:** challenge results, check evidence, and surface risk.
- **Caretaker:** guide setup and watch deployment health, backups, models, and
  upgrades.

These responsibilities require separate identities or processes only when
their permissions, triggers, or accountability differ. A larger role count is
not a product goal.

## User sovereignty

A user may be an individual or an organization using the workspace as a
tenant. The user owns the Fieldkeepers' configuration, skills, knowledge,
memory, evaluations, and work products.

The deployment operator is a custodian acting under user authorization.
Operating the host, storage, or keys does not grant ownership of user data or
intelligent assets. NuoField and model providers do not become a source of
truth for those assets.

## Customization and learning

Users can evolve the default team in layers:

1. Install, fork, and edit versioned skills and operating procedures.
2. Change employee responsibilities, tools, permissions, knowledge, and models.
3. Turn approved feedback into memory, examples, and evaluation cases.
4. Optionally fine-tune model weights when the earlier layers are insufficient.

Official updates must not overwrite local customization. Every material change
must be inspectable, exportable, reversible, and portable to another
deployment.

## Governance

- Employees cannot grant themselves permissions or weaken approval policy.
- Read-only diagnosis is the default for instance operations.
- Risky changes require explicit approval, bounded execution, verification,
  and evidence.
- Employees cannot rewrite their own audit history.
- Memory changes and skill upgrades are versioned rather than silently learned.
- Users can pause, replace, export, or remove any employee.

## First acceptance test

A fresh deployment passes when a user can:

1. meet the default team without authoring prompts or personas;
2. receive a truthful instance self-check;
3. assign one real task through the normal workspace interface;
4. approve or reject risky action;
5. inspect the result, model egress, and evidence;
6. modify one skill and roll that change back; and
7. export the workspace and employee assets without a hosted control plane.
