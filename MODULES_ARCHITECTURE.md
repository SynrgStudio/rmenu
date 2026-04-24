# MODULES ARCHITECTURE — rmenu

Status: Frozen v1  
Date: 2026-04-24  
Scope: public architecture of the modular `rmenu` core.

---

## 1. Purpose

This document is the constitution of the `rmenu` module system.

It defines:

- what belongs to the core,
- what must live as a module,
- which public vocabulary exists in v1,
- which boundaries modules cannot cross,
- and when touching the core again is allowed.

Main rule:

> If a feature can be implemented as a module, it does not belong in the core.

---

## 2. Product boundary

`rmenu` is a native Windows launcher and an extensible command surface.

The core should remain:

- small,
- fast,
- predictable,
- safe,
- authoritative over UI, state, ranking, execution, and policy.

The core is not a general GUI framework or an unrestricted plugin runtime. It is a controlled base for composing extensions without compromising the primary launcher experience.

---

## 3. What the core is

The core includes only the pieces required for `rmenu` to work as a launcher and stable modular platform.

### 3.1 Launcher core

- Window creation and rendering.
- Input, selection, and scroll.
- Fuzzy matching and ranking.
- Base sources: History, Start Menu, PATH, direct input.
- Index cache.
- Launch backend through Windows/ShellExecuteW and controlled fallback.
- Configuration and CLI.
- Base metrics and diagnostics.

### 3.2 Module platform

- Module discovery.
- Directory format with `module.toml`.
- Single-file `.rmod` format.
- Normalization into a common internal descriptor.
- Module runtime.
- External module host.
- IPC boundary.
- Capability enforcement.
- Payload validation and sanitization.
- Timeout, budget, dedupe, restart, and disable policies.
- Module telemetry and debug output.

### 3.3 UI primitives allowed for modules

- Items.
- Badges.
- Hints.
- Subtitles/source labels.
- Quick-select keys.
- Input accessory.

The core renders every primitive. Modules only declare intent.

---

## 4. What the core is not

The core must not contain feature-specific behavior such as:

- calculator logic,
- productivity-specific commands,
- note-taking workflows,
- script catalogs,
- clipboard managers,
- automation for one local machine,
- custom module-specific layouts,
- domain-specific ranking hacks.

Those belong in modules unless multiple real modules prove a general need that cannot be expressed with v1 primitives.

---

## 5. Responsibility boundaries

### 5.1 Core

The core owns:

- global state,
- rendering,
- ranking,
- execution policy,
- module loading,
- capability enforcement,
- module input validation,
- dedupe,
- timeouts and recovery,
- diagnostics.

### 5.2 Module runtime

The module runtime owns:

- routing hooks,
- applying action requests after validation,
- isolating failures,
- recording telemetry,
- coordinating external hosts.

### 5.3 External module host

The host owns:

- running module code outside the main process,
- translating IPC requests to module hooks,
- serializing responses,
- failing independently from the launcher.

### 5.4 User modules

A module owns:

- declaring metadata and capabilities,
- implementing fast deterministic hooks,
- providing items or decorations through the public contract,
- handling its own domain logic,
- treating errors as recoverable.

---

## 6. Public vocabulary v1

### Providers

Providers contribute items for a query. They require `providers`. The core controls budget, timeout, merge, dedupe, and final ranking.

### Commands

Commands are named invocable actions. They require `commands`. Recommended format: `/module::command`.

### Decorations

Decorations enrich existing items with badges, hints, or similar declarative metadata. They require `decorate-items` and never control layout or drawing.

### Input Accessory

Input accessory is a short contextual status rendered next to the input. It requires `input-accessory`; the core decides color and placement.

### Capabilities

Capabilities are declarative permissions. A module declares what it uses; operations without permission are rejected with `permission_denied`.

### Key Hooks

Key hooks receive controlled key events. They require `keys` and do not replace the core keybinding system.

### Runtime Actions

Runtime actions are controlled state-change requests from modules, such as `replaceItems`, `setInputAccessory`, or `registerCommand`. The core may reject actions for invalid state or missing permission.

---

## 7. Explicit module limits

A module cannot:

- draw UI directly,
- access Win32/GDI,
- replace global layout,
- replace the ranking engine,
- mutate arbitrary core state,
- bypass capabilities,
- depend on Rust internals or memory layout,
- break launcher operation when it fails,
- assume undocumented execution order.

A module can only operate through:

- declared capabilities,
- public hooks,
- `ctx`,
- validated runtime actions,
- public item/accessory/command/provider contracts.

---

## 8. Public contracts v1

The public v1 contracts are defined in:

- `MODULES_API_SPEC_V1.md`
- `RMOD_SPEC_V1.md`
- `MANIFEST_SPEC_V1.md`
- `CTX_ACTIONS_SPEC_V1.md`
- `PROVIDER_EXECUTION_POLICY.md`
- `ERROR_ISOLATION_POLICY.md`
- `MODULES_CAPABILITIES_MATRIX.md`
- `MODULES_AUTHORING_GUIDE.md`
- `MODULES_OPERATIONS_GUIDE.md`
- `MODULES_QUICKSTART.md`
- `DECISIONS.md`

These documents form the v1 conceptual contract. If they conflict, this architecture states intent and the specific specs define operational details.

---

## 9. API stability policy v1

Allowed v1 changes:

- clarify documentation,
- fix bugs,
- harden validation without breaking valid modules,
- improve diagnostics,
- add optional ignorable fields,
- improve performance.

Breaking changes include:

- removing public fields,
- changing command routing semantics,
- letting modules bypass core policies,
- exposing internals as public API,
- changing meaning of existing capabilities.

Breaking changes require a new API version or explicit documented decision.

---

## 10. Core change policy

After v1 freeze, the core accepts changes only for:

1. critical bug,
2. crash,
3. security/isolation,
4. performance,
5. Windows compatibility,
6. v1 contract correction,
7. general need demonstrated by several real modules.

Rejected reasons:

- “it would be convenient”,
- “this one module needs it”,
- “it looks nicer in core”,
- “hardcoding it is faster”,
- “maybe it will be useful someday”.

Before adding anything to core, ask:

1. Can it be a module?
2. Is it a primitive or a feature?
3. Is it general or specific?
4. Are two or three real modules affected?
5. Does it expand the public language or just improve ergonomics?
6. Can documentation or templates solve it?

If the answers do not justify core work, it must remain a module.

---

## 11. Accepting a new primitive

A new primitive is accepted only if it:

- unlocks multiple real modules,
- has small stable semantics,
- keeps the core authoritative,
- has capability enforcement,
- has error/timeout policy if it runs logic,
- has a public spec,
- has tests,
- has authoring guidance,
- does not expose internals.

---

## 12. Sending features to modules

A feature should be sent to a module if it:

- depends on a specific workflow,
- adds a specialized command,
- adds a specific text transformation,
- adds local automation,
- introduces domain-specific logic,
- can be expressed with existing primitives.

---

## 13. API v2 process

Minimum process:

1. Document friction with examples from real modules.
2. Show why v1 primitives are insufficient.
3. Write a decision in `DECISIONS.md` or an equivalent document.
4. Define the new contract.
5. Define compatibility or migration.
6. Keep v1 as long as reasonable unless an explicit decision says otherwise.

---

## 14. Deprecation

A public API is never removed silently.

Process:

1. Document deprecated API.
2. Provide replacement.
3. Warn through debug/diagnostics when possible.
4. Document removal in changelog/decision.
5. Remove only with API bump if existing modules break.

---

## 15. Documentation policy

Official public v1 documentation lives at the repository root.

Historical reports, snapshots, and exploratory reasoning should live under:

```text
docs/historico/
docs/audits/
```

The root should contain only:

- current specs,
- official guides,
- accepted decisions,
- public system policies.

---

## 16. Core closure criteria

The core can be considered closed when:

- this architecture is documented,
- v1 vocabulary is frozen,
- public specs do not contradict each other,
- `.rmod` and directory loaders are stable,
- the external runtime isolates errors,
- capabilities are enforced,
- real modules validate the API without core-specific behavior,
- tests and checks pass,
- future core-change policy is accepted.

---

## 17. Final declaration

The core must not grow by feature accumulation.

The core exists to provide:

- stable primitives,
- safe execution,
- deterministic UI,
- public contract,
- diagnostics,
- performance.

Functional expansion of `rmenu` must happen through modules first.
