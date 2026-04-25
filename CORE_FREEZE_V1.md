# CORE FREEZE V1 — rmenu

Status: frozen v1  
Freeze date: 2026-04-24  
Project: `rmenu`  
API version: module API v1

---

## 1. Declaration

The `rmenu` core is frozen as a stable launcher and module platform.

From this freeze onward, new product features must be attempted as modules first. The core should not grow by feature accumulation.

Main rule:

> If a feature can be implemented as a module, it does not belong in the core.

---

## 2. Frozen core scope

The frozen v1 core scope includes:

- native Windows launcher window, rendering, input, selection, and scroll;
- fuzzy matching and source-aware ranking;
- base sources: History, Start Menu, PATH, and direct input;
- index cache and cache invalidation;
- launch backend through `ShellExecuteW` with controlled fallback;
- configuration, CLI options, metrics, diagnostics, and debug commands;
- module discovery for `.rmod` and directory `module.toml` modules;
- external module host process and IPC boundary;
- capability enforcement;
- payload validation and sanitization;
- timeout, budget, dedupe, restart, and disable policies;
- module telemetry and `--modules-debug` reporting;
- official v1 UI primitives rendered by the core.

The core remains authoritative over:

- UI rendering and layout;
- global state;
- ranking and final ordering;
- execution and launch policy;
- module loading and routing;
- capability checks;
- error isolation and recovery;
- diagnostics and performance guardrails.

---

## 3. Frozen module API v1

The module API v1 is frozen around these primitives:

- Providers;
- Commands;
- Decorations;
- Input Accessory;
- Capabilities;
- Key Hooks;
- Runtime Actions.

Frozen public contracts:

- `MODULES_ARCHITECTURE.md`
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

These documents define the v1 conceptual and operational contract. Valid v1 modules should continue to work unless a documented v2 or explicit migration decision supersedes v1.

---

## 4. Frozen module formats

### `.rmod` v1

`.rmod` v1 remains a plain UTF-8 text format with:

- `#!rmod/v1` magic line;
- key-value headers;
- named blocks such as `module.js`, `config.json`, and `readme.md`.

### Directory manifest v1

Directory modules remain defined by:

- `modules/<name>/module.toml`;
- declared entry file;
- declared metadata and capabilities.

Both formats normalize into the same internal descriptor model.

---

## 5. Explicit non-goals for the core

The frozen core must not absorb feature-specific behavior such as:

- calculator logic;
- script catalog logic;
- shortcut domain policy;
- clipboard managers;
- note-taking workflows;
- custom local automation;
- specialized productivity commands;
- module-specific ranking hacks;
- module-specific UI layouts.

Those belong in modules unless several real modules demonstrate a general primitive gap.

---

## 6. Allowed future core changes

Post-freeze core changes are allowed only for:

1. critical bug fixes;
2. crash fixes;
3. security or isolation fixes;
4. performance improvements;
5. Windows compatibility fixes;
6. v1 contract corrections;
7. diagnostics or observability improvements that do not change valid module semantics;
8. general needs demonstrated by several real modules.

Allowed non-breaking v1 changes include:

- documentation clarifications;
- additional tests;
- stricter validation that only rejects invalid data;
- safer defaults;
- optional ignorable fields;
- better debug output;
- internal refactors that preserve public behavior.

---

## 7. Disallowed future core changes

Post-freeze core changes are rejected when they are justified only by:

- convenience for one module;
- one local workflow;
- product feature accumulation;
- hardcoded ranking exceptions;
- module-specific rendering;
- exposing internal Rust state or memory layout;
- bypassing capabilities;
- replacing module-level composition with core-specific behavior.

---

## 8. New primitive policy

A new core primitive requires all of:

1. documented friction from real modules;
2. evidence that v1 primitives cannot express the need;
3. proof that the need is general, not module-specific;
4. a decision record in `DECISIONS.md` or a successor decision document;
5. a public spec;
6. capability and error-isolation rules when applicable;
7. tests;
8. authoring and operations guidance;
9. migration or compatibility notes.

A new primitive may require module API v2.

---

## 9. Breaking-change policy

Breaking v1 changes include:

- removing public fields;
- changing meaning of existing fields or capabilities;
- changing command routing semantics incompatibly;
- changing valid `.rmod` or `module.toml` semantics incompatibly;
- allowing modules to bypass core policies;
- exposing core internals as API.

Breaking changes require a documented API version change or explicit migration decision.

---

## 10. Validation basis

This freeze is based on:

- three validated real modules:
  - `modules/calculator.rmod`;
  - `modules/local-scripts.rmod`;
  - `modules/shortcuts.rmod`;
- completed Phase 3 hardening;
- completed Phase 4 automated and local verification;
- completed Phase 5 product/core polish;
- successful validation with modules enabled and with no external modules loaded;
- current test baseline: 74 tests passed, 0 failed.

Validated module capabilities include:

- external `InputAccessory`;
- external `ctx.replaceItems([])` and `ctx.replaceItems(items)`;
- command-prefill flow through `ctx.setQuery(...)`;
- key hooks with capability enforcement;
- providers, commands, input accessory actions, runtime actions, telemetry, restart/backoff, disable thresholds, hot reload, and IPC payload limits.

---

## 11. Future work after freeze

After core freeze, normal work moves to:

- modules;
- module templates;
- examples;
- documentation;
- visual polish within existing primitives;
- bug fixes;
- performance improvements;
- Windows compatibility;
- controlled API evolution through documented decisions.

---

## 12. Final statement

`rmenu` core v1 is closed as a launcher and module platform.

Future feature development should happen through modules first. Core changes must follow the freeze policy above.
