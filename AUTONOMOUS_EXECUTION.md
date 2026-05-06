---
continuity_session: CONT-2026-05-04-1945-ahk-suite-rmenu-migration
created_at: 2026-05-04 19:45
updated_at: 2026-05-04 19:45
status: active
goal: Migrar la suite AHK hacia rmenu de forma nativa mediante core primitives, módulos, helpers y daemon futuro
---

# AUTONOMOUS_EXECUTION.md — rmenu AHK suite migration

## Purpose

This file defines the operating contract for the active continuable session.

Goal: migrate the existing `C:\tuicommandcenter\AHKFolder` AHK suite toward `rmenu` in the most native architecture possible, starting with core primitives and module foundations, then module implementations, helpers, and a future daemon for global hotkeys/hooks.

The active work must remain auditable, pausable, resumable, and safe for parallel-agent work.

## Session metadata

```text
continuity_session: CONT-2026-05-04-1945-ahk-suite-rmenu-migration
created_at: 2026-05-04 19:45
updated_at: 2026-05-04 19:45
status: active
goal: Migrar la suite AHK hacia rmenu de forma nativa mediante core primitives, módulos, helpers y daemon futuro
```

Managed files:

```text
AUTONOMOUS_EXECUTION.md = reglas del juego
ACTIVE_QUEUE.md         = cola/tareas/dependencias
STATE.md                = bitácora/checkpoints
```

## Source of truth

Read in this order:

1. System/developer instructions and project rules injected by the harness.
2. `C:\Users\dnaon\.pi\agent\AGENTS.md` rules when available in context.
3. `AUTONOMOUS_EXECUTION.md`.
4. `ACTIVE_QUEUE.md`.
5. `STATE.md`.
6. `codebase-report.md` generated for the AHK-to-rmenu migration.
7. Root `rmenu` architecture/spec docs:
   - `README.md`
   - `MODULES_ARCHITECTURE.md`
   - `MODULES_API_SPEC_V1.md`
   - `CTX_ACTIONS_SPEC_V1.md`
   - `RMOD_SPEC_V1.md`
   - `MANIFEST_SPEC_V1.md`
   - `MODULES_CAPABILITIES_MATRIX.md`
   - `MODULES_AUTHORING_GUIDE.md`
   - `MODULES_OPERATIONS_GUIDE.md`
   - `CORE_FREEZE_V1.md`
   - `DECISIONS.md`
8. AHK suite source when needed:
   - `C:\tuicommandcenter\AHKFolder\Master.ahk`
   - `C:\tuicommandcenter\AHKFolder\config.ahk`
   - `C:\tuicommandcenter\AHKFolder\Modules\*.ahk`
   - `C:\tuicommandcenter\AHKFolder\anytype.js`
   - `C:\tuicommandcenter\AHKFolder\sniptool\*.py`

## Command chain

```text
/init-cont  -> create/refresh active continuity infrastructure
/plan-cont  -> refine ACTIVE_QUEUE.md into an official implementation plan
/start-cont -> execute the queue until DoD or real blocker
/fin-cont   -> archive the session and suggest commit message
```

## Triggers

This contract applies when the user invokes:

- `/plan-cont`
- `/start-cont`
- `/fin-cont`
- “continúa”
- “ejecuta cola”
- “arma el plan”
- “no pares hasta terminar”

## Scope and autonomy level

Allowed scope for this session:

- Core platform changes that are general primitives for modules, not AHK-specific hacks.
- Module path resolution for dev/debug and installed usage.
- Module config/secrets conventions.
- Item action primitives (`launch`, `command`, `copy`, `runas`) if validated by plan.
- `onSubmit` implementation and submit outcome behavior.
- JS bridge parity with documented module API.
- Module feedback/log/toast/accessory behavior.
- Initial `.rmod` modules replacing Command Center command surfaces.
- Documentation/spec updates needed to keep v1 contract coherent.
- Tests and validation for changed behavior.

Out of immediate scope unless explicitly planned later:

- Full AHK WindowManager port.
- Permanent global hotkey daemon implementation.
- TweetFlow implementation; keep as future module/helper placeholder.
- Releasing/publishing artifacts.

Autonomy level:

- Plan thoroughly.
- Implement only after `/start-cont`.
- Prefer incremental, testable changes.
- Stop for user confirmation before removing intentional behavior or making breaking API decisions.

## Allowed actions

- Read source/spec/docs relevant to the active task.
- Edit only files required by the claimed task.
- Add tests for changed behavior.
- Run validation commands required by this repo.
- Update `ACTIVE_QUEUE.md` and `STATE.md` at checkpoints.
- Create modules under `modules/` when the plan reaches module implementation.
- Create docs for migration decisions when core/module contracts change.

## Forbidden actions

- Do not commit unless the user explicitly asks.
- Do not use `git add -A` or `git add .`.
- Do not run destructive git commands:
  - `git reset --hard`
  - `git checkout .`
  - `git clean -fd`
  - `git stash`
- Do not overwrite unrelated uncommitted changes from other agents.
- Do not hardcode personal secrets/tokens into code, modules, examples, reports, or logs.
- Do not move AHK-specific workflow into the `rmenu` core when it can live as module/helper/daemon.
- Do not run `npm run dev`, `npm run build`, or `npm test` unless explicitly instructed by project rules for this repo. This repo is Rust; use Cargo validation.

## Validation commands

For Rust code changes, use targeted validation first, then broad validation when appropriate:

```bash
cargo fmt --check
cargo check
cargo test
```

When code was formatted, run:

```bash
cargo fmt
cargo check
cargo test
```

For module/runtime changes, also run relevant diagnostics when possible:

```bash
cargo run --bin rmenu -- --modules-debug
```

For release-mode/performance-sensitive changes, only if requested or plan requires it:

```bash
cargo build --release
cargo run --bin rmenu -- --metrics
```

No validation is required for documentation-only edits, but mention that validation was not run.

## Execution loop

For `/start-cont`:

1. Read `AUTONOMOUS_EXECUTION.md`, `ACTIVE_QUEUE.md`, and `STATE.md`.
2. Pick first `pending` task whose dependencies are done.
3. Mark it `in_progress`, claim it, and checkpoint in `STATE.md`.
4. Implement the smallest coherent change.
5. Run task validation.
6. Fix failures before moving on.
7. Mark task `done`, `partial`, or `blocked` with evidence.
8. Update `STATE.md`.
9. Continue until all DoD are complete or a real blocker appears.

## Stop conditions

Stop and report when:

- A required decision is product/API-breaking and not already approved.
- Validation fails and root cause cannot be fixed safely.
- A file conflict appears in a file not touched by this session.
- A task requires secrets, external API credentials, or paid services unavailable locally.
- A command would be destructive or forbidden.
- Disk space/toolchain/environment blocks validation.

## Checkpointing rules

Update `STATE.md` after:

- init/plan/start/finish transitions;
- each completed task;
- each blocker;
- each validation run;
- each scope-changing decision.

Checkpoint entries must include:

- timestamp;
- task ID;
- files changed;
- validation result;
- next step.

## Queue rules

- Status values: `pending`, `in_progress`, `done`, `blocked`, `partial`, `cancelled`.
- Never renumber existing task IDs.
- Add new tasks with new IDs only.
- Preserve completed task history.
- Dependencies must be explicit.
- Prefer small task DoD over large ambiguous tasks.

## Claim rules

- Claim only one task at a time unless tasks are explicitly independent.
- Preserve claims unless stale or explicitly overridden.
- If another agent has modified a task's likely files, inspect before editing.

## Archive/finalization rules

`/fin-cont` should:

- archive active files under `docs/continuity/archive/<continuity_session>/`;
- mark active files as archived/idle or clear queue according to skill rules;
- generate suggested commit message;
- summarize done/partial/blocked work;
- not commit unless user asks.

## Reporting format

Use concise Spanish technical reports:

```text
Hecho:
- ...

Validación:
- ...

Siguiente:
- ...

Bloqueos:
- ninguno / ...
```

## Project-specific task map

Primary core files likely involved:

- `src/settings.rs` — CLI/config parsing, possible `--modules-dir`.
- `src/main.rs` — module dir resolution and runtime config wiring.
- `src/app_state.rs` — `LauncherItem` action fields if adopted.
- `src/launcher.rs` — `runas`, copy/launch action helpers if adopted.
- `src/ui_win32.rs` — Enter/submit flow, close/keep-open behavior, clipboard feedback if needed.
- `src/modules/types.rs` — public module item/action types.
- `src/modules/ipc.rs` — IPC payloads/actions for external modules.
- `src/modules/context.rs` — ctx actions and snapshots.
- `src/modules/actions.rs` — action application and validation.
- `src/modules/hooks.rs` — `onSubmit` trait hook.
- `src/modules/mod.rs` — runtime dispatch, capability checks, telemetry.
- `src/module_host_main.rs` — JS bridge parity.
- `modules/*.rmod` — new/updated modules.
- Root specs/docs — update when public contract changes.

AHK reference files:

- `C:\tuicommandcenter\AHKFolder\Modules\CommandCenter.ahk`
- `C:\tuicommandcenter\AHKFolder\Modules\AnytypeHandler.ahk`
- `C:\tuicommandcenter\AHKFolder\Modules\gDriveRClone.ahk`
- `C:\tuicommandcenter\AHKFolder\Modules\ColorPicker.ahk`
- `C:\tuicommandcenter\AHKFolder\Modules\SnipTool.ahk`
- `C:\tuicommandcenter\AHKFolder\anytype.js`

## Re-entry instructions

On a new session:

1. Read this file.
2. Read `ACTIVE_QUEUE.md`.
3. Read `STATE.md`.
4. Run `git status --short`.
5. Continue from first pending unblocked task.
6. Avoid touching files outside the claimed task.

## Commit policy

- Do not commit without explicit user request.
- If user asks to commit, stage only files changed in this session.
- Never use `git add -A` or `git add .`.
- Run required validation before commit unless user explicitly accepts skipping.
- Include related issue/PR references only when applicable.
