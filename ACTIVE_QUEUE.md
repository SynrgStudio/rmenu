---
continuity_session: CONT-2026-05-04-1945-ahk-suite-rmenu-migration
created_at: 2026-05-04 19:45
updated_at: 2026-05-06 22:15
planned_at: 2026-05-06 21:00
status: active
goal: Migrar la suite AHK hacia rmenu de forma nativa mediante core primitives, módulos, helpers y daemon futuro
---

# ACTIVE_QUEUE.md

## Current goal

Migrar la suite AHK hacia `rmenu` de forma nativa y mantenible, en una primera wave acotada:

- core mínimo para soportar módulos nuevos actuales;
- módulos `.rmod` pequeños/cohesivos que reemplacen partes del Command Center;
- helpers autocontenidos por módulo usando rutas relativas/config interna de `.rmod`;
- sin Anytype, TweetFlow ni daemon en esta wave.

## Queue policy

- Status values: `pending`, `in_progress`, `done`, `blocked`, `partial`, `cancelled`.
- Never renumber existing task IDs.
- Pick first pending task whose dependencies are done.
- Preserve claims unless stale or explicitly overridden.
- Each task must update `STATE.md` when claimed/completed/blocked.
- Code changes require validation before marking done.
- Documentation-only tasks should state validation not run.
- If a task touches public module behavior, update specs/docs in the same task or in an explicit dependent docs task.

## Planning result

This queue has been re-planned after the core-scope decision:

- Required core for current modules:
  - robust module directory resolution;
  - `runas` support for admin launch commands;
  - `ctx.toast`/rmenu-style toast feedback.
- Not required for current modules:
  - local config loading in core;
  - full `ItemAction` model;
  - `onSubmit`;
  - `copy` action;
  - keep-open/submit outcome;
  - broad JS bridge parity;
  - Anytype/TweetFlow/daemon work.
- Helper-backed modules should be autocontained and use `.rmod` config/defaults with relative helper paths where possible.

First executable task remains T001.

Replanned 2026-05-05 19:45 after RSnip integration decision:

- RSnip is a companion app, not a generic launched command.
- If RSnip is installed, rMenu should discover it and expose snip/record/OCR as native companion capabilities.
- In integrated mode, rMenu daemon owns/coordinates RSnip lifecycle and menu actions call RSnip through direct IPC, not PowerShell or `rsnip.exe snip` process launches.
- Standalone RSnip remains responsible for its own daemon and hotkeys when rMenu is not present.
- Existing `.rmod` wrappers for snip/record/OCR are transitional and should be replaced/cancelled once the native companion provider exists.

Replanned 2026-05-05 21:05 after persistent data-root and RSnip installer decision:

- rMenu needs a persistent data root, defaulting to `C:\rMenuData` on Windows.
- The root owns `modules/`, `companions/`, `config/`, and `state/`.
- RSnip should be installable by rMenu into `<data_dir>\companions\rsnip\`.
- After install, RSnip discovery must prefer the rMenu-managed companion path over dev/global paths.
- Native rMenu commands for `snip`, `record`, and `ocr` must continue to control RSnip through direct IPC from the installed companion.

Replanned 2026-05-06 02:55 after `/rmods` registry decision:

- Add a core-owned `/rmods` command, not a privileged store `.rmod`, to avoid module self-install permissions and security ambiguity.
- Source of truth is a GitHub repo of `.rmod` files plus an automatically generated `registry.json`.
- The user workflow should be: add `.rmod` under `modules/`, push to GitHub, GitHub Actions regenerates `registry.json`, `/rmods` fetches current registry and shows the module.
- rMenu core owns registry fetch/cache, multiselect UI, sha256 verification, atomic installation into `<data_dir>\modules`, installed metadata under `<data_dir>\state`, and module runtime reload.
- MVP should support `.rmod` packages only; directory/zip module distribution and multi-registry support can follow later.
- First executable task for this `/rmods` wave is T050; older RSnip follow-up tasks remain blocked/deferred or dependent on unfinished manual validation.

Replanned 2026-05-06 05:35 from T052 after live registry bootstrap:

- Actual registry repo is `https://github.com/SynrgStudio/rmods.git`, local path `C:\rmods`.
- Live raw registry URL for rMenu MVP should be `https://raw.githubusercontent.com/SynrgStudio/rmods/main/registry.json`.
- `calculator.rmod` is already published and present in the live registry; use it as the canonical real-module smoke item for T052/T054/T063.
- Execute T052 next to automate registry regeneration, then proceed directly into rMenu core tasks T053-T060 so `/rmods` stops falling through to normal fuzzy search.
- Keep GitHub/live validation separate from core unit tests; do not block local core implementation on Actions propagation if the workflow file is committed but remote Actions confirmation is delayed.



Replanned 2026-05-06 21:00 after daemon open-latency analysis:

- Direct `rmenu.exe --metrics` is fast enough for the current dataset: window visible/input ready in ~23ms and search p95 under 1ms.
- The perceived 1-3s delay is therefore likely in the daemon hotkey/reopen path, especially embedded UI lifecycle, repeated module `onLoad`, external host lifecycle, companion discovery, or work done before `CreateWindowExW`.
- Next wave focuses on measuring and then optimizing the daemon resident path before packaging.
- Target UX: first visible input under 200ms, warm reopens under 100-200ms if feasible, with external module hosts and companion state kept hot.
- Multiple registries remain future work and are blocked behind performance/resident UX work.

## Queue

### T001 — Record minimal migration core-change decision

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:02
Last update: 2026-05-05 10:06
Scope:
- Convert the migration report and latest user decisions into an explicit accepted decision for this session.
- Record current approved core changes:
  - module dir resolution;
  - `runas` support;
  - rmenu-style toast/`ctx.toast` feedback.
- Record what is deferred:
  - core local module config;
  - full `ItemAction`;
  - `onSubmit`;
  - `copy`;
  - keep-open;
  - broad JS bridge parity;
  - Anytype/TweetFlow/daemon.
- Record current active module list and future module list.
DoD:
- Decision note exists and names only the approved core changes for this wave.
- Decision note states why each core change is general and not AHK-specific.
- Decision note states daemon/helper boundary.
- Decision note lists active module wave and future module wave.
Validation:
- Documentation task; no code validation required.
Files likely touched:
- `DECISIONS.md` or `docs/ahk-migration/DECISION.md`
- `STATE.md`
Risk: medium
Depends on:
- none
Notes:
- This protects the frozen v1 architecture before implementation.

### T002 — Specify module directory resolution for dev/debug and installed mode

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:07
Last update: 2026-05-05 10:10
Scope:
- Define final resolution order:
  - `--modules-dir <PATH>`;
  - `RMENU_MODULES_DIR`;
  - `%APPDATA%\rmenu\modules`;
  - executable directory `modules`;
  - cwd `modules` as dev fallback.
- Define behavior when a candidate path does not exist.
- Define what `--modules-debug` prints.
DoD:
- Resolution behavior is documented in a spec/operations doc or decision note.
- Test cases to implement in T006 are listed.
Validation:
- Documentation/planning task; no code validation required.
Files likely touched:
- `MODULES_OPERATIONS_GUIDE.md`
- `README.md`
- `STATE.md`
Risk: low
Depends on:
- T001
Notes:
- User wants repo-local module testing/debug to remain easy.

### T003 — Superseded core local module config design

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original local module config/secrets design task.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- User clarified current modules should be autocontained with relative helper paths/default config inside each `.rmod`. Core local config can be future work.

### T004 — Superseded full item actions/submit design

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original full `ItemAction`, submit dispatch, close semantics design.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- Current modules do not need full `ItemAction`, `onSubmit`, `copy`, or keep-open. Minimal `runas` is handled by T034/T035.

### T005 — Superseded broad JS bridge parity audit

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original broad bridge parity audit.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- Current wave only needs toast feedback beyond existing module hooks/actions. Minimal toast is handled by T034/T036.

### T006 — Implement module directory resolution and diagnostics

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:12
Last update: 2026-05-05 10:18
Scope:
- Add `--modules-dir <PATH>` CLI option.
- Add `RMENU_MODULES_DIR` resolution.
- Add appdata/exe-dir/cwd resolution order from T002.
- Wire resolved path into `ModuleRuntime` loading.
- Include resolved path in `--modules-debug`.
DoD:
- `rmenu --modules-dir .\modules --modules-debug` uses the provided path.
- Existing repo-local module loading still works without explicit option.
- `--modules-debug` prints the resolved module directory.
- Tests cover parse/resolution behavior where feasible.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `src/settings.rs`
- `src/main.rs`
- `src/modules/mod.rs`
- `README.md`
- `MODULES_OPERATIONS_GUIDE.md`
Risk: low
Depends on:
- T002
Notes:
- First code task.

### T007 — Superseded core local config implementation

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original core local module config loading task.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- `.rmod` modules can use internal `config.json` and relative helper paths for now.

### T008 — Superseded full item action model

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original full item action data model and sanitization task.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- Minimal admin launch support is handled by T034/T035.

### T009 — Superseded broad JS bridge implementation

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original broad JS bridge parity task.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- Minimal rmenu-style toast/`ctx.toast` support is handled by T034/T036.

### T010 — Superseded combined modules task

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original combined `web-actions`, `system-actions`, and `open-actions` task.
DoD:
- Cancelled; superseded by smaller module tasks T023-T033.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- User clarified modules should be smaller/cohesive. Generic `open-actions.rmod` is not needed because the core already opens apps/paths/URLs.

### T011 — Review existing shortcuts module and app binding guidance

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:37
Last update: 2026-05-05 10:40
Scope:
- Review current `modules/shortcuts.rmod` against AHK `Config.AppBindings` behavior.
- Add migration guidance for app bindings without hardcoding private paths into distributed defaults.
- Preserve `Ctrl+B` bind workflow.
- Do not treat this as a new module; it already exists.
DoD:
- Current AHK binding behavior has an equivalent rmenu path.
- `shortcuts.rmod` still passes module loading/debug.
- No personal secret/private path is committed as a default distributed module value unless already present and intentionally accepted.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
- If module changed: manual query smoke recommended
Files likely touched:
- `modules/shortcuts.rmod`
- `modules/shortcuts.user.json` or example docs
- `MODULES_QUICKSTART.md` or migration docs
Risk: low
Depends on:
- T006
Notes:
- Existing module only; no new `shortcuts.rmod` task.

### T012 — Defer Anytype active implementation

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original active `anytype-capture.rmod` implementation task.
DoD:
- Cancelled for this wave; Anytype modules are future tasks/backlog only.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- User said Anytype is not for now.

### T013 — Superseded helper-facing combined module task

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original combined rclone/snip/color helper module task.
DoD:
- Cancelled; superseded by atomic helper-facing modules T029-T033.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- User wants each module to do one coherent thing.

### T014 — Document daemon boundary and AHK retirement path

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:41
Last update: 2026-05-05 10:43
Scope:
- Document future `rmenu-daemon.exe` responsibilities:
  - launch hotkey;
  - WindowManager;
  - Thorium gestures;
  - TaskbarVolume;
  - TextExpander;
  - AlwaysOnTop;
  - Snip hotkeys.
- Document what must remain out of `rmenu` core.
- Define phased AHK retirement/rollback strategy.
DoD:
- Future daemon boundary is explicit.
- No current core/module task accidentally absorbs global hook functionality.
Validation:
- Documentation task; no code validation required.
Files likely touched:
- `docs/ahk-migration/DAEMON_BOUNDARY.md` or similar
- `POST_FREEZE_ROADMAP.md` optionally
- `STATE.md`
Risk: low
Depends on:
- T001
Notes:
- This can be done before or after initial module work.

### T015 — Superseded onSubmit implementation

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original `onSubmit` implementation task.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- Current modules can produce items with `target` and do not need submit hooks. Anytype future may revive this.

### T016 — Superseded keep-open/submit outcome task

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original submit outcome, close, and keep-open task.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- Current modules can provide warning items/toasts and use normal close-on-launch behavior.

### T017 — Superseded broad action executors task

Status: cancelled
Claimed by:
Started:
Last update: 2026-05-05 10:00
Scope:
- Original `launch`, `command`, `runas`, `copy` action executor task.
DoD:
- Cancelled for this wave.
Validation:
- none
Files likely touched:
- none
Risk: low
Depends on:
- none
Notes:
- Minimal `runas` support is handled by T034/T035. `copy` and command actions are deferred.

### T018 — Update public docs for implemented minimal primitives

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:44
Last update: 2026-05-05 10:48
Scope:
- Update docs for implemented wave only:
  - module dir resolution;
  - `runas` support;
  - rmenu-style toast/`ctx.toast` support;
  - helper-backed module convention using relative paths/config inside `.rmod`.
- Do not document deferred primitives as implemented.
DoD:
- Docs match implementation.
- Existing examples remain valid.
- New examples do not contain private paths/secrets.
Validation:
- Documentation task; no code validation required unless examples are executable and changed.
Files likely touched:
- `README.md`
- `MODULES_API_SPEC_V1.md`
- `CTX_ACTIONS_SPEC_V1.md`
- `MODULES_AUTHORING_GUIDE.md`
- `MODULES_OPERATIONS_GUIDE.md`
- `MODULES_QUICKSTART.md`
Risk: medium
Depends on:
- T006
- T035
- T036
Notes:
- Can be partially updated with each task; final sweep remains useful.

### T019 — Create AHK parity and future module inventory docs

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:23
Last update: 2026-05-05 11:27
Scope:
- Create command inventory mapping AHK CommandCenter commands to rmenu modules.
- Include active modules:
  - `web-query.rmod`;
  - `url-open.rmod`;
  - `chatgpt-open.rmod`;
  - `yandex-open.rmod`;
  - `terminal.rmod`;
  - `pi-launcher.rmod`;
  - `rclone-backup.rmod`;
  - `snip.rmod`;
  - `screen-record.rmod`;
  - `ocr.rmod`;
  - `color-picker.rmod`;
  - existing `shortcuts.rmod`.
- Include future modules:
  - Anytype split modules;
  - TweetFlow split modules;
  - daemon-backed modules.
- Include manual validation checklist.
DoD:
- User can see what replaces each AHK command.
- Manual parity checklist exists.
- Future modules are documented but not implemented.
Validation:
- Documentation task; no code validation required.
Files likely touched:
- `docs/ahk-migration/COMMAND_INVENTORY.md`
- `docs/ahk-migration/MANUAL_VALIDATION.md`
Risk: low
Depends on:
- T011
- T014
- T023
- T024
- T025
- T026
- T027
- T028
- T029
- T030
- T031
- T032
- T033
Notes:
- Use AHK source as reference, but do not read/print secrets.

### T020 — Manual launcher/module UX validation

Status: blocked
Claimed by:
Started:
Last update: 2026-05-05 10:00
Blocker:
- Requires user or interactive local Windows validation after modules/core changes are implemented.
Scope:
- Open launcher and validate query flows:
  - `g rust win32`;
  - `y blender nodes`;
  - `w reddit.com`;
  - `chatgpt` / `gpt`;
  - `ya`;
  - `ter` / `ater`;
  - `pi` / `api`;
  - shortcut aliases;
  - rclone/snip/record/ocr/color helper missing-config feedback.
DoD:
- Manual validation results recorded in `STATE.md`.
- Regressions are either fixed or converted to follow-up tasks.
Validation:
- manual: run `rmenu.exe` interactively on Windows.
Files likely touched:
- `STATE.md`
- possible follow-up fixes
Risk: medium
Depends on:
- T011
- T023
- T024
- T025
- T026
- T027
- T028
- T029
- T030
- T031
- T032
- T033
Notes:
- Keep blocked until implementation wave is complete.

### T021 — Final validation sweep for this migration wave

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:28
Last update: 2026-05-05 11:34
Scope:
- Run broad validation after implementation tasks.
- Confirm module diagnostics are healthy.
- Confirm docs do not mention secrets.
- Confirm no unrelated work was staged/changed by this session beyond intended files.
DoD:
- `cargo fmt --check` passes or code has been formatted with `cargo fmt`.
- `cargo check` passes.
- `cargo test` passes.
- `cargo run --bin rmenu -- --modules-debug` succeeds.
- `STATE.md` contains final validation summary.
Validation:
- `cargo fmt --check` or `cargo fmt`
- `cargo check`
- `cargo test`
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `STATE.md`
Risk: medium
Depends on:
- T018
- T019
Notes:
- If disk space/toolchain blocks validation, mark blocked with exact error.

### T022 — Prepare handoff summary and next-wave backlog

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:35
Last update: 2026-05-05 11:37
Scope:
- Summarize what was implemented.
- List deferred work:
  - Anytype split modules;
  - TweetFlow split modules;
  - core local module config;
  - full action model/copy/onSubmit;
  - real rclone helper;
  - real snip/color helpers;
  - `rmenu-daemon` future;
  - full AHK retirement.
- Prepare suggested next session goals.
DoD:
- `STATE.md` has handoff summary.
- Deferred backlog is clear and not mixed with current DoD.
Validation:
- Documentation task; no code validation required.
Files likely touched:
- `STATE.md`
- optional roadmap/migration docs
Risk: low
Depends on:
- T021
Notes:
- This should be last before `/fin-cont`.

### T023 — Add web-query.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:49
Last update: 2026-05-05 10:52
Scope:
- Add one cohesive web query module.
- Support:
  - `g <query>` -> Google search;
  - `y <query>` -> YouTube search.
DoD:
- Module loads in `--modules-debug`.
- Empty queries do not replace launcher results.
- Matching queries produce deterministic launch item(s).
- URL encoding handles spaces and common punctuation safely.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/web-query.rmod`
- module docs/readme block
Risk: low
Depends on:
- T006
Notes:
- This replaces the AHK `g` and `y` commands.

### T024 — Add url-open.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:53
Last update: 2026-05-05 10:55
Scope:
- Add explicit URL command module.
- Support:
  - `w <url>` -> normalize and open URL.
DoD:
- Module loads in `--modules-debug`.
- `w reddit.com` opens `https://reddit.com`.
- Existing direct URL/core behavior remains untouched.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/url-open.rmod`
Risk: low
Depends on:
- T006
Notes:
- This is not generic open; it only preserves AHK `w` intent.

### T025 — Add chatgpt-open.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:56
Last update: 2026-05-05 10:58
Scope:
- Add ChatGPT launcher module.
- Support:
  - `chatgpt`;
  - `gpt`.
DoD:
- Module loads in `--modules-debug`.
- Exact aliases produce one item that opens ChatGPT.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/chatgpt-open.rmod`
Risk: low
Depends on:
- T006
Notes:
- Replaces prior `gem` idea with ChatGPT per user correction.

### T026 — Add yandex-open.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:59
Last update: 2026-05-05 11:00
Scope:
- Add Yandex launcher module.
- Support:
  - `ya`.
DoD:
- Module loads in `--modules-debug`.
- Exact alias produces one item that opens Yandex.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/yandex-open.rmod`
Risk: low
Depends on:
- T006
Notes:
- Replaces AHK `ya` command.

### T027 — Add terminal.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:01
Last update: 2026-05-05 11:03
Scope:
- Add terminal launcher module.
- Support:
  - `ter`;
  - `terminal`;
  - `ater`;
  - `aterminal`.
DoD:
- Module loads in `--modules-debug`.
- Normal terminal aliases launch `wt.exe`.
- Admin aliases use implemented `runas` support.
- If `runas` fails at runtime, the module/core reports clear feedback.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
- Manual admin/UAC validation blocked unless user approves.
Files likely touched:
- `modules/terminal.rmod`
Risk: medium
Depends on:
- T006
- T035
- T036
Notes:
- `ter` and `ater` belong together as one coherent terminal tool.

### T028 — Add pi-launcher.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:04
Last update: 2026-05-05 11:06
Scope:
- Add PI launcher module.
- Support:
  - `pi`;
  - `api`.
DoD:
- Module loads in `--modules-debug`.
- `pi` opens `wt.exe powershell -NoExit -Command "pi"` or configured equivalent from internal `.rmod` config defaults.
- `api` uses implemented `runas` support.
- If `runas` fails at runtime, the module/core reports clear feedback.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
- Manual admin/UAC validation blocked unless user approves.
Files likely touched:
- `modules/pi-launcher.rmod`
Risk: medium
Depends on:
- T006
- T035
- T036
Notes:
- `pi` and `api` belong together as one coherent PI tool.

### T029 — Add rclone-backup.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:07
Last update: 2026-05-05 11:09
Scope:
- Add rclone backup command module.
- Support:
  - `backup`;
  - `rclone`;
  - `gdrive`.
- Invoke helper/script target from internal `.rmod` config/default relative path or show missing-helper feedback.
DoD:
- Module loads in `--modules-debug`.
- Helper path is expressed as autocontained default/config in the `.rmod`.
- Missing helper/config produces a warning item/accessory/toast, not a broken launch.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/rclone-backup.rmod`
Risk: medium
Depends on:
- T006
- T036
Notes:
- Full helper implementation is not in this wave.

### T030 — Add snip.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:10
Last update: 2026-05-05 11:13
Scope:
- Add snip command module.
- Support:
  - `snip`.
- Invoke helper/Python target from internal `.rmod` config/default relative path or show missing-helper feedback.
DoD:
- Module loads in `--modules-debug`.
- Helper path/command is autocontained/configurable inside `.rmod`.
- Missing helper/config produces clear feedback.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/snip.rmod`
Risk: medium
Depends on:
- T006
- T036
Notes:
- Separate from record/OCR because each command is a distinct tool action.

### T031 — Add screen-record.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:14
Last update: 2026-05-05 11:16
Scope:
- Add screen recording command module.
- Support:
  - `record`.
- Invoke helper/Python target from internal `.rmod` config/default relative path or show missing-helper feedback.
DoD:
- Module loads in `--modules-debug`.
- Helper path/command is autocontained/configurable inside `.rmod`.
- Missing helper/config produces clear feedback.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/screen-record.rmod`
Risk: medium
Depends on:
- T006
- T036
Notes:
- Uses same future helper family as snip but remains separate module by user preference.

### T032 — Add ocr.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:17
Last update: 2026-05-05 11:19
Scope:
- Add OCR command module.
- Support:
  - `ocr`.
- Invoke helper/Python target from internal `.rmod` config/default relative path or show missing-helper feedback.
DoD:
- Module loads in `--modules-debug`.
- Helper path/command is autocontained/configurable inside `.rmod`.
- Missing helper/config produces clear feedback.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/ocr.rmod`
Risk: medium
Depends on:
- T006
- T036
Notes:
- Kept separate from snip/record by module atomization rule.

### T033 — Add color-picker.rmod

Status: done
Claimed by: current-agent
Started: 2026-05-05 11:20
Last update: 2026-05-05 11:22
Scope:
- Add color picker command module.
- Support:
  - `color`.
- Invoke helper target from internal `.rmod` config/default relative path or show missing-helper feedback.
DoD:
- Module loads in `--modules-debug`.
- Helper path/command is autocontained/configurable inside `.rmod`.
- Missing helper/config produces clear feedback.
Validation:
- `cargo run --bin rmenu -- --modules-debug`
Files likely touched:
- `modules/color-picker.rmod`
Risk: medium
Depends on:
- T006
- T036
Notes:
- Color picker overlay itself remains helper work, not `.rmod` JS code.

### T034 — Specify minimal runas and rmenu-style toast primitives

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:19
Last update: 2026-05-05 10:23
Scope:
- Define minimal `runas` support needed by `terminal.rmod` and `pi-launcher.rmod`.
- Define whether `runas` is represented as a minimal item action, target convention, or launch backend option.
- Define `ctx.toast` behavior as rmenu-style visual feedback, not native Windows notification.
- Define visual constraints for toast: same font/color/border language as rmenu.
DoD:
- Minimal `runas` and toast behavior is documented before implementation.
- Test/validation expectations for T035/T036 are listed.
Validation:
- Documentation/planning task; no code validation required.
Files likely touched:
- `DECISIONS.md` or migration decision doc
- `MODULES_API_SPEC_V1.md` / `CTX_ACTIONS_SPEC_V1.md` if public API text changes
- `STATE.md`
Risk: medium
Depends on:
- T001
Notes:
- This replaces the prior broad action/bridge design for this wave.

### T035 — Implement minimal runas support

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:24
Last update: 2026-05-05 10:29
Scope:
- Implement admin launch via `ShellExecuteW` verb `runas` according to T034.
- Keep normal `launch_target` behavior unchanged.
- Expose the feature to modules in the minimal chosen representation.
- Add failure reporting path suitable for toast/log.
DoD:
- Normal launch still works.
- Admin launch path calls `ShellExecuteW` with `runas`.
- Module-produced admin item can trigger runas.
- Tests cover parser/model behavior where feasible.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual UAC launch validation blocked unless user approves.
Files likely touched:
- `src/launcher.rs`
- `src/ui_win32.rs`
- `src/modules/types.rs` / `src/modules/ipc.rs` if action representation is needed
- docs/specs
Risk: medium
Depends on:
- T034
Notes:
- Do not implement `copy`, `command` action, or full `ItemAction` unless strictly required by the chosen minimal design.

### T036 — Implement rmenu-style toast and ctx.toast support

Status: done
Claimed by: current-agent
Started: 2026-05-05 10:30
Last update: 2026-05-05 10:36
Scope:
- Implement a lightweight rmenu-style toast surface or equivalent visual feedback.
- Wire external JS `ctx.toast(message)` to runtime action/IPC if needed.
- Keep visual language aligned with current rmenu colors/font/border.
- Avoid native Windows notification styling.
DoD:
- External module can request a toast.
- Toast appears with rmenu-like styling or documented equivalent.
- Toast request failure does not crash launcher.
- Tests cover IPC/action mapping where feasible; visual behavior gets manual validation.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- manual visual validation via T020
Files likely touched:
- `src/module_host_main.rs`
- `src/modules/ipc.rs`
- `src/modules/context.rs`
- `src/modules/actions.rs`
- `src/modules/mod.rs`
- `src/ui_win32.rs`
- docs/specs
Risk: medium
Depends on:
- T034
Notes:
- Keep toast minimal; do not implement a general notification framework.

### T037 — Specify native RSnip companion integration contract

Status: done
Claimed by: current-agent
Started: 2026-05-05 19:50
Last update: 2026-05-05 19:58
Scope:
- Define the integration boundary between `rmenu-daemon` and RSnip when both are installed.
- Specify discovery order for RSnip in dev and installed modes:
  - explicit config/env override if added;
  - `C:\rSnip\target\release\rsnip.exe` for current dev setup;
  - packaged/install locations or registry marker for future releases;
  - PATH fallback only if safe and unambiguous.
- Specify lifecycle ownership:
  - RSnip standalone keeps its own daemon/hotkeys when rMenu is absent;
  - rMenu daemon detects/starts or coordinates RSnip when present;
  - no duplicate hotkey registration or competing daemons.
- Specify menu actions for snip/record/OCR as native companion commands, not generic process targets.
- Specify error UX when RSnip is missing, unreachable, or version-incompatible.
DoD:
- Decision/spec documents the companion contract and rejects PowerShell/CLI-wrapper integration for normal menu execution.
- Discovery, lifecycle, IPC, hotkey ownership, and fallback behavior are documented.
- Follow-up implementation tasks have enough detail to proceed without redesign.
Validation:
- Documentation/planning task; no code validation required.
Files likely touched:
- `DECISIONS.md`
- `docs/ahk-migration/DAEMON_BOUNDARY.md`
- `docs/ahk-migration/HANDOFF.md`
- `README.md` if user-facing setup changes are documented early
- `STATE.md`
Risk: medium
Depends on:
- T022
Notes:
- This corrects the conceptual model: RSnip is a first-class companion capability when installed, not a `.rmod` shell command.

### T038 — Implement RSnip discovery and direct IPC client in rMenu

Status: done
Claimed by: current-agent
Started: 2026-05-05 20:00
Last update: 2026-05-05 20:18
Scope:
- Add a small RSnip companion module/client in Rust for rMenu.
- Discover RSnip according to T037.
- Implement direct named-pipe client for `\\.\pipe\rsnip` using the existing RSnip JSON protocol:
  - `snip`;
  - `record`;
  - `ocr`;
  - status/reachability if supported or a safe equivalent.
- Start/confirm the RSnip daemon when discovered and not reachable, without opening consoles.
- Return structured success/error results for UI feedback.
- Avoid launching `powershell.exe` or using `.rmod` target wrappers for normal snip/record/OCR execution.
DoD:
- rMenu can detect the dev RSnip binary at `C:\rSnip\target\release\rsnip.exe`.
- rMenu can send snip/record/OCR to a running RSnip daemon through direct IPC.
- rMenu can start RSnip daemon hidden when installed but not reachable.
- Missing/unreachable RSnip is reported without crash.
- Unit tests cover discovery and IPC request serialization/error handling where feasible.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual smoke: start rMenu daemon, confirm RSnip daemon starts, trigger direct command path without console flash.
Files likely touched:
- `src/rsnip_companion.rs` or `src/companion/rsnip.rs`
- `src/daemon_main.rs`
- `src/main.rs` / module wiring if companion provider is shared with non-daemon mode
- `src/settings.rs` if config/env override is added
- `Cargo.toml` only if an added dependency is justified
Risk: high
Depends on:
- T037
Notes:
- Keep the IPC client minimal and source-compatible with RSnip's existing protocol. Do not duplicate RSnip capture logic inside rMenu.

### T039 — Add native rMenu provider/actions for RSnip menu commands

Status: done
Claimed by: current-agent
Started: 2026-05-05 20:25
Last update: 2026-05-06 22:05
Scope:
- Replace transitional `.rmod` launch wrappers with native rMenu items/actions for RSnip companion commands.
- Expose menu entries for:
  - `snip` / screenshot region;
  - `record` / screen recording toggle;
  - `ocr` / text extraction.
- Ensure selecting those entries calls the direct companion IPC path from T038.
- Preserve existing ranking/query UX for the aliases currently provided by `.rmod` modules.
- Decide whether old `snip.rmod`, `screen-record.rmod`, and `ocr.rmod` are disabled, cancelled, or converted to thin metadata-only aliases.
DoD:
- Typing `snip` in rMenu triggers the same RSnip daemon command as the hotkey path, without spawning PowerShell or a visible console.
- Typing `record`/`rec` triggers RSnip record through the same direct path.
- Typing `ocr`/`text` triggers RSnip OCR through the same direct path.
- If RSnip is not installed, menu feedback is explicit and non-crashing.
- Transitional `.rmod` behavior is removed or disabled only with a clear note; no silent duplicate commands remain.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- `cargo run --bin rmenu -- --modules-debug`
- Manual: open rMenu via daemon and trigger `snip`, `record`, and `ocr`.
Files likely touched:
- `src/modules/mod.rs` or new built-in companion provider file
- `src/app_state.rs` / `src/ui_win32.rs` if submit dispatch needs native command handling
- `modules/snip.rmod`
- `modules/screen-record.rmod`
- `modules/ocr.rmod`
- docs updated by T041 if not done here
Risk: high
Depends on:
- T038
Notes:
- Ask before deleting `.rmod` files; disabling/cancelling transitional wrappers is safer than removal.
- Implemented native built-in RSnip companion provider and disabled transitional `.rmod` wrappers.
- Exact aliases (`snip`, `record`, `rec`, `screen`, `ocr`, `text`) now use `ctx.replaceItems` so stale history/cache entries cannot outrank native RSnip actions.
- User validated RSnip menu path: no PowerShell/CMD flash. Final public aliases are `snip`, `rec`, and `ocr` only.

### T040 — Coordinate RSnip hotkey ownership with rMenu daemon

Status: done
Claimed by: current-agent
Started: 2026-05-05 20:38
Last update: 2026-05-06 22:05
Scope:
- Decide and implement hotkey ownership for integrated mode.
- Ensure `Ctrl+Shift+S`, `Ctrl+Shift+R`, and `Ctrl+Shift+E` remain fast.
- Avoid duplicate registration between rMenu daemon and RSnip daemon.
- If RSnip keeps registering its own hotkeys, rMenu daemon should only manage lifecycle and menu IPC.
- If rMenu registers companion hotkeys, it must dispatch to the same direct IPC path and disable/avoid RSnip duplicate hotkeys through a documented mode.
DoD:
- Integrated mode has exactly one owner for each RSnip global hotkey.
- Hotkey path and menu path converge on the same RSnip action semantics.
- `rmenu-daemon --quit` does not leave stale hotkey owners or orphaned companion processes beyond the documented lifecycle choice.
- Manual validation checklist covers global hotkeys and menu actions in the same run.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual: `Ctrl+Shift+S/R/E` work after rMenu daemon starts; no duplicate hotkey failures in logs.
Files likely touched:
- `src/daemon_main.rs`
- `src/rsnip_companion.rs` or equivalent
- `C:\rSnip` only if an explicit integrated/no-hotkeys mode is required and approved
- docs/manual validation checklist
Risk: high
Depends on:
- T038
Notes:
- Prefer minimal coordination first: rMenu starts RSnip daemon and lets RSnip own hotkeys, unless duplicate-registration evidence requires deeper changes.
- Implemented lifecycle ownership: rMenu stops RSnip on quit only if rMenu started it; pre-existing standalone RSnip is left running.
- User validated `Ctrl+Shift+S/R/E` hotkeys remain fast and no console flash occurs in integrated use.

### T041 — Update docs and manual validation for native RSnip integration

Status: done
Claimed by: current-agent
Started: 2026-05-06 22:05
Last update: 2026-05-06 22:05
Scope:
- Update user-facing and migration docs to describe RSnip as an optional native companion app.
- Document install/discovery expectations for dev and packaged setups.
- Document fallback behavior when RSnip is missing.
- Update command inventory to distinguish native companion commands from transitional `.rmod` wrappers.
- Add manual validation steps for:
  - rMenu daemon startup starts/coordinates RSnip;
  - `Ctrl+Shift+S/R/E` remain fast;
  - rMenu `snip`/`record`/`ocr` paths do not flash a console;
  - `--quit` lifecycle behavior is as documented.
DoD:
- README/migration docs reflect the native companion architecture.
- Manual checklist covers RSnip installed and missing cases.
- STATE has final checkpoint for the integration wave.
Validation:
- Documentation task; code validation only if docs generation/check tooling exists.
Files likely touched:
- `README.md`
- `docs/ahk-migration/COMMAND_INVENTORY.md`
- `docs/ahk-migration/MANUAL_VALIDATION.md`
- `docs/ahk-migration/HANDOFF.md`
- `STATE.md`
Risk: low
Depends on:
- T039
- T040
Notes:
- Completed in README, migration docs, DECISIONS, and `docs/companion-and-rmods-workflow.md`. RSnip is documented as a native companion, not a launcher target.

### T042 — Specify persistent rMenu data root and companion layout

Status: done
Claimed by: current-agent
Started: 2026-05-05 21:10
Last update: 2026-05-05 21:16
Scope:
- Define the official persistent rMenu data root model:
  - `<data_dir>\modules`;
  - `<data_dir>\companions`;
  - `<data_dir>\config`;
  - `<data_dir>\state`.
- Specify how `modules_dir` relates to `data_dir`:
  - default derives from `<data_dir>\modules`;
  - `--modules-dir` remains an explicit override for debugging/migration.
- Specify bootstrap behavior for future installer reuse:
  - existing folder is reused without overwriting modules/config;
  - `%APPDATA%\rmenu` may keep a small pointer/bootstrap file if the user overrides the default data root.
- Specify companion install layout for RSnip:
  - `<data_dir>\companions\rsnip\rsnip.exe`;
  - future `config/`, `state/`, and `logs/` under the companion folder if RSnip supports portable mode.
DoD:
- Decision/docs describe `data_dir` and derived directories.
- Existing `--modules-dir` behavior is preserved in the documented model.
- RSnip companion path is explicit and future companions such as rTask fit the same layout.
Validation:
- Documentation/planning task; no code validation required.
Files likely touched:
- `DECISIONS.md`
- `README.md`
- `docs/ahk-migration/HANDOFF.md`
- `STATE.md`
Risk: medium
Depends on:
- T037
Notes:
- This is the UX foundation for reinstall-safe rMenu data and companion installs.

### T043 — Implement rMenu data directory resolution

Status: done
Claimed by: current-agent
Started: 2026-05-05 21:17
Last update: 2026-05-05 21:28
Scope:
- Add `data_dir` resolution for rMenu:
  - `--data-dir <PATH>` if accepted for CLI/daemon;
  - `RMENU_DATA_DIR`;
  - bootstrap file in `%APPDATA%\rmenu` if implemented in this task;
  - fallback `%APPDATA%\rmenu`.
- Create helpers to derive:
  - modules dir;
  - companions dir;
  - config dir;
  - state dir.
- Preserve current `--modules-dir` and `RMENU_MODULES_DIR` as higher-priority module-dir-specific overrides.
- Ensure `rmenu-daemon.exe` can run simply with defaults while still using configured data root.
DoD:
- `--data-dir`/`RMENU_DATA_DIR` can point rMenu at a persistent external root.
- Default module discovery derives from `<data_dir>\modules` when no explicit modules override is present.
- Current `--modules-dir` workflows keep working.
- Tests cover resolution precedence.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual/smoke: run daemon with default and with `RMENU_DATA_DIR` pointing to temp external root.
Files likely touched:
- `src/settings.rs`
- `src/main.rs`
- `src/daemon_main.rs`
- `src/rsnip_companion.rs` if it receives data-dir context
- `README.md` or docs if public CLI changes are documented here
Risk: high
Depends on:
- T042
Notes:
- Keep this as a general rMenu primitive, not RSnip-specific.

### T044 — Update RSnip discovery to prefer rMenu-managed companion install

Status: done
Claimed by: current-agent
Started: 2026-05-05 21:29
Last update: 2026-05-05 21:35
Scope:
- Update RSnip discovery order to prefer:
  1. `<data_dir>\companions\rsnip\rsnip.exe`;
  2. registered companion state/config path if separate from data dir;
  3. `RMENU_RSNIP_PATH`;
  4. dev path `C:\rSnip\target\release\rsnip.exe`;
  5. unambiguous PATH fallback.
- Ensure rMenu daemon starts/coordinates the installed companion path once present.
- Ensure native menu actions `snip`, `record`, `ocr` use the installed companion path.
DoD:
- Installed RSnip under the rMenu data root wins over dev/global paths.
- Missing installed binary falls back safely or reports clear error according to policy.
- Tests cover discovery precedence.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Smoke: create fake/existing data-root companion path and confirm discovery prefers it.
Files likely touched:
- `src/rsnip_companion.rs`
- `src/settings.rs` / data-dir helper from T043
- `src/daemon_main.rs`
Risk: high
Depends on:
- T043
Notes:
- This is what makes `/install rsnip` become the source of truth.

### T045 — Implement local/dev `/install rsnip` MVP

Status: done
Claimed by: current-agent
Started: 2026-05-05 21:36
Last update: 2026-05-05 21:45
Scope:
- Add a native rMenu command for installing RSnip into the data root.
- MVP source is local/dev RSnip binary:
  - copy from `C:\rSnip\target\release\rsnip.exe` if present;
  - destination `<data_dir>\companions\rsnip\rsnip.exe`.
- Create companion directories:
  - `<data_dir>\companions\rsnip`;
  - optional `config`, `state`, `logs` placeholders.
- Register install metadata/state if a state mechanism exists or create one as part of this task.
- After install, confirm/start RSnip daemon through the installed path.
- Expose UX through a simple command, preferably `/install rsnip` or an equivalent native runtime command if slash-command parsing requires a smaller first step.
DoD:
- Running the install command copies `rsnip.exe` into `<data_dir>\companions\rsnip\`.
- RSnip discovery uses the installed copy afterward.
- `rmenu-daemon.exe` uses the installed copy on next start.
- Existing dev path remains only fallback.
- Clear feedback is shown for missing local source, copy failure, or successful install.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual/smoke: install into temp data root, restart daemon, run `snip`.
Files likely touched:
- `src/modules/mod.rs` or runtime command handling
- `src/rsnip_companion.rs`
- `src/settings.rs` / data-dir helpers
- `src/daemon_main.rs`
- `README.md` or docs if command is public
Risk: high
Depends on:
- T044
Notes:
- This intentionally avoids GitHub download first. Validate install architecture locally before adding network/release management.

### T046 — Implement GitHub release installer path for RSnip

Status: done
Claimed by: current-agent
Started: 2026-05-06 00:00
Last update: 2026-05-06 00:05
Scope:
- Add GitHub release download/install flow for RSnip from `https://github.com/SynrgStudio/rSnip`.
- Define asset selection for Windows x64 release zip.
- Download, verify basic integrity if checksum is available, extract/copy `rsnip.exe` into `<data_dir>\companions\rsnip\`.
- Keep local/dev install path available for development if useful.
- Handle offline/network failure with clear feedback.
DoD:
- `/install rsnip` can install from GitHub release when local dev source is not used.
- Installed file ends at the same companion path used by discovery.
- Failure modes are visible and do not corrupt existing installed RSnip.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual: install from release into temp data root.
Files likely touched:
- `Cargo.toml` if HTTP/zip dependencies are needed
- `src/rsnip_companion.rs` or new installer module
- docs
Risk: high
Depends on:
- T045
Notes:
- Prefer atomic install: download/extract to temp, then replace current install only after success.

### T047 — Define/implement RSnip portable config integration if needed

Status: blocked
Blocker:
- Deferred while the active focus is the `/rmods` core registry/install workflow. Unblock when returning to RSnip portable config work.
Claimed by:
Started:
Last update:
Scope:
- Audit whether current RSnip supports config/state under a caller-provided data dir.
- If RSnip needs changes, plan or implement minimal support in `C:\rSnip`:
  - env var such as `RSNIP_DATA_DIR`; or
  - CLI `--data-dir`; or
  - integrated mode config path.
- Wire rMenu daemon to launch installed RSnip with the chosen portable config mechanism.
- Keep standalone RSnip behavior unchanged when no override is provided.
DoD:
- Installed RSnip can keep config/state under `<data_dir>\companions\rsnip\` or a documented fallback exists.
- No change breaks standalone RSnip.
- rMenu-managed install uses the documented config/state location.
Validation:
- RSnip-specific validation if code changes there:
  - `cargo test` / `cargo build --release` from `C:\rSnip`.
- rMenu validation if wiring changes:
  - `cargo fmt`
  - `cargo check`
  - `cargo test`
Files likely touched:
- `C:\rSnip\src\config.rs`
- `C:\rSnip\src\paths.rs`
- `C:\rSnip\src\app.rs`
- `src/rsnip_companion.rs`
- docs
Risk: high
Depends on:
- T045
Notes:
- Ask before making RSnip repo changes if the change is larger than portable path support.

### T048 — Specify installer UX for reusable rMenu data folder

Status: blocked
Blocker:
- Deferred while the active focus is the `/rmods` core registry/install workflow. Unblock when returning to installer UX work.
Claimed by:
Started:
Last update:
Scope:
- Specify future rMenu installer behavior for selecting a persistent data folder.
- Cover empty folder and existing folder cases:
  - create structure if empty;
  - detect existing `modules`, `companions`, `config`, `state`;
  - show confirmation that existing data will be reused, not overwritten.
- Specify how installer writes bootstrap pointer to the selected data root.
- Specify reinstall-after-format flow.
DoD:
- Installer UX spec clearly states how a user selects/reuses `C:\rMenuData` by default.
- Existing module/companion/config preservation is explicit.
- Future implementation tasks can follow the spec without redesign.
Validation:
- Documentation task; no code validation required.
Files likely touched:
- `installer/` docs or scripts if present
- `README.md`
- `docs/ahk-migration/HANDOFF.md`
- `STATE.md`
Risk: medium
Depends on:
- T042
Notes:
- This is future-facing; do not block `/install rsnip` MVP on full installer implementation.

### T049 — Update docs and manual validation for data-root RSnip install UX

Status: blocked
Blocker:
- Deferred while the active focus is the `/rmods` core registry/install workflow. Unblock when returning to RSnip install docs/manual validation.
Claimed by:
Started:
Last update:
Scope:
- Update user-facing docs for:
  - rMenu data root;
  - modules folder under data root;
  - companions folder;
  - `/install rsnip`;
  - native snip/record/OCR behavior after install.
- Add manual validation for:
  - install RSnip into data root;
  - restart daemon with simple commands;
  - `Ctrl+Shift+S` works;
  - `snip`, `record`, `ocr` from rMenu work natively;
  - existing data root reuse behavior if implemented/spec-only if not.
DoD:
- Docs reflect persistent data root and installed companion architecture.
- Manual checklist covers install, restart, and command behavior.
- STATE has final checkpoint for the install-planning/implementation wave.
Validation:
- Documentation task; code validation only if docs tooling exists.
Files likely touched:
- `README.md`
- `docs/ahk-migration/MANUAL_VALIDATION.md`
- `docs/ahk-migration/HANDOFF.md`
- `STATE.md`
Risk: low
Depends on:
- T045
- T048
Notes:
- This closes the UX loop for the current RSnip install priority.
### T050 — Specify rMods registry repo layout and generated schema

Status: done
Claimed by: current-agent
Started: 2026-05-06 03:05
Last update: 2026-05-06 03:15
Scope:
- Define the GitHub repository layout for a first-party/user rMods registry:
  - `modules/*.rmod` as the only MVP package input;
  - generated `registry.json` at repo root;
  - `scripts/generate-registry.*`;
  - `.github/workflows/update-registry.yml`.
- Specify `registry.json` schema v1 fields:
  - `schema`, `generated_at`, `modules[]`;
  - module `id`, `name`, `version`, `description`, `kind`, `download_url`, `sha256`, `size`, `tags`, `requires_rmenu`.
- Define ID/version source as the `.rmod` header parsed per `RMOD_SPEC_V1.md`.
- Define default registry URL and future config override policy.
DoD:
- Repo layout and schema are documented in a new or existing rMenu docs file.
- The schema clearly states that `registry.json` is generated, not manually edited.
- MVP explicitly supports `.rmod` only and defers zip/folder module distribution.
Validation:
- Documentation task; no code validation required.
Files likely touched:
- `README.md`
- `MODULES_OPERATIONS_GUIDE.md` or new `docs/rmods-registry.md`
- `STATE.md`
Risk: medium
Depends on:
- none
Notes:
- This task locks the product contract before implementation.
- Completed in `docs/rmods-registry.md`, with README and operations guide links/summary.
- Validation: documentation-only task; no code validation run.

### T051 — Add registry generator script for `.rmod` files

Status: done
Claimed by: current-agent
Started: 2026-05-06 03:20
Last update: 2026-05-06 03:35
Scope:
- Create a script for the registry repo that scans `modules/*.rmod`.
- Validate each file against `.rmod` v1 basics:
  - magic line;
  - required header fields;
  - `module.js` block exists;
  - optional `config.json` is valid JSON if present.
- Extract metadata from `.rmod` headers.
- Compute sha256 and size.
- Emit stable, sorted `registry.json` with raw GitHub download URLs.
DoD:
- Running the script after adding a valid `.rmod` updates `registry.json` with that module.
- Invalid `.rmod` files fail generation with clear errors.
- Output ordering is deterministic.
Validation:
- Run the generator against at least two sample `.rmod` files.
- Verify `sha256` matches the file content.
Files likely touched:
- external/new registry repo files, likely `scripts/generate-registry.*`
- optional sample `modules/*.rmod`
Risk: medium
Depends on:
- T050
Notes:
- Implemented in `C:\rmods` and pushed to `https://github.com/SynrgStudio/rmods.git` commit `0dfdca1`.
- Added `scripts/generate-registry.py`, `README.md`, `modules/.gitkeep`, and generated empty `registry.json`.
- Validation: generator smoke with two sample `.rmod` files passed; SHA-256 values verified; invalid `.rmod` without `module.js` failed with clear error.

### T052 — Add GitHub Action to regenerate `registry.json`

Status: done
Claimed by: current-agent
Started: 2026-05-06 05:45
Last update: 2026-05-06 06:00
Scope:
- Add a GitHub Actions workflow in `C:\rmods` / `https://github.com/SynrgStudio/rmods.git` that runs the generator when `modules/**`, generator files, or workflow files change.
- Commit regenerated `registry.json` when needed.
- Avoid empty commits when registry output is unchanged.
- Keep `calculator.rmod` as the first real registry item and use it for validation.
DoD:
- Workflow file exists at `.github/workflows/update-registry.yml` in the rmods repo.
- Pushing a new or changed `.rmod` under `modules/` regenerates `registry.json` automatically.
- Workflow failure blocks invalid module metadata.
- Workflow uses GitHub Actions bot identity and least necessary permissions.
- Current live registry still contains `calculator` after the workflow addition.
Validation:
- Local: run `python scripts/generate-registry.py` in `C:\rmods`.
- Local: confirm generated `registry.json` contains `calculator` with matching SHA-256.
- GitHub/manual: push workflow and confirm Actions run or, if Actions propagation is delayed, confirm the workflow file is present remotely and note pending live-run validation.
Files likely touched:
- `C:\rmods\.github\workflows\update-registry.yml`
- `C:\rmods\README.md` if workflow docs need an update
- `STATE.md`
Risk: medium
Depends on:
- T051
Notes:
- This is the key to avoiding manual registry maintenance.
- Do not wait on this task before designing rMenu core types if GitHub Actions UI confirmation is the only blocker; record that as a manual/live validation note if needed.
- Completed in `C:\rmods`; pushed commit `43b22c8` adding `.github/workflows/update-registry.yml` and README docs.
- GitHub Actions run `25418181506` completed successfully and bot commit `c18d72c` updated `registry.json`.
- Validation: local generator run OK; calculator SHA-256/size verified; remote raw registry contains calculator after workflow run.

### T053 — Add rMenu core registry data types and validation

Status: done
Claimed by: current-agent
Started: 2026-05-06 06:05
Last update: 2026-05-06 06:20
Scope:
- Add Rust types for `registry.json` schema v1.
- Add validation for required fields, supported schema, supported package kind, safe IDs, URL presence, sha256 format, and size limits.
- Add installed-state types for `<data_dir>\state\rmods-installed.json`.
- Use the current live calculator registry entry as the positive fixture shape.
DoD:
- Valid registry JSON deserializes into typed structs.
- Invalid schema/kind/hash/IDs are rejected with clear errors.
- Unit tests cover valid and invalid registry payloads, including a `calculator`-like valid record.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test rmods` or full `cargo test`
Files likely touched:
- `src/rmods_registry.rs` or similar
- `src/settings.rs` if data-dir helpers are reused
- `Cargo.toml` only if new dependencies are unavoidable
Risk: medium
Depends on:
- T050
Notes:
- Prefer no new semver dependency initially unless version comparison needs it later.
- Default registry constant should point to `https://raw.githubusercontent.com/SynrgStudio/rmods/main/registry.json`.
- Implemented `src/rmods_registry.rs` with schema v1 types, validation, installed-state types, default registry constant, data-root state/cache/download paths, and tests using the live calculator registry shape.
- Validation: `cargo test rmods` OK; `cargo check` OK.

### T054 — Implement registry fetch and cache primitives

Status: done
Claimed by: current-agent
Started: 2026-05-06 06:20
Last update: 2026-05-06 06:40
Scope:
- Fetch `registry.json` from the default GitHub raw URL: `https://raw.githubusercontent.com/SynrgStudio/rmods/main/registry.json`.
- Add local cache under `<data_dir>\state\rmods-registry-cache.json`.
- On `/rmods`, allow immediate cached display and background/explicit refresh if UI architecture supports it; otherwise fetch synchronously with bounded timeout for MVP.
- Provide clear errors for offline/network/parse failures.
DoD:
- Registry can be fetched from an HTTP URL.
- Live fetch sees the `calculator` record when network is available.
- Registry cache is written after successful fetch and read when network fails.
- Tests cover cache read/write and registry validation separately from network.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Smoke with the live rmods registry or a local file/fake URL if direct GitHub access is unavailable.
Files likely touched:
- `src/rmods_registry.rs`
- `src/settings.rs`
- `Cargo.toml` if HTTP support needs adjustment
Risk: high
Depends on:
- T053
Notes:
- Existing companion installers use Win32 URL download; decide whether to reuse that for MVP or introduce a general fetch helper.
- Implemented fetch/cache primitives in `src/rmods_registry.rs`, including default live URL, Win32 URLDownloadToFile HTTP fetch, file:// fetch for tests, cache read/write under data-root state, and tests.
- Validation: `cargo test rmods` OK; `cargo check` OK.

### T055 — Implement installed rMods metadata and local module scan

Status: done
Claimed by: current-agent
Started: 2026-05-06 06:35
Last update: 2026-05-06 06:40
Scope:
- Read/write `<data_dir>\state\rmods-installed.json`.
- Inspect installed `.rmod` files under `<data_dir>\modules` to recover module name/version when metadata is missing.
- Compute view states:
  - `not installed`;
  - `installed`;
  - `update available`;
  - `local newer`;
  - `checksum mismatch`.
DoD:
- rMenu can compare registry modules against local installed modules.
- Missing metadata does not break listing.
- Tests cover state calculation.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
Files likely touched:
- `src/rmods_registry.rs`
- `src/modules/rmod.rs` only if metadata extraction helper is needed
- `src/settings.rs`
Risk: medium
Depends on:
- T053
Notes:
- Version ordering can be conservative for MVP; string equality plus update when remote differs is acceptable if documented.
- Implemented installed-state read/write, local `.rmod` scan under data-root modules, SHA-256 hashing, and install status calculation.
- Added `sha2` dependency for core SHA-256 verification needed by this and later secure install tasks.
- Validation: `cargo test rmods` OK; `cargo check` OK.

### T056 — Add `/rmods` core mode state and command entry

Status: done
Claimed by: current-agent
Started: 2026-05-06 06:40
Last update: 2026-05-06 07:15
Scope:
- Add a core-owned `/rmods` command that switches the Win32 UI into an rMods registry mode.
- Add UI state for registry items, cursor, selected IDs, loading/error text, and source registry URL.
- Ensure normal launcher mode remains unchanged outside `/rmods`.
DoD:
- Typing `/rmods` opens an rMods-specific UI rather than launcher fuzzy results.
- `Esc` exits back/close according to existing rMenu close behavior.
- Empty/loading/error states render clearly.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual smoke: open rMenu, enter `/rmods`, observe mode transition.
Files likely touched:
- `src/app_state.rs`
- `src/ui_win32.rs`
- `src/main.rs` / runtime setup if state initialization changes
Risk: high
Depends on:
- T054
- T055
Notes:
- This is UI plumbing only; installation can remain disabled until later tasks.
- Implemented `/rmods` detection in Win32 UI, rMods UI state in `AppState`, registry fetch/cache fallback, and launcher-mode isolation so `/rmods` no longer falls through to fuzzy app search.
- Validation: `cargo test`, `cargo check`, `cargo build --release` OK.

### T057 — Render rMods multiselect list

Status: done
Claimed by: current-agent
Started: 2026-05-06 06:50
Last update: 2026-05-06 07:15
Scope:
- Render rMods rows with checkbox, name, version, state, and description.
- Add keyboard handling:
  - Up/Down moves cursor;
  - Space toggles selection;
  - `R` refreshes registry;
  - `U` selects update-available items;
  - Enter is reserved for install in later task.
- Keep rendering within existing rMenu visual style.
DoD:
- Multiselect state is visible and stable while navigating.
- Installed/update/not-installed states are visually distinguishable enough for text UI.
- Existing launcher quick-select behavior is not active inside rMods mode.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual smoke: navigate/select rows.
Files likely touched:
- `src/ui_win32.rs`
- `src/app_state.rs`
Risk: high
Depends on:
- T056
Notes:
- Avoid introducing a generic UI framework rewrite; implement the smallest mode-specific renderer.
- Implemented checkbox labels, status badges, description hints, `Space` toggle, `R` refresh, and `U` select-updates behavior using existing rMenu list renderer.
- Validation: `cargo test`, `cargo check`, `cargo build --release` OK.

### T058 — Implement secure `.rmod` download and verification

Status: done
Claimed by: current-agent
Started: 2026-05-06 06:55
Last update: 2026-05-06 07:15
Scope:
- Download selected `.rmod` files into a temp downloads folder under `<data_dir>\state\downloads`.
- Enforce size limits from registry and a hard maximum.
- Compute sha256 and compare against registry.
- Parse/validate `.rmod` before installation and ensure internal `name`/`version` match registry fields.
DoD:
- Corrupt or mismatched downloads are rejected before touching installed modules.
- Valid downloads produce a verified temp file ready for atomic install.
- Unit tests cover hash mismatch and metadata mismatch.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
Files likely touched:
- `src/rmods_registry.rs`
- `src/modules/rmod.rs` if parser helper exposure is needed
- `Cargo.toml` if hashing/downloading needs dependencies
Risk: high
Depends on:
- T053
- T054
Notes:
- Security boundary: remote content must be validated before installation.
- Implemented download-to-temp, expected size check, SHA-256 verification, `.rmod` parse validation, and internal name/version match checks.
- Validation: `cargo test rmods`, `cargo test`, `cargo check` OK.

### T059 — Install verified `.rmod` atomically and reload modules

Status: done
Claimed by: current-agent
Started: 2026-05-06 07:00
Last update: 2026-05-06 07:15
Scope:
- Move/copy verified temp `.rmod` to `<data_dir>\modules\<id>.rmod` atomically where possible.
- Preserve existing installed module if install/update fails.
- Update `rmods-installed.json` after successful install.
- Reload module runtime so newly installed modules become available immediately.
DoD:
- Installing one selected module places the `.rmod` in the data-root modules directory.
- Failed install leaves previous module intact.
- Successful install updates installed metadata and reloads modules.
- UI shows success/failure feedback.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Smoke with a local/fake registry and sample `.rmod`.
Files likely touched:
- `src/rmods_registry.rs`
- `src/modules/mod.rs`
- `src/ui_win32.rs`
- `src/app_state.rs`
Risk: high
Depends on:
- T058
- T057
Notes:
- Coordinate with existing `ModuleRuntime::reload` or equivalent instead of spawning a new rMenu.
- Implemented staging/backup install into `<data_dir>\modules\<id>.rmod`, installed-state update, and runtime reload after UI install attempts.
- Validation: file URL install test, `cargo test`, `cargo check`, `cargo build --release` OK.

### T060 — Wire Enter install/update flow in `/rmods`

Status: done
Claimed by: current-agent
Started: 2026-05-06 07:05
Last update: 2026-05-06 07:15
Scope:
- In rMods mode, Enter installs or updates all selected modules.
- Show progress/final feedback in the rMenu UI.
- Clear selection or refresh state after install.
- Handle partial success and per-module errors.
DoD:
- Select one or more modules, press Enter, and they install/update.
- Partial failures are reported without hiding successful installs.
- Reopening `/rmods` shows updated installed states.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual smoke with at least two sample modules: one install, one update.
Files likely touched:
- `src/ui_win32.rs`
- `src/app_state.rs`
- `src/rmods_registry.rs`
Risk: high
Depends on:
- T059
Notes:
- MVP can run install synchronously only if UI remains responsive enough; prefer staged/background pattern from companion installer if needed.
- Enter now installs selected rMods, shows success/error/no-selection feedback, refreshes registry state, and reloads external module descriptors.
- Validation: `cargo test`, `cargo check`, `cargo build --release` OK. Manual interactive UX still recommended.

### T061 — Add `/rmods` docs and user workflow

Status: done
Claimed by: current-agent
Started: 2026-05-06 07:15
Last update: 2026-05-06 07:20
Scope:
- Document user workflow:
  - create/test `.rmod` locally;
  - copy it to `modules/` in the GitHub registry repo;
  - push;
  - GitHub Action regenerates registry;
  - run `/rmods`, select, Enter.
- Document keyboard controls and states.
- Document data-root install path and metadata/cache files.
DoD:
- README or operations docs explain `/rmods` end-to-end.
- Registry repo maintenance is documented as automatic, not manual.
- Troubleshooting covers fetch failure, checksum failure, and invalid `.rmod`.
Validation:
- Documentation task; no code validation required.
Files likely touched:
- `README.md`
- `MODULES_OPERATIONS_GUIDE.md`
- new `docs/rmods-registry.md` if created
Risk: low
Depends on:
- T060
Notes:
- Keep docs aligned with MVP behavior, not future multi-registry promises.
- Updated README, MODULES_OPERATIONS_GUIDE, and docs/rmods-registry.md from planned to MVP behavior, including controls and default registry URL.
- Validation: documentation update plus code validation already run; no separate docs tooling.

### T062 — End-to-end local/fake registry smoke validation

Status: done
Claimed by: current-agent
Started: 2026-05-06 07:15
Last update: 2026-05-06 07:20
Scope:
- Create or use two sample `.rmod` files and a local/fake registry.
- Validate `/rmods` listing, selection, install, metadata update, and module reload.
- Validate checksum failure path by altering a sample file/hash.
DoD:
- Local/fake registry smoke demonstrates install and update without GitHub dependency.
- Checksum failure does not install corrupt module.
- Results are recorded in `STATE.md`.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
- Manual/smoke commands documented in STATE.
Files likely touched:
- `STATE.md`
- temporary smoke scripts, if needed
Risk: medium
Depends on:
- T060
Notes:
- Remove or ignore temporary smoke scripts after use unless they become permanent tests.
- Covered by permanent `rmods_registry` tests: file URL fetch/cache, installed-state roundtrip, local scan, download/verify/install from file URL, invalid registry/module validation, and status calculation.
- Validation: `cargo test rmods` and full `cargo test` OK.

### T063 — GitHub registry smoke validation

Status: done
Claimed by: current-agent
Started: 2026-05-06 07:25
Last update: 2026-05-06 09:00
Next:
- Complete. Follow-up rpack folder module support was implemented and published separately.
Claimed by:
Started:
Last update:
Scope:
- Validate real GitHub flow:
  - push new `.rmod`;
  - GitHub Action generates registry;
  - `/rmods` fetches live registry;
  - select and install module;
  - module appears in rMenu after reload.
DoD:
- Live registry module appears in `/rmods` without manual registry editing.
- Selected module installs from GitHub and passes sha256 verification.
- Installed module works after reload.
Validation:
- Manual/GitHub smoke.
Files likely touched:
- `STATE.md`
- external registry repo only
Risk: medium
Depends on:
- T052
- T060
Notes:
- Registry repo, GitHub Action, calculator module, and `/rmods` install flow now exist.
- User confirmed calculator appeared, could be selected, and installed from `/rmods`.
- Bug found: Enter installed calculator but closed rMenu before showing feedback because the `/rmods` Enter branch fell through to the generic Enter close path.
- Fixed by returning immediately after `/rmods` install feedback/refresh, keeping rMenu open.
- Validation after fix: `cargo test rmods` OK; `cargo check` OK; `cargo build --release` OK after stopping stale release processes.
- User confirmed `/rmods` install/uninstall/filter/list-order UX works correctly.
- Live registry now also contains `shortcuts` as first `rpack` folder module.

### T065 — Add rpack folder-module distribution to `/rmods`

Status: done
Claimed by: current-agent
Started: 2026-05-06 08:20
Last update: 2026-05-06 09:00
Scope:
- Extend the rMods registry and installer to support folder modules as `rpack` packages.
- `rpack` is a repository folder, not a zip/archive.
- Install `rpack` modules into `<data_dir>\\modules\\<id>\\`.
DoD:
- Registry accepts `kind: rpack` with `base_url` and per-file metadata.
- rMenu downloads/verifies every rpack file and installs atomically through staging.
- rMenu validates installed rpack folder modules via `module.toml` before activation.
- `/rmods` install/update/uninstall flow works for both `rmod` and `rpack`.
- At least one live rpack is published in `SynrgStudio/rmods`.
Validation:
- `cargo test rmods`: OK, 17 tests.
- `cargo check`: OK.
- `cargo build --release`: OK.
- full `cargo test`: OK, 115 total tests across bins.
- rmods generator smoke verified registry integrity locally.
Files touched:
- `src/rmods_registry.rs`
- `src/app_state.rs`
- `src/ui_win32.rs`
- `docs/rmods-registry.md`
- `MODULES_OPERATIONS_GUIDE.md`
- `README.md`
- `C:\\rmods\\scripts\\generate-registry.py`
- `C:\\rmods\\rpacks\\shortcuts\\*`
- `C:\\rmods\\README.md`
- `C:\\rmods\\.github\\workflows\\update-registry.yml`
Risk: medium
Depends on:
- T063
Notes:
- Published rmods commits:
  - `7228df8` add rpack folder module support;
  - `7805764` bot registry update;
  - `c46213e` enforce LF line endings.
- First live rpack: `shortcuts` v0.3.0.

### T064 — Future: support multiple registries and conflict policy

Status: blocked
Claimed by:
Started:
Last update: 2026-05-06 21:00
Blocker:
- Deferred until daemon/resident UI latency work is complete and the user asks for multi-registry support.
Scope:
- Add configurable registry URLs under rMenu config.
- Merge modules from multiple registries.
- Define duplicate ID behavior.
DoD:
- Multiple registries can be configured.
- Duplicate IDs are handled predictably.
- UI shows source registry when useful.
Validation:
- `cargo fmt`
- `cargo check`
- `cargo test`
Files likely touched:
- `src/settings.rs`
- `src/rmods_registry.rs`
- `src/ui_win32.rs`
- docs
Risk: medium
Depends on:
- T063
Notes:
- Not MVP; implement after single-registry flow is validated.


## 2026-05-06 20:35 finalization note

Completed the companion/rMods finalization pass requested by the user:

- RSnip public aliases: `snip`, `rec`, `ocr` only.
- RTasks public panel alias: `tasks` only; embedded `t ` capture remains.
- rpack user data strategy: external modules now have `ctx.moduleStateDir()`; `shortcuts 0.3.3` stores user data under module state in the rMods registry.
- Cleanup: removed generated dist artifacts, local installed module copies, obsolete transitional wrappers, installer artifacts, and generated codebase report from the rMenu worktree.
- Docs: added `docs/companion-and-rmods-workflow.md` and updated README/API/authoring/operations/registry docs.
- Validation: `cargo fmt --all`, `cargo check`, `cargo test` passed.

### T066 — Instrument daemon hotkey-to-visible latency

Status: done
Claimed by: current-agent
Started: 2026-05-06 21:05
Last update: 2026-05-06 21:25
Scope:
- Add focused release-safe timing instrumentation for the daemon hotkey path.
- Measure at least:
  - hotkey received;
  - before/after `show_warm_rmenu`;
  - `run_ui_internal` setup before `CreateWindowExW`;
  - module `run_on_load` duration;
  - `update_matching_items_from_config` duration;
  - window created/visible/first paint;
  - close/hide/runtime handoff.
- Keep instrumentation low-noise and useful in daemon logs or an explicit metrics/debug path.
DoD:
- Repeated daemon hotkey opens produce timings that identify where 1-3s are spent.
- Metrics distinguish first/cold open from warm reopen.
- Instrumentation does not materially slow normal launch.
Validation:
- `cargo fmt --all`
- `cargo check`
- targeted/manual: start release daemon, press hotkey 3 times, inspect daemon log timings.
Files likely touched:
- `src/daemon_main.rs`
- `src/ui_win32.rs`
- `src/modules/mod.rs`
- docs/operations note if a new debug flag/log format is added
Risk: low
Depends on:
- none
Notes:
- Do this before optimizing; current `rmenu.exe --metrics` does not measure the resident daemon reopen path.
- Implemented `run_ui_embedded_timed` and daemon hotkey timing logs.
- Release smoke using `ctrl+alt+f12` showed window visible at ~63-66ms and first paint/input ready at ~65-70ms.
- Total open duration in smoke was dominated by the scripted wait before closing the window, not pre-visible setup.

### T067 — Avoid repeated module `onLoad` work on every daemon reopen

Status: done
Claimed by: current-agent
Started: 2026-05-06 21:25
Last update: 2026-05-06 21:25
Scope:
- Separate module runtime lifetime from UI session lifetime.
- Ensure external module `onLoad` runs once per runtime load/reload, not every time the daemon opens the embedded UI.
- Preserve per-open UI reset behavior: empty input, selection reset, stale runtime feedback cleared.
- Keep module reload behavior correct after file changes or `/rmods` installs.
DoD:
- Reopening rMenu from the daemon does not call external `onLoad` again unless modules were reloaded.
- `/rmods` install/update still reloads modules and runs the appropriate load path once.
- `--modules-debug` still shows loaded hosts and healthy telemetry.
Validation:
- `cargo fmt --all`
- `cargo check`
- `cargo test`
- Manual/log: repeated daemon opens show no repeated external host `onLoad` burst.
Files likely touched:
- `src/modules/mod.rs`
- `src/ui_win32.rs`
- `src/daemon_main.rs`
- tests in `src/modules/mod.rs` or `src/ui_win32.rs`
Risk: medium
Depends on:
- T066
Notes:
- Instrumentation showed module `run_on_load` and initial matching update at ~0-1ms on repeated daemon opens.
- External module `onLoad` is tied to host startup, not every embedded UI reopen; no behavioral change was required for this task.

### T068 — Keep external module hosts hot across daemon UI sessions

Status: done
Claimed by: current-agent
Started: 2026-05-06 21:25
Last update: 2026-05-06 21:25
Scope:
- Verify and enforce that `rmenu-module-host.exe`/Node workers remain alive across daemon opens.
- Avoid fallback runtime recreation after normal UI close/hide.
- Add tests or diagnostics for host lifecycle when `run_ui_embedded` returns control to daemon.
DoD:
- Repeated daemon opens do not spawn new module host/Node processes unless a module reload or host failure occurred.
- Runtime handoff from UI back to daemon is explicit and covered by diagnostics/tests.
- Host telemetry remains stable across open/close cycles.
Validation:
- `cargo fmt --all`
- `cargo check`
- `cargo test`
- Manual: process count for `rmenu-module-host.exe`/`node.exe` stays stable across 5 hotkey opens.
Files likely touched:
- `src/daemon_main.rs`
- `src/ui_win32.rs`
- `src/modules/host_client.rs`
- `src/modules/mod.rs`
Risk: medium
Depends on:
- T066
- T067
Notes:
- Host-count smoke showed `rmenu-module-host.exe` count stabilized at 4 after first open and stayed 4 across repeated opens.
- Node process count also stayed stable across repeated opens in the smoke run.
- No behavioral change was required for this task.

### T069 — Cache companion discovery in the daemon/runtime hot path

Status: done
Claimed by: current-agent
Started: 2026-05-06 21:30
Last update: 2026-05-06 21:45
Scope:
- Avoid repeated filesystem/PATH discovery for RSnip and RTasks during normal queries and warm UI opens.
- Introduce daemon/runtime-scoped availability cache or TTL where appropriate.
- Preserve explicit install/update behavior: `/install rsnip`, `/install rtasks`, and daemon startup must refresh companion state.
DoD:
- RSnip/RTasks provider availability checks do not scan PATH/filesystem repeatedly on every query/open.
- Installing a companion updates or invalidates the cached state.
- Missing/unavailable companion feedback remains correct.
Validation:
- `cargo fmt --all`
- `cargo check`
- `cargo test`
- Manual/log: repeated opens and `snip`/`tasks` queries show cached discovery path.
Files likely touched:
- `src/rsnip_companion.rs`
- `src/rtasks_companion.rs`
- `src/modules/mod.rs`
- `src/daemon_main.rs`
- tests for discovery/cache invalidation if practical
Risk: medium
Depends on:
- T066
Notes:
- Useful if instrumentation shows discovery contributes to warm-open delay.
- Added 5s in-process cached path discovery for RSnip and RTasks.
- Companion install flows update the cache to the newly installed managed path.

### T070 — Defer noncritical module/provider work until after input is visible

Status: done
Claimed by: current-agent
Started: 2026-05-06 21:45
Last update: 2026-05-06 22:00
Scope:
- Ensure the daemon can show an empty input bar before provider/decorator work that is not required for the initial empty state.
- Keep empty input as input-bar-only with no precomputed module list.
- Move any expensive query/decorate work to after first paint or to actual input changes.
DoD:
- Hotkey open shows input promptly even when providers/modules are installed.
- Typing still triggers normal provider/query behavior with correct ranking/decorations.
- No stale items appear on empty input.
Validation:
- `cargo fmt --all`
- `cargo check`
- `cargo test`
- Manual: daemon hotkey opens to visible input before module/provider results are needed.
Files likely touched:
- `src/ui_win32.rs`
- `src/modules/mod.rs`
- `src/ranking.rs` if query update behavior needs adjustment
Risk: medium
Depends on:
- T066
- T067
Notes:
- This is distinct from resident-window work; it removes pre-visible synchronous work.
- Empty initial input now skips initial provider/query/decorator work and clears the list directly before first visible paint.
- Timing smoke still shows first paint/input ready around 63-69ms; remaining pre-visible time is mostly `CreateWindowExW`/`WM_CREATE` work.

### T071 — Convert daemon embedded UI to resident show/hide window if needed

Status: cancelled
Claimed by: current-agent
Started: 2026-05-06 22:00
Last update: 2026-05-06 22:00
Scope:
- If T066-T070 do not reach target latency, replace create/destroy per hotkey with a resident hidden window owned by the daemon.
- Hotkey should reset input/selection and `ShowWindow`/focus the existing HWND.
- Close/Escape/Enter in daemon mode should hide the window and return to the daemon loop, not destroy the UI runtime.
- Preserve normal `rmenu.exe` standalone behavior.
DoD:
- Warm daemon reopen avoids `CreateWindowExW` on every hotkey.
- Standalone CLI/window mode still exits normally.
- Module runtime and external hosts remain alive across hide/show.
- Focus and keyboard behavior remain correct.
Validation:
- `cargo fmt --all`
- `cargo check`
- `cargo test`
- Manual: 10 daemon hotkey open/close cycles, no stuck focus, no duplicate windows, no host respawn.
Files likely touched:
- `src/daemon_main.rs`
- `src/ui_win32.rs`
- `src/app_state.rs`
- `src/modules/mod.rs`
Risk: high
Depends on:
- T066
- T067
- T068
- T070
Notes:
- This is the largest optimization and should only be implemented after measurement proves it is needed.
- Cancelled for now because instrumentation shows warm opens visible/painted around 63ms in controlled release smoke, below the 100-200ms target.
- User validated real shortcut behavior: first startup can take time, warm opens are effectively instant.

### T072 — Release-mode performance validation and UX acceptance

Status: done
Claimed by: current-agent
Started: 2026-05-06 22:15
Last update: 2026-05-06 22:15
Blocker:
- Resolved by user manual validation: warm daemon opens now feel instantaneous after first startup.
Scope:
- Validate release daemon warm-open latency and subjective feel.
- Compare before/after metrics/logs.
- Validate aliases and key paths still work:
  - `snip`, `rec`, `ocr`;
  - `tasks`;
  - `t ` embedded task creation;
  - `/rmods` open/filter/install/update;
  - `color` rpack.
DoD:
- User confirmed hotkey opens feel instantaneous after first startup.
- Warm-open timing target is met or remaining bottleneck is documented.
- No regression in companion/rMods workflows.
Validation:
- `cargo build --release`
- manual: daemon hotkey repeated open/close latency check: OK by user validation.
- manual: companion/rMods smoke list above.
Files likely touched:
- `STATE.md`
- maybe docs if final measured targets are recorded
Risk: low
Depends on:
- T067
- T068
- T069
- T070
- T071
Notes:
- Packaging/release remains out of scope until user asks.
