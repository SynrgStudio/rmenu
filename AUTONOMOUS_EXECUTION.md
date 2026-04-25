# AUTONOMOUS_EXECUTION.md — rmenu

Status: active autonomous execution contract  
Project: `rmenu`  
Primary use: make long-running closure/hardening work continuable across chat sessions.

---

## Purpose

This file defines how an agent should execute long-running tasks in this repository when the user invokes `/start-cont` or asks to continue autonomously.

The goal is to keep work moving until a real Definition of Done or a real stop condition is reached, while preserving safety, project rules, and restartability between sessions.

`/init-cont` creates or updates this contract.  
`/start-cont` executes tasks using this contract.

---

## Source of truth

Read in this order:

1. System/developer instructions and project rules injected by the harness.
2. `C:\Users\dnaon\.pi\agent\AGENTS.md` project rules when available in context.
3. `AUTONOMOUS_EXECUTION.md`.
4. `STATE.md`.
5. `CORE_CLOSURE_CHECKLIST.md`.
6. Root architecture/spec docs relevant to the current task:
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
7. Code and tests relevant to the current task.

If files conflict, obey the higher-priority source and document the conflict in the final report or `STATE.md`.

---

## Triggers

Use this contract when the user says any of:

- `/start-cont <task>`
- `start-cont <task>`
- `continúa en modo autónomo`
- `completa Phase 3`
- `completa fase X`
- `no pares hasta terminar <task>`
- `hacé todo lo que puedas hasta terminar`
- `modo autónomo <task>`
- `continúa desde STATE.md`

If the user invokes `/init-cont`, update this file only; do not execute the long-running task unless separately asked.

---

## Scope and autonomy level

Autonomy level: high within the requested scope.

The agent may, without asking:

- inspect relevant source/docs/config;
- add or update targeted tests;
- make minimal implementation fixes required by the requested task;
- refactor small internal code only when needed for correctness, hardening, or tests;
- create temporary fixtures in tests;
- update `STATE.md`, `CORE_CLOSURE_CHECKLIST.md`, and relevant docs/checklists;
- run allowed validation commands;
- continue through multiple small implementation/test/doc loops until DoD or stop condition.

The agent must ask before:

- removing intentional functionality;
- changing public module API/v1 contracts unless explicitly required and documented;
- adding dependencies;
- changing product/UX policy not already decided in specs;
- running prohibited commands;
- committing, pushing, rebasing, or staging files;
- touching broad unrelated files if other agents may be working.

---

## Allowed actions

Allowed during autonomous execution:

- Read files with `read`.
- Search with safe shell commands such as `rg`, `find`, `git diff`, `git status`.
- Edit only task-relevant files.
- Add focused unit tests or integration-style tests inside the existing Rust test structure.
- Update state/checklist/docs to reflect validated progress.
- Run validation commands listed below.
- Run targeted diagnostic commands listed below.

Prefer small, verifiable increments over broad rewrites.

---

## Forbidden actions

Do not do these unless the user explicitly asks and confirms override:

- `git reset --hard`
- `git checkout .`
- `git clean -fd`
- `git stash`
- `git add -A`
- `git add .`
- `git commit --no-verify`
- commit/push/rebase without explicit user request;
- remove intentional functionality to silence errors;
- downgrade dependencies to fix type/build issues;
- introduce shortcut-specific or module-specific core behavior unless it is a documented v1 contract correction/general primitive;
- start Phase 6 freeze declaration before blockers are resolved.

Also obey all higher-level project rules from the harness/AGENTS context.

---

## Validation commands

Rust/project validation currently used for `rmenu`:

```bash
cargo fmt
cargo test
cargo check
```

Module diagnostics:

```bash
cargo run --bin rmenu -- --modules-debug
```

Other useful diagnostics, when relevant:

```bash
cargo run --bin rmenu -- --metrics
cargo run --bin rmenu -- --debug-ranking <query>
cargo run --bin rmenu -- --reindex
```

Release/manual command, only when needed and not prohibited by current instructions:

```powershell
cargo build --release
.\target\release\rmenu.exe
```

Current project rules note:

- `cargo test` and `cargo check` are allowed and already used.
- `cargo fmt` is expected after Rust changes.
- `cargo clippy` is not adopted as mandatory; only run if the user/project explicitly adopts it.
- Do not run commands forbidden by higher-level rules.

---

## Execution loop

For `/start-cont <task>`:

1. Read `AUTONOMOUS_EXECUTION.md`.
2. Read `STATE.md`.
3. Read the relevant checklist section in `CORE_CLOSURE_CHECKLIST.md`.
4. Identify the next unchecked/highest-priority item inside requested scope.
5. Inspect only relevant code/docs.
6. Implement the smallest safe change.
7. Run required validation.
8. If validation fails, debug root cause and iterate.
9. Update `STATE.md` and checklist/docs.
10. Continue with the next item.
11. Stop only on DoD or a real stop condition.

Do not stop merely because a partial batch of work is complete.

---

## Stop conditions

Stop only when one is true:

- requested task/phase DoD is complete;
- user/product input is required;
- manual/visual validation is required and cannot be automated;
- a required validation command is forbidden, unavailable, or fails due to environment;
- tests/checks fail and root cause remains unclear after reasonable debugging;
- continuing would require destructive action;
- context limit is near and a checkpoint is needed;
- work would require changing public API/v1 contract without explicit approval.

Before stopping, update `STATE.md` and relevant checklist if possible.

---

## Checkpointing rules

Before every autonomous stop, update `STATE.md` with:

- what was completed;
- files changed;
- validation commands run and results;
- remaining blockers;
- exact next recommended step;
- whether user/manual input is needed.

Update `CORE_CLOSURE_CHECKLIST.md` as follows:

- mark `[x]` only when validated;
- leave `[ ]` for partial coverage and add a short note describing what is covered and what remains;
- do not mark manual validation complete unless it was actually performed or the user has confirmed it.

If the context is near the limit, write a checkpoint first, then stop with re-entry instructions.

---

## Reporting format

Final response after `/start-cont` should be concise:

```text
Estado: completado | bloqueado | checkpoint
Tarea: <task>
Hecho:
- ...
Archivos:
- ...
Validación:
- <command>: OK/failed
Pendiente/Bloqueo:
- ...
Siguiente:
- ...
```

Do not claim completion if validation or checklist items remain.

---

## Project-specific task map

Primary checklist:

```text
CORE_CLOSURE_CHECKLIST.md
```

Current major phases:

- Phase 1: documentation/vocabulary/architecture boundary — mostly complete.
- Phase 2: real module validation — functionally validated; checklist has some optional/manual mismatch to reconcile.
- Phase 3: functional hardening — complete.
- Phase 4: tests and verification — active next verification target.
- Phase 5: product/core polish — next product/manual target.
- Phase 6: freeze declaration — do not start yet.

Core principle for all phases:

> If a feature can be implemented as a module, it does not belong in the core.

Future core changes are acceptable only for critical bug, crash, security/isolation, performance, Windows compatibility, v1 contract correction, or a general need proven by several real modules.

---

## Current known long-running tasks

### Phase 3 — Functional hardening

Status: complete.

Source:

```text
CORE_CLOSURE_CHECKLIST.md#Phase 3
STATE.md#Later phases pending
```

Completed coverage according to `STATE.md` and checklist:

- external module isolation/error-isolation;
- request timeout behavior;
- hung host kill/degrade behavior;
- external restart/backoff behavior;
- auto-disable after timeout thresholds;
- successful reload resets health/backoff counters;
- one module's errors do not break others;
- `--modules-debug` exposes policy, capabilities, health, telemetry, and recent errors;
- mixed `.rmod` + directory `module.toml` discovery;
- invalid `.rmod` and `module.toml` errors;
- disabled modules are discovered but not loaded;
- deterministic descriptor priority/name ordering;
- hot reload/debounce;
- provider/command/input-accessory/key capability enforcement;
- provider budgets, timeouts, item caps, sanitization, dedupe, and command collisions;
- UI primitive rendering coverage for badges, hints, input accessory, quick select, and narrow widths;
- IPC request/response payload limits and invalid payload handling.

Do not reopen Phase 3 unless a regression or newly discovered hardening gap appears.

### Complete Phase 4 — Tests and verification

Continue Phase 4 from the remaining checklist items. Add tests only for behavior already specified or clearly required by verification.

### Prepare Phase 6 freeze

Do not start until Phase 4 tests, Phase 5 polish/performance/manual checks, and manual blockers are resolved.

---

## Current repository state notes

As of the Phase 3 completion checkpoint:

- Branch: `main`.
- Current tests reported in `STATE.md`: 72 passed, 0 failed.
- There are uncommitted working-tree changes from completed Phase 3 hardening work. Do not stage/commit unless explicitly asked.
- `git status --short` may show line-ending/timestamp noise; verify with `git diff --name-only` and actual diffs before assuming a file was intentionally changed.

---

## Re-entry instructions

To continue in a future session:

```text
/start-cont continúa desde STATE.md
```

or target the next phase explicitly:

```text
/start-cont Completa Phase 4
```

The agent must then:

1. Read this file.
2. Read `STATE.md`.
3. Read `CORE_CLOSURE_CHECKLIST.md` for the requested phase.
4. Pick the next remaining validated gap.
5. Work until DoD or stop condition.

---

## Commit policy

Do not commit unless the user explicitly asks.

If the user asks to commit:

1. Run required validation first.
2. Run `git status`.
3. Stage only files changed by this agent/session with explicit paths.
4. Never use `git add -A` or `git add .`.
5. Include issue-closing syntax only when there is a related issue.
6. Do not force push.

---

## Manual validation policy

Manual/visual validation is allowed to remain pending if it requires the user or real UI interaction.

When a manual item blocks freeze, document it in `STATE.md` and final report instead of pretending it is complete.

Known manual-ish areas:

- launcher UX feel/performance;
- visual UI primitive rendering under real Windows conditions;
- manual launcher mode and stdin/script mode tests;
- final shortcuts checklist mismatch in `CORE_CLOSURE_CHECKLIST.md` vs `STATE.md` should be reconciled carefully.
