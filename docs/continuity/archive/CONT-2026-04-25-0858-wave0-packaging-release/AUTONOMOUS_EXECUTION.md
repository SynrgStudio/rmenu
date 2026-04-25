---
continuity_session: CONT-2026-04-25-0858-wave0-packaging-release
created_at: 2026-04-25 08:58
updated_at: 2026-04-25 08:58
status: active
goal: Preparar Wave 0 Packaging/release para rmenu post-freeze
---

# AUTONOMOUS_EXECUTION.md — rmenu

Status: active autonomous execution contract  
Project: `rmenu`  
Primary use: make Wave 0 packaging/release work continuable across chat sessions.

---

## Purpose

This file defines how an agent should execute the active continuable session for `rmenu`.

Active goal:

> Preparar Wave 0 Packaging/release para rmenu post-freeze: release checklist, GitHub release workflow, install/update documentation, binary signing research, changelog/release notes, and packaging policy after Core Closed v1.

The goal is to keep work moving until a real Definition of Done or a real stop condition is reached, while preserving safety, project rules, and restartability between sessions.

---

## Session metadata

```text
continuity_session: CONT-2026-04-25-0858-wave0-packaging-release
created_at: 2026-04-25 08:58
updated_at: 2026-04-25 08:58
status: active
goal: Preparar Wave 0 Packaging/release para rmenu post-freeze
```

Managed files:

```text
AUTONOMOUS_EXECUTION.md = reglas del juego
ACTIVE_QUEUE.md         = cola/tareas/dependencias
STATE.md                = bitácora/checkpoints
```

---

## Source of truth

Read in this order:

1. System/developer instructions and project rules injected by the harness.
2. `C:\Users\dnaon\.pi\agent\AGENTS.md` project rules when available in context.
3. `AUTONOMOUS_EXECUTION.md`.
4. `ACTIVE_QUEUE.md`.
5. `STATE.md`.
6. `CORE_FREEZE_V1.md`.
7. `POST_FREEZE_ROADMAP.md`.
8. `CORE_CLOSURE_CHECKLIST.md`.
9. Root architecture/spec docs relevant to the current task:
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
10. Code, workflows, docs, and release files relevant to the current task.

If files conflict, obey the higher-priority source and document the conflict in the final report or `STATE.md`.

---

## Command chain

```text
/init-cont  -> creates/updates this contract + state + active queue
/plan-cont  -> refines ACTIVE_QUEUE.md with a deeper implementation plan
/start-cont -> executes ACTIVE_QUEUE.md until DoD or stop condition
/fin-cont   -> archives the continuity session and suggests a commit message
```

---

## Triggers

Use this contract when the user says any of:

- `/start-cont`
- `/start-cont Wave 0`
- `start-cont`
- `continúa en modo autónomo`
- `ejecuta cola`
- `no pares hasta terminar`
- `continúa desde STATE.md`
- `prepará packaging/release`

If the user invokes `/init-cont`, update continuity infrastructure only; do not execute the long-running task unless separately asked.

---

## Scope and autonomy level

Autonomy level: high within Wave 0 packaging/release scope.

The agent may, without asking:

- inspect relevant docs/config/workflows;
- create or update release documentation;
- create `.github/workflows/release.yml`;
- create `RELEASE_CHECKLIST.md`, `INSTALL.md`, `CHANGELOG.md`, and `docs/release/BINARY_SIGNING.md`;
- update `README.md`, `POST_FREEZE_ROADMAP.md`, `STATE.md`, and related release docs;
- define artifact layout and packaging policy;
- define how demo modules are packaged as examples;
- run allowed validation commands;
- iterate on docs/workflow YAML errors when locally detectable.

The agent must ask before:

- changing core launcher/module runtime functionality;
- changing public module API/v1 contracts;
- adding dependencies;
- adding MSI/auto-updater/installer behavior beyond documentation;
- committing, pushing, rebasing, or staging files;
- touching broad unrelated files if other agents may be working.

---

## Allowed actions

Allowed during autonomous execution:

- Read files with `read`.
- Search with safe shell commands such as `rg`, `find`, `git diff`, `git status`.
- Edit only Wave 0 task-relevant files.
- Create focused docs/workflow files.
- Update `ACTIVE_QUEUE.md`, `STATE.md`, `POST_FREEZE_ROADMAP.md`, and relevant docs.
- Run validation commands listed below.

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
- downgrade dependencies;
- add MSI/auto-updater complexity in Wave 0;
- change frozen core behavior for release convenience;
- package `shortcuts.rmod` with local user-specific targets as an active default module.

Also obey all higher-level project rules from the harness/AGENTS context.

---

## Validation commands

Primary validation for this session:

```bash
cargo fmt
cargo test
cargo check
cargo build --release
```

Useful release diagnostics, when relevant:

```bash
cargo run --bin rmenu -- --metrics
cargo run --bin rmenu -- --modules-debug
cargo run --bin rmenu -- --debug-ranking blender
cargo run --bin rmenu -- --reindex --metrics
```

GitHub Actions workflow validation is partially manual/local; full validation requires running the workflow on GitHub.

---

## Execution loop

For `/start-cont`:

1. Read `AUTONOMOUS_EXECUTION.md`.
2. Read `ACTIVE_QUEUE.md`.
3. Read `STATE.md`.
4. Pick the first executable task by queue rules.
5. Inspect only relevant docs/config/workflow/code.
6. Implement the smallest safe change.
7. Run task validation.
8. If validation fails, debug root cause and iterate.
9. Update `ACTIVE_QUEUE.md` and `STATE.md`.
10. Continue with the next executable task.
11. Stop only on DoD or a real stop condition.

Do not stop merely because a partial batch of work is complete.

---

## Stop conditions

Stop only when one is true:

- Wave 0 DoD is complete;
- user/product input is required;
- GitHub-only validation is required and cannot be performed locally;
- a required validation command is unavailable or fails due to environment;
- tests/checks fail and root cause remains unclear after reasonable debugging;
- continuing would require destructive action;
- context limit is near and a checkpoint is needed;
- work would require changing frozen core behavior or public API/v1 contract without explicit approval.

Before stopping, update `STATE.md` and `ACTIVE_QUEUE.md` if possible.

---

## Checkpointing rules

Before every autonomous stop, update `STATE.md` with:

- what was completed;
- files changed;
- validation commands run and results;
- remaining blockers;
- exact next recommended step;
- whether user/manual input is needed.

Update `ACTIVE_QUEUE.md` as follows:

- mark `done` only when DoD and validation are satisfied;
- mark `partial` if progress is validated but work remains;
- mark `blocked` if user/manual/GitHub-only validation is required;
- never mark manual validation complete unless it was actually performed or confirmed by the user.

---

## Queue rules

Status values:

```text
pending
in_progress
done
blocked
partial
cancelled
```

Rules:

- Never renumber existing task IDs.
- Pick first `pending` task whose dependencies are done.
- Resume `in_progress` tasks claimed by this agent/session first.
- Preserve claims unless stale or explicitly overridden.
- Do not work on `blocked` or `cancelled` tasks.

---

## Claim rules

Before editing files for a task, update `ACTIVE_QUEUE.md`:

```text
Status: in_progress
Claimed by: current-agent
Started: YYYY-MM-DD HH:MM
Last update: YYYY-MM-DD HH:MM
```

Also update front matter `updated_at` in managed files when materially changed.

---

## Archive/finalization rules

When `/fin-cont` is invoked:

- archive current continuity files into the continuity archive location if configured by the skill;
- summarize completed tasks and validation;
- list remaining tasks or blockers;
- suggest a commit message;
- do not commit unless explicitly asked.

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

Active queue:

```text
ACTIVE_QUEUE.md
```

Wave 0 deliverables:

- `RELEASE_CHECKLIST.md`
- `INSTALL.md`
- `docs/release/BINARY_SIGNING.md`
- `CHANGELOG.md`
- `.github/workflows/release.yml`
- updates to `README.md`, `POST_FREEZE_ROADMAP.md`, and `STATE.md`

Current product decisions:

- Manual smoke tests have already been validated by the user and should be documented as release smoke checks to repeat.
- Release zip should include Windows x64 binaries, config/docs, and module examples where appropriate.
- `shortcuts.rmod` with local hardcoded targets must not be active by default in packaged releases.
- Prefer `modules/examples/shortcuts.example.rmod` or equivalent with generic/documented targets.
- No MSI or auto-updater in Wave 0.
- GitHub Actions release workflow should run on Windows and create artifacts/releases from `v*` tags.

---

## Re-entry instructions

To continue in a future session:

```text
/start-cont continúa desde STATE.md
```

or:

```text
/start-cont Ejecuta Wave 0 Packaging/release
```

The agent must then:

1. Read this file.
2. Read `ACTIVE_QUEUE.md`.
3. Read `STATE.md`.
4. Pick the next executable task.
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
