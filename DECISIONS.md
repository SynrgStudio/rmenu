# DECISIONS — rmenu

Short record of active architectural decisions.

Historical or exploratory decisions may live under `docs/historico/`. This file summarizes current decisions that affect the public contract.

---

## DEC-001 — Modular core v1 is frozen around small primitives

Status: Accepted  
Date: 2026-04-24

### Context

`rmenu` evolved from a native launcher into an extensible command surface. To avoid accidental core growth, the module architecture needs an explicit boundary.

### Decision

Core v1 is stabilized around these primitives:

- providers,
- commands,
- decorations,
- input accessory,
- capabilities,
- key hooks,
- runtime actions.

New features must be attempted as modules first. A new primitive is accepted only with evidence from multiple real modules and a documented decision.

### Consequences

- The core stops receiving feature-specific behavior.
- Functional expansion happens through modules.
- The public contract is documented in frozen v1 specs.
- Breaking changes require a new API version or an explicit decision.

---

## DEC-002 — `.rmod` v1 is plain text

Status: Accepted  
Date: 2026-04-24

### Context

Modules need a distributable format that is easy to audit.

### Decision

`.rmod` v1 is always plain UTF-8 text with the `#!rmod/v1` magic line, key-value headers, and named blocks.

If a packed or binary format exists in the future, it must use a different extension.

### Consequences

- Modules are readable and auditable.
- The format is easy to version.
- The loader can normalize `.rmod` files and module directories into the same descriptor.

---

## DEC-003 — The core renders; modules declare intent

Status: Accepted  
Date: 2026-04-24

### Context

Allowing arbitrary rendering from modules would compromise stability, performance, and visual consistency.

### Decision

Modules cannot draw UI, access Win32/GDI, or modify global layout. They can only provide declarative primitives: items, badges, hints, quick-select keys, and input accessories.

### Consequences

- UI remains consistent.
- The core keeps control of layout and performance.
- Complex visual features must wait for an official primitive or live outside the core.

---

## DEC-004 — Fail the module, not the launcher

Status: Accepted  
Date: 2026-04-24

### Context

External modules can fail, hang, or return invalid payloads.

### Decision

Errors, timeouts, and invalid payloads are isolated per module/host. The launcher must remain operational.

### Consequences

- Runtime applies timeouts, backoff, and disable policies.
- `--modules-debug` exposes telemetry.
- The core prioritizes stability over completing results from faulty modules.

---

## DEC-005 — Local intents use an explicit prefix and `replaceItems`

Status: Accepted  
Date: 2026-04-24

### Context

Some modules, such as `local-scripts`, need a scoped search where their results do not compete with History, Start Menu, PATH, or global fuzzy ranking. `ProviderDef.priority` only controls provider execution order, and `dedupe_source_priority` only resolves duplicates; neither should be interpreted as general visual priority.

### Decision

Local-intent modules must use an explicit query prefix and `ctx.replaceItems(...)` to enter a scoped result mode. For `local-scripts`, the v1 prefix is `>`.

Examples:

- `>` lists local scripts.
- `> bu` filters local scripts.
- `build` keeps the normal global launcher behavior.

### Consequences

- The global launcher is not polluted by local results without explicit intent.
- The module can pre-sort exact/prefix/contains matches inside its scoped list.
- No magic boosts, semantic hints, or core ranking changes are added.
