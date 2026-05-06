---
continuity_session: CONT-2026-05-04-1945-ahk-suite-rmenu-migration
created_at: 2026-05-04 19:45
updated_at: 2026-05-06 22:15
status: active
goal: Migrar la suite AHK hacia rmenu de forma nativa mediante core primitives, módulos, helpers y daemon futuro
---

# STATE — rmenu

## Current status

Implementation wave completed through final non-interactive validation and handoff. Follow-up daemon MVP is implemented. Queue has been replanned for native RSnip companion integration: rMenu should discover/coordinate RSnip when installed and execute snip/record/OCR through direct companion IPC instead of shell/CLI wrapper targets. T020 remains blocked on manual interactive launcher/hotkey validation and UAC approval.

Active session:

```text
continuity_session: CONT-2026-05-04-1945-ahk-suite-rmenu-migration
status: active
goal: Migrar la suite AHK hacia rmenu de forma nativa mediante core primitives, módulos, helpers y daemon futuro
```

## Last checkpoint

2026-05-06 10:00 — `color-picker` rpack implemented and published

- Implemented `color-picker` as isolated rpack in `SynrgStudio/rmods`.
- No rMenu-core color picker integration was added; rMenu only launches the rpack helper target.
- Added source helper project:
  - `C:\\rmods\\tools\\color-picker\\Cargo.toml`;
  - `C:\\rmods\\tools\\color-picker\\src\\main.rs`.
- Helper behavior:
  - `color-picker.exe --format hex|rgb|hsl|all`;
  - waits for left click;
  - samples pixel under cursor using Win32 `GetPixel`;
  - copies formatted color to clipboard;
  - `Esc` cancels.
- Added rpack:
  - `C:\\rmods\\rpacks\\color-picker\\module.toml`;
  - `module.js`;
  - `config.json`;
  - `README.md`;
  - `bin\\color-picker.exe`.
- rMenu commands exposed by module:
  - `color`;
  - `cp`;
  - `picker`;
  - `pick`;
  - optional formats: `hex`, `rgb`, `hsl`, `all`.
- Registry now contains 9 modules, including:
  - `color-picker:rpack`.
- Published commits:
  - `79eaa86` — add color-picker rpack;
  - `38d82b6` — bot update registry.
- GitHub Action success confirmed.

## Previous checkpoint

2026-05-06 09:30 — `/rmods` refresh/update hotkeys moved off plain letters

- User confirmed plain `R/r/U/u` are not viable as `/rmods` commands because `/rmods <query>` uses letters for filtering.
- Screenshots showed `R`, `r`, `U`, `u` becoming part of the input/filter (`/rmods shorR`, `/rmods r`, `/rmods u`).
- Changed `/rmods` controls:
  - refresh: `F5` or `Ctrl+R`;
  - mark update-available modules: `Ctrl+U`.
- Plain `r` and `u` now remain normal filter text.
- Updated UI empty-hint text and docs.
- Validation:
  - `cargo test ui_win32::tests`: OK;
  - `cargo check`: OK;
  - stopped live rMenu processes;
  - `cargo build --release`: OK.

## Previous checkpoint

2026-05-06 09:15 — slash command Enter precedence fixed for shortcuts bind

- User tested `shortcuts` rpack:
  - selected Blender item;
  - pressed Ctrl+B;
  - input changed to `/shortcuts::bind `;
  - typed `1` and pressed Enter;
  - nothing happened.
- Root cause:
  - Enter handling launched selected/matching items before dispatching slash commands.
  - `/shortcuts::bind 1` could still have a matching item list, so the command dispatch path was skipped.
- Fix:
  - slash commands now dispatch before selected-item launch in the generic Enter path.
  - `/install rsnip|rtasks` special handling remains intact.
  - selected-item launch still works for non-slash input.
- Validation:
  - full `cargo test`: OK, 115 tests across bins;
  - `cargo check`: OK;
  - initial `cargo build --release` was blocked by live `rmenu-daemon.exe` lock;
  - stopped `rmenu-daemon`, `rmenu`, `rmenu-module-host` via PowerShell;
  - `cargo build --release`: OK.

## Previous checkpoint

2026-05-06 09:00 — `rpack` folder modules implemented and published

- Defined and implemented `rpack` as folder-module distribution, not zip/archive.
- rMenu `/rmods` now supports registry package kinds:
  - `rmod` -> installs `<data_dir>\\modules\\<id>.rmod`;
  - `rpack` -> installs `<data_dir>\\modules\\<id>\\`.
- Registry schema v1 extended with `rpack` fields:
  - `base_url`;
  - per-file `files[]` with `path`, `sha256`, `size`;
  - aggregate `sha256` and total `size`.
- Added secure rpack install path:
  - validates file paths against traversal/absolute paths;
  - downloads every file to staging;
  - verifies per-file size/SHA-256;
  - validates `module.toml` with the existing directory module loader;
  - validates aggregate folder SHA-256;
  - atomically moves staging into `<data_dir>\\modules\\<id>`;
  - updates `rmods-installed.json` with package `kind`.
- Uninstall now removes both possible package forms for an ID:
  - `<id>.rmod`;
  - `<id>\\` rpack folder.
- Local scan now detects installed directory modules and calculates their aggregate rpack hash.
- Tests added:
  - file:// rpack install;
  - rpack scan/state kind preservation;
  - rpack uninstall;
  - unsafe rpack file path rejection.
- Validation:
  - `cargo test rmods`: OK, 17 tests;
  - `cargo check`: OK;
  - `cargo build --release`: OK;
  - full `cargo test`: OK, 115 total tests across bins.
- Updated docs:
  - `docs/rmods-registry.md`;
  - `MODULES_OPERATIONS_GUIDE.md`;
  - `README.md`.
- rmods registry repo updated and pushed:
  - commit `7228df8` — add rpack folder module support;
  - bot commit `7805764` — update generated registry;
  - commit `c46213e` — enforce LF line endings.
- Published first live rpack:
  - `shortcuts` version `0.3.0`;
  - source `C:\\rmods\\rpacks\\shortcuts`;
  - live registry now contains `shortcuts` with `kind: rpack`.

## Previous checkpoint

2026-05-06 08:15 — `/rmods` multiselect cursor retention and filtering implemented

- User confirmed `/rmods` pending-change install/uninstall UX works well.
- Fixed annoying selection jump:
  - Space now toggles the highlighted rMod without losing/restoring selection to the first item.
  - rMods selection is tracked by module ID across re-render.
  - WM_CHAR space generated after Space toggle is suppressed so it does not mutate `/rmods` input or reset selection.
- Added `/rmods` filter support:
  - `/rmods <query>` filters registry items with fuzzy scoring.
  - Example: `/rmods ca` matches `calculator`; `/rmods c` matches both `calculator` and `chatgpt-open`.
  - If already in `/rmods`, typing a letter directly auto-inserts the separator (`/rmods c`) for convenient filtering without sacrificing Space toggle behavior.
- Validation:
  - `cargo test rmods`: OK, 15 tests;
  - `cargo check`: OK;
  - `cargo build --release`: OK.

## Previous checkpoint

2026-05-06 07:55 — `/rmods` pending-change uninstall flow implemented

- Updated `/rmods` checkbox semantics per user request:
  - `[x]` = installed;
  - `[ ]` = not installed;
  - `[/]` = pending change.
- `Space` now toggles a pending action instead of a simple install selection:
  - not installed -> pending install;
  - installed/local newer/checksum mismatch -> pending uninstall;
  - update available -> pending update.
- `Enter` now applies pending changes and reports summary:
  - `rMods: installed N, updated N, removed N`.
- Added `uninstall_rmod` in `src/rmods_registry.rs`:
  - removes `<data_dir>\modules\<id>.rmod`;
  - removes installed-state entry;
  - validates safe module ID.
- Added tests for uninstall file/state removal.
- Updated docs for marker semantics.
- Validation:
  - `cargo test rmods`: OK, 15 tests;
  - `cargo check`: OK;
  - `cargo build --release`: OK.
- Manual re-check recommended: `/rmods`, Space on installed calculator should show `[/]`; Enter should remove it and refresh to `[ ]`.

## Previous checkpoint

2026-05-06 07:35 — `/rmods` install feedback close bug fixed

- User manually validated `/rmods` enough to confirm:
  - `calculator` appears;
  - `Space` selection works;
  - `Enter` installs `calculator`.
- Bug found: after pressing Enter, rMenu closed immediately and hid install feedback.
- Root cause:
  - `/rmods` Enter branch installed and set runtime feedback, but then fell through to the generic Enter handler's unconditional `request_ui_exit(hwnd, 0)`.
- Fix:
  - added an immediate `return LRESULT(0)` after `/rmods` install feedback/refresh.
  - rMenu should now stay open and show `Installed N rMod(s)` or error/no-selection feedback.
- Validation:
  - `cargo test rmods`: OK, 14 tests;
  - `cargo check`: OK;
  - initial `cargo build --release` was blocked by running release processes (`Access is denied` removing exe);
  - stopped stale `rmenu-daemon`, `rmenu`, `rmenu-module-host` processes;
  - `cargo build --release`: OK.
- T063 is now partial pending one manual re-check for visible feedback.

## Previous checkpoint

2026-05-06 07:20 — `/rmods` core MVP implemented through non-interactive validation

- Completed T053-T062.
- Added `src/rmods_registry.rs` with:
  - schema v1 registry structs and validation;
  - default registry URL `https://raw.githubusercontent.com/SynrgStudio/rmods/main/registry.json`;
  - installed-state/cache/download paths under data-root state;
  - registry HTTP fetch on Windows via `URLDownloadToFileW` plus `file://` test fetch;
  - cache read/write;
  - installed-state read/write;
  - local `.rmod` scan under `<data_dir>\modules`;
  - SHA-256 hashing via new `sha2` dependency;
  - install status calculation;
  - secure download/verify/install helper with size/hash/`.rmod` validation and staging/backup install.
- Added rMods UI state to `AppState`.
- Added `/rmods` Win32 UI behavior:
  - `/rmods` no longer falls through to normal fuzzy app search;
  - fetches live registry with cache fallback;
  - renders modules with checkbox, version, status badge, and description hint;
  - `Space` toggles selection;
  - `R` refreshes;
  - `U` selects update-available modules;
  - `Enter` installs selected modules, refreshes view state, updates installed metadata, and reloads external module descriptors.
- Updated docs:
  - `README.md`;
  - `MODULES_OPERATIONS_GUIDE.md`;
  - `docs/rmods-registry.md`.
- Validation:
  - `cargo test rmods`: OK, 14 tests;
  - `cargo check`: OK;
  - full `cargo test`: OK, 112 total tests across bins;
  - `cargo build --release`: OK.
- Remaining blocker:
  - T063 manual interactive GitHub smoke: open release rMenu, type `/rmods`, confirm calculator appears, select with Space, press Enter, and confirm install/load from GitHub.

## Previous checkpoint

2026-05-06 06:00 — T052 rMods registry GitHub Action added

- Completed T052.
- Added workflow in `C:\rmods`:
  - `.github/workflows/update-registry.yml`.
- Workflow behavior:
  - runs on pushes to `main` affecting `modules/**`, `scripts/generate-registry.py`, or the workflow file;
  - supports manual `workflow_dispatch`;
  - grants `contents: write` only;
  - runs `python scripts/generate-registry.py`;
  - verifies generated registry schema shape;
  - commits `registry.json` only when changed, avoiding empty commits.
- Updated `C:\rmods\README.md` with workflow behavior.
- Pushed `C:\rmods` commit:
  - `43b22c8` — `ci: regenerate rmods registry automatically`.
- GitHub Actions run completed successfully:
  - run id `25418181506`;
  - URL: `https://github.com/SynrgStudio/rmods/actions/runs/25418181506`;
  - bot follow-up commit `c18d72c` updated generated timestamp in `registry.json`.
- Fast-forwarded local `C:\rmods` to `origin/main` after the bot commit.
- Validation:
  - local `python scripts/generate-registry.py`: OK;
  - local calculator entry SHA-256 and size match `modules/calculator.rmod`: OK;
  - remote raw `registry.json` contains `calculator` after the workflow run: OK.
- Next executable `/rmods` task: T053 — Add rMenu core registry data types and validation.

## Previous checkpoint

2026-05-06 05:35 — Replanned `/rmods` from T052

- Replanned `ACTIVE_QUEUE.md` from T052 based on the now-live registry repo.
- Actual registry repo is now canonical:
  - local: `C:\rmods`;
  - remote: `https://github.com/SynrgStudio/rmods.git`;
  - raw registry: `https://raw.githubusercontent.com/SynrgStudio/rmods/main/registry.json`.
- Updated T052 to target the real `C:\rmods` repo and use `calculator.rmod` as the first validation module.
- Updated T053/T054 notes to use the live calculator registry shape and raw registry URL.
- Updated T063 blocker: repo and first module exist; still blocked until T052 workflow and T060 `/rmods` install flow are complete.
- Corrected `docs/rmods-registry.md` references from placeholder `rmenu-rmods` to actual `rmods` repo/URL.
- No implementation performed.
- Validation: planning/docs only; no code validation run.

## Previous checkpoint

2026-05-06 05:25 — Calculator rMod published to rmods registry

- Added `modules/calculator.rmod` from `C:\rMenu\modules\calculator.rmod` to `C:\rmods`.
- Regenerated `C:\rmods\registry.json` with `scripts/generate-registry.py`.
- Registry now includes:
  - `id`: `calculator`;
  - `version`: `0.1.0`;
  - `size`: `3021`;
  - `sha256`: `de00cc81828884f32688d344099e8fb2553887d7d30fc652d0b3b1e0f5c7f227`.
- Committed and pushed to `https://github.com/SynrgStudio/rmods.git`:
  - commit `4cc6c04` — `add calculator rmod`.
- Validation:
  - generator run: OK;
  - local registry SHA-256 check: OK;
  - remote raw `registry.json` contains calculator entry: OK via web fetch.

## Previous checkpoint

2026-05-06 03:35 — T051 rMods registry generator added

- Completed T051.
- Cloned/initialized registry repo at `C:\rmods` from `https://github.com/SynrgStudio/rmods.git`.
- Added and pushed commit `0dfdca1` (`feat: add rmod registry generator`) with:
  - `scripts/generate-registry.py`;
  - `README.md`;
  - `modules/.gitkeep`;
  - generated empty `registry.json`.
- Generator behavior:
  - scans `modules/*.rmod`;
  - validates `.rmod` v1 magic, required headers, numeric/supported `api_version`, supported module `kind`, non-empty capabilities, `module.js` block, duplicate blocks, optional JSON config;
  - extracts ID/version/description/tags/compat metadata;
  - computes SHA-256 and size;
  - emits deterministic, ID-sorted schema v1 `registry.json` with raw GitHub download URLs.
- Validation:
  - generated registry from two temp sample `.rmod` files: OK;
  - verified generated SHA-256 values against file bytes: OK;
  - invalid `.rmod` missing `module.js` failed with clear error: OK.
- Next executable `/rmods` task: T052 — Add GitHub Action to regenerate `registry.json`.

## Previous checkpoint

2026-05-06 03:15 — T050 rMods registry schema/layout specified

- Completed T050.
- Added `docs/rmods-registry.md` defining:
  - MVP GitHub repo layout: `modules/*.rmod`, generated `registry.json`, `scripts/generate-registry.*`, `.github/workflows/update-registry.yml`;
  - schema v1 top-level fields and module record fields;
  - metadata source rules from `.rmod` headers per `RMOD_SPEC_V1.md`;
  - default registry URL shape and future config override policy;
  - validation policy for the generator and rMenu core;
  - install/cache/state paths under `C:\rMenuData`;
  - deferred work: zip/folder packages, multi-registry, signing, dependencies, uninstall UI.
- Updated `README.md` with planned `/rmods` workflow summary.
- Updated `MODULES_OPERATIONS_GUIDE.md` with planned registry install section.
- Validation: documentation-only task; no code validation run.
- Next executable `/rmods` task: T051 — Add registry generator script for `.rmod` files.

## Previous checkpoint

2026-05-06 02:55 — `/rmods` registry implementation plan added

- Replanned `ACTIVE_QUEUE.md` for a core-owned `/rmods` registry/install workflow.
- Decision captured:
  - `/rmods` belongs in rMenu core, not in a privileged store `.rmod`;
  - registry source is a GitHub repo with `modules/*.rmod` and generated `registry.json`;
  - GitHub Actions generates registry metadata automatically when `.rmod` files are pushed;
  - rMenu core fetches registry, shows multiselect UI, verifies sha256, installs atomically into `C:\rMenuData\modules`, updates state, and reloads modules.
- Added queue tasks T050-T064 covering:
  - registry schema/repo layout;
  - generator script;
  - GitHub Action;
  - rMenu registry types/fetch/cache/local state;
  - `/rmods` UI/multiselect;
  - secure download/install/reload;
  - docs and smoke validation.
- No implementation performed.
- Validation: documentation/planning only; no code validation run.

## Previous checkpoint

2026-05-06 02:45 — RTasks panel focus restore fixed

- Fixed focus restore after closing RTasks panel.
- RTasks `v0.1.8` published:
  - release: `https://github.com/SynrgStudio/rtasks/releases/tag/v0.1.8`;
  - captures the previously focused window before opening panel;
  - restores it when panel closes via toggle, Escape, or close request.
- rMenu daemon also tracks the foreground window around `Ctrl+Space` and restores focus after it sends the closing panel toggle.
- Reinstalled managed RTasks companion via `rmenu.exe --silent --install rtasks` from GitHub latest.
- Validation:
  - RTasks: `cargo fmt`, `cargo test`, `cargo check`, `cargo build --release`: OK;
  - rMenu: `cargo test`, `cargo check`, `cargo build --release --bin rmenu-daemon`: OK;
  - managed latest install: OK.

## Previous checkpoint

2026-05-06 02:25 — RTasks panel toggle and hidden flash fixed

- Published RTasks `v0.1.7`:
  - release: `https://github.com/SynrgStudio/rtasks/releases/tag/v0.1.7`.
- Fixed panel IPC behavior:
  - repeating the `panel` IPC command now toggles the panel closed instead of forcing it open;
  - therefore rMenu daemon `Ctrl+Space` opens the panel, and pressing `Ctrl+Space` again closes it.
- Removed hidden-mode placeholder UI rendering:
  - no more `Hidden. Hotkeys will open capture or panel.` text flash when closing the task panel.
- Validation:
  - RTasks: `cargo fmt`, `cargo test`, `cargo check`, `cargo build --release`: OK;
  - smoke: direct `rtasks.exe daemon --no-hotkeys`, `rtasks.exe panel`, `rtasks.exe panel` toggles visible panel to off-screen hidden mode;
  - managed companion reinstalled via `rmenu.exe --silent --install rtasks` from GitHub latest.

## Previous checkpoint

2026-05-06 02:05 — Fixed RTasks panel IPC from hidden companion

- Root cause for `Ctrl+Space` not opening RTasks panel:
  - rMenu daemon registered the hotkey successfully and sent `panel` IPC;
  - RTasks `v0.1.5` had changed hidden companion mode to a fully invisible eframe viewport;
  - while fully invisible, eframe did not keep processing update/repaint cycles promptly, so the IPC message stayed pending and the panel did not become visible.
- Fixed in RTasks `v0.1.6`:
  - hidden companion remains an off-screen 1x1 tool window (`with_taskbar(false)`, no tray under `--no-hotkeys`) so the event loop stays alive/responsive;
  - panel positioning now falls back to Win32 primary screen metrics if egui has no monitor size while hidden.
- Published RTasks `v0.1.6`:
  - release: `https://github.com/SynrgStudio/rtasks/releases/tag/v0.1.6`;
  - latest `rtasks.exe` asset updated.
- Reinstalled managed RTasks companion via rMenu latest asset.
- Smoke validation:
  - direct managed `rtasks.exe daemon --no-hotkeys` then `rtasks.exe panel` shows visible `RTasks` window at screen-right coordinates;
  - RTasks tests/check/build OK.

## Previous checkpoint

2026-05-06 01:45 — RTasks embedded-mode UX fixes

- Fixed RTasks mode shortcut updates in rMenu by handling `WM_SYSKEYDOWN`; `Alt+1/2/3` and `Alt+Q/W/E` now update the right-side status/priority accessory immediately.
- Active RTasks accessory state now draws with explicit green active color instead of theme selected text color.
- Published RTasks `v0.1.5`:
  - release: `https://github.com/SynrgStudio/rtasks/releases/tag/v0.1.5`;
  - `daemon --no-hotkeys` now starts without tray icon;
  - hidden daemon viewport starts invisible and skips taskbar.
- Reinstalled managed RTasks companion through rMenu latest asset into `C:\rMenuData\companions\rtasks\rtasks.exe`.
- Validation:
  - RTasks: `cargo fmt`, `cargo test`, `cargo check`, `cargo build --release`: OK;
  - rMenu: `cargo fmt`, `cargo test`, `cargo check`, `cargo build --release`: OK;
  - smoke: `/install rtasks` latest starts one managed companion process from `C:\rMenuData\companions\rtasks\rtasks.exe`.

## Previous checkpoint

2026-05-06 01:20 — RTasks embedded rMenu mode implemented

- Added RTasks `v0.1.4`:
  - release: `https://github.com/SynrgStudio/rtasks/releases/tag/v0.1.4`;
  - stable latest asset: `rtasks.exe`.
- Extended RTasks IPC:
  - `add_task` command with `input`, optional `status`, optional `priority`;
  - `daemon --no-hotkeys` for rMenu-owned hotkey integration.
- rMenu now starts RTasks companion as `rtasks.exe daemon --no-hotkeys`, preventing standalone Alt+Space/Ctrl+Space hotkeys when rMenu owns integration.
- Added embedded RTasks capture mode in rMenu:
  - input prefix `t ` activates task mode;
  - right-side ghost summary shows `TODO` and `prio:MEDIA` by default;
  - `Alt+1/2/3` toggles `TODO/DOING/DONE`;
  - `Alt+Q/W/E` toggles `prio:ALTA/prio:MEDIA/prio:BAJA`;
  - pressing the same shortcut again clears that status/priority;
  - Enter sends task text/status/priority to RTasks via IPC `add_task` and closes rMenu.
- rMenu daemon now owns/registers `Ctrl+Space` and dispatches RTasks panel by IPC, including if RTasks is installed after the daemon starts.
- Existing `tasks`/`todo`/`doing`/`done` queries still show RTasks task records inside rMenu with status/priority/due metadata.
- Validation:
  - RTasks: `cargo fmt`, `cargo test` (9 passed), `cargo check`, `cargo build --release`: OK;
  - RTasks release v0.1.4 created and pushed;
  - rMenu: `cargo fmt`, `cargo test` (98 total across bins), `cargo check`, `cargo build --release`: OK;
  - smoke: `rmenu.exe --silent --install rtasks` installed latest from GitHub and started `C:\rMenuData\companions\rtasks\rtasks.exe` while local dev exe was hidden.

## Previous checkpoint

2026-05-06 00:45 — RTasks companion first pass implemented

- Cloned/reviewed `https://github.com/SynrgStudio/rtasks` into `C:\rTasks`.
- Added native RTasks IPC to RTasks:
  - pipe: `\\.\pipe\rtasks`;
  - commands: `quick_add`, `panel`, `shutdown`;
  - CLI helpers: `rtasks.exe daemon`, `quick-add`, `panel`, `shutdown`.
- Published RTasks `v0.1.3`:
  - release: `https://github.com/SynrgStudio/rtasks/releases/tag/v0.1.3`;
  - assets: `rtasks-v0.1.3-windows-x64.zip`, stable `rtasks.exe`, `SHA256SUMS.txt`.
- Added rMenu RTasks companion support:
  - install latest from `https://github.com/SynrgStudio/rtasks/releases/latest/download/rtasks.exe`;
  - install path: `C:\rMenuData\companions\rtasks\rtasks.exe`;
  - creates companion `config`, `state`, `logs` folders;
  - `rtasks:` launcher target support for `quick-add` and `panel`;
  - built-in provider aliases: `task`, `todo`, `quick`, `quick-add`, `tasks`, `rtasks`, `panel`;
  - daemon starts/confirms RTasks if installed and stops it on `rmenu-daemon.exe --quit`.
- `/install rtasks` uses the same staged UI flow as RSnip.
- Validation:
  - RTasks: `cargo fmt`, `cargo test` (9 passed), `cargo check`, `cargo build --release`: OK;
  - RTasks IPC smoke: daemon -> panel -> shutdown, process exits: OK;
  - rMenu: `cargo fmt`, `cargo test` (98 total across bins), `cargo check`, `cargo build --release`: OK;
  - rMenu smoke: temporarily hid local `C:\rTasks\target\release\rtasks.exe`; `rmenu.exe --silent --install rtasks` installed and started `C:\rMenuData\companions\rtasks\rtasks.exe` from GitHub latest.

## Previous checkpoint

2026-05-06 00:05 — RSnip GitHub release install completed

- Published RSnip `v0.1.2` to GitHub:
  - commit: `856757103c8702fbfc9ecb8983c257e595170cfb`;
  - release: `https://github.com/SynrgStudio/rSnip/releases/tag/v0.1.2`.
- RSnip release assets now include:
  - `rsnip-v0.1.2-windows-x64.zip`;
  - stable latest asset `rsnip.exe`;
  - `rsnip-setup-v0.1.2.exe` from Inno Setup;
  - `SHA256SUMS.txt`.
- Added/committed RSnip installer packaging:
  - `installer/build-installer.ps1`;
  - `installer/rsnip.iss`;
  - updated `release-local.ps1` to build/upload installer and stable `rsnip.exe` asset.
- Updated rMenu `/install rsnip` and `--install rsnip` to download latest from:
  - `https://github.com/SynrgStudio/rSnip/releases/latest/download/rsnip.exe`.
- Local/dev copy path remains fallback only if GitHub download fails and `C:\rSnip\target\release\rsnip.exe` exists.
- UI feedback now says `Fetching rSnip from GitHub latest release`.
- Validation:
  - RSnip release script validation: `cargo fmt --check`, `cargo check`, `cargo test` (55 passed), `cargo build --release`: OK;
  - Inno Setup build: OK;
  - rMenu `cargo fmt`: OK;
  - rMenu `cargo check`: OK;
  - rMenu `cargo test`: OK, 94 tests passed;
  - rMenu `cargo build --release`: OK;
  - smoke: temporarily hid local dev `C:\rSnip\target\release\rsnip.exe`; `rmenu.exe --silent --install rsnip` installed and started `C:\rMenuData\companions\rsnip\rsnip.exe` from GitHub latest.

## Previous checkpoint

2026-05-05 23:02 — Deadlock in `/install rsnip` feedback removed

- User confirmed correct binary/branding now says `rMenu`, but `/install rsnip` still froze.
- Root cause found: Enter handler held `APP_STATE` lock and called `UpdateWindow`; `UpdateWindow` synchronously triggered paint, and paint attempted to lock `APP_STATE`, deadlocking the UI thread.
- Removed synchronous `UpdateWindow` from the Enter handler.
- UI now invalidates and returns to the message loop before painting, avoiding recursive paint while the state lock is held.
- Validation:
  - stopped stale rMenu/rMenu daemon/RSnip processes before rebuild;
  - `cargo fmt`: OK;
  - `cargo check`: OK;
  - `cargo test`: OK, 94 tests passed;
  - `cargo build --release`: OK.

## Previous checkpoint

2026-05-05 22:55 — Resource rebuild and UI-thread install messaging fixed

- User screenshot showed Windows still reporting `dmenu clone for Windows` and rMenu still entering `Not responding`.
- Root cause for stale branding: `build.rs` embedded `resource.rc` but did not declare `cargo:rerun-if-changed=resource.rc`, so Cargo reused stale executable resources.
- Added `cargo:rerun-if-changed=resource.rc`.
- Verified release binary metadata after rebuild:
  - `rmenu-daemon.exe FileDescription = rMenu`;
  - `rmenu-daemon.exe ProductName = rMenu`;
  - `rmenu.exe FileDescription = rMenu`;
  - `rmenu.exe ProductName = rMenu`.
- Hardened `/install rsnip` async UI update path:
  - worker no longer calls redraw/timer APIs directly;
  - worker posts custom UI messages back to the window;
  - UI thread performs redraw and close timer setup.
- Validation:
  - stopped stale rMenu/rMenu daemon/RSnip processes before rebuild;
  - `cargo fmt`: OK;
  - `cargo check`: OK;
  - `cargo test`: OK, 94 tests passed;
  - `cargo build --release`: OK;
  - PowerShell VersionInfo check: release exes now report `rMenu`.

## Previous checkpoint

2026-05-05 22:45 — `/install rsnip` no-responding fix and rMenu branding

- User reported `/install rsnip` put rMenu into Windows `Not responding`.
- Root cause: install/copy/start was still running synchronously on the UI thread.
- Moved `/install rsnip` work to a background thread:
  - UI immediately shows `Fetching rSnip from local/dev source`;
  - worker changes feedback to `Installing rSnip`;
  - worker installs/starts RSnip;
  - worker changes feedback to final success/error;
  - UI auto-closes after ~1 second.
- Updated visible window title to `rMenu`.
- Updated Windows VERSIONINFO resource:
  - `FileDescription = rMenu`;
  - `ProductName = rMenu`;
  - removed `dmenu clone for Windows` branding.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 94 tests passed
  - `cargo build --release`: OK

## Previous checkpoint

2026-05-05 22:35 — Install flow, daemon RSnip shutdown, and dynamic UI height updated

- `rmenu-daemon.exe --quit` now stops the discovered/managed RSnip daemon instead of leaving a pre-existing `rsnip.exe` running.
- `/install rsnip` UI flow now shows staged feedback:
  - `Fetching rSnip from local/dev source`;
  - `Installing rSnip`;
  - `rSnip installed as rMenu companion` or an error.
- After successful/failed `/install rsnip`, rMenu stays visible long enough to show the final status and auto-closes after ~1 second.
- rMenu initial UI no longer opens with a prefilled minimum list; empty input starts as input-bar-only.
- List height is dynamic:
  - empty query: input only;
  - query with one match: input + one row;
  - query with N matches: input + N visible rows up to configured max.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 94 tests passed
  - `cargo build --release`: OK
  - smoke: install RSnip, start daemon, `rmenu-daemon.exe --quit` stops daemon and leaves `rsnip_count=0`.

## Previous checkpoint

2026-05-05 22:18 — `/install rsnip` UI feedback fixed

- User manually validated `/install rsnip` creates `C:\rMenuData\companions\rsnip` correctly.
- Found UX issue: rMenu closed immediately after Enter, so success feedback was invisible.
- Changed slash-command dispatch to report whether a command was handled.
- UI now keeps rMenu open after handled slash commands and redraws the active accessory feedback instead of immediately closing.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 94 tests passed
  - `cargo build --release`: OK

## Previous checkpoint

2026-05-05 22:08 — Default data root changed to `C:\rMenuData`

- Changed default Windows data root from `%APPDATA%\rmenu` fallback model to `C:\rMenuData`.
- `rmenu.exe --install rsnip` now installs to `C:\rMenuData\companions\rsnip\rsnip.exe` when no `--data-dir` / `RMENU_DATA_DIR` override is supplied.
- Updated examples/tests/docs from `D:\rMenuData` to `C:\rMenuData`.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 94 tests passed
  - `cargo build --release`: OK
  - smoke: `rmenu.exe --silent --install rsnip` created `C:\rMenuData\companions\rsnip\rsnip.exe`, `config`, `state`, `logs`, and started the installed RSnip process.

## Previous checkpoint

2026-05-05 21:45 — T045 completed: local/dev `/install rsnip` MVP

- Added rMenu install command path for RSnip MVP:
  - CLI: `rmenu.exe --data-dir <path> --install rsnip`;
  - runtime slash command path: `/install rsnip`.
- MVP copies from local/dev source:
  - `C:\rSnip\target\release\rsnip.exe`
- Destination:
  - `<data_dir>\companions\rsnip\rsnip.exe`
- Installer creates companion subdirectories:
  - `config`;
  - `state`;
  - `logs`.
- After copy, install confirms/starts the RSnip daemon from the installed companion path.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 94 tests passed
  - `cargo build --release`: OK
  - smoke: `rmenu.exe --data-dir <temp> --install rsnip` installs to `<temp>\companions\rsnip\rsnip.exe`, creates config/state/logs, and starts the installed RSnip process.

## Previous checkpoint

2026-05-05 21:35 — T044 completed: RSnip discovery prefers rMenu-managed install

- Updated RSnip companion discovery to prefer:
  - `<data_dir>\companions\rsnip\rsnip.exe`;
  - `RMENU_RSNIP_PATH`;
  - dev path `C:\rSnip\target\release\rsnip.exe`;
  - unambiguous PATH fallback.
- Added `companion_rsnip_path_from_data_dir` helper and tests for the data-root RSnip layout.
- Propagated `--data-dir` into `RMENU_DATA_DIR` for the current rMenu/rMenu-daemon process so companion discovery and menu actions see the selected data root.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 94 tests passed
  - `cargo build --release`: OK
  - smoke: copied `rsnip.exe` to temp `<data_dir>\companions\rsnip\rsnip.exe`; daemon started that managed copy and `--quit` stopped rMenu.

## Previous checkpoint

2026-05-05 21:28 — T043 completed: data-dir resolution implemented

- Added `--data-dir <PATH>` to `rmenu.exe` and `rmenu-daemon.exe` parsing/help.
- Added `RMENU_DATA_DIR` support.
- Added `RmenuDataDirs` helpers deriving:
  - `modules_dir = <data_dir>\modules`;
  - `companions_dir = <data_dir>\companions`;
  - `config_dir = <data_dir>\config`;
  - `state_dir = <data_dir>\state`.
- Preserved explicit `--modules-dir` / `RMENU_MODULES_DIR` as higher-priority module-dir overrides.
- Startup command persistence includes `--data-dir` when provided.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 93 tests passed
  - `cargo build --release`: OK after stopping running rMenu binaries that locked release exes
  - smoke: `RMENU_DATA_DIR=<temp>` makes `rmenu --modules-debug` use `<temp>\modules`
  - smoke: `rmenu-daemon.exe --data-dir <temp>` starts and `--quit` stops it.

## Previous checkpoint

2026-05-05 21:16 — T042 completed: persistent data root specified

- Added DEC-008 to `DECISIONS.md`.
- Official target layout:
  - `<data_dir>\modules`;
  - `<data_dir>\companions\rsnip\rsnip.exe`;
  - `<data_dir>\companions\rsnip\config|state|logs` for future portable RSnip support;
  - `<data_dir>\config`;
  - `<data_dir>\state`.
- Documented that `modules_dir` derives from `<data_dir>\modules` by default once implemented, while `--modules-dir` and `RMENU_MODULES_DIR` remain explicit module-only overrides.
- Updated README with the persistent data-root model.
- Updated handoff with installer reuse expectations and companion layout.
- Validation: documentation task; no code validation required.

## Previous checkpoint

2026-05-05 21:05 — persistent data-root and `/install rsnip` wave planned

- Replanned `ACTIVE_QUEUE.md` for rMenu-managed companion installs.
- Accepted target layout:
  - `<data_dir>\modules`;
  - `<data_dir>\companions\rsnip\rsnip.exe`;
  - `<data_dir>\config`;
  - `<data_dir>\state`.
- Added tasks:
  - T042 specify persistent data root and companion layout;
  - T043 implement data-dir resolution;
  - T044 prefer rMenu-managed RSnip companion path in discovery;
  - T045 implement local/dev `/install rsnip` MVP;
  - T046 add GitHub release install path;
  - T047 add/coordinate RSnip portable config support if needed;
  - T048 specify installer UX for reusable data folder;
  - T049 update docs/manual validation for install UX.
- Current priority is `/install rsnip` into rMenu's persistent data root, then native `snip`/`record`/`ocr` use that installed companion.

## Previous checkpoint

2026-05-05 20:55 — simple daemon lifecycle and RSnip alias override fixed

- Simplified the expected operator path to:
  - `rmenu-daemon.exe --quit`
  - `rmenu-daemon.exe`
- Verified no extra cleanup commands are required for normal rMenu-owned RSnip lifecycle.
- Fixed RSnip exact aliases so stale history/cache cannot outrank the native companion provider:
  - `snip`;
  - `record` / `rec` / `screen`;
  - `ocr` / `text`.
- Added F1-F12 hotkey parsing support for daemon smoke-test/alternate hotkeys.
- Validation:
  - `cargo fmt`: OK
  - `cargo test`: OK, 88 tests passed
  - `cargo check`: OK
  - `cargo build --release`: OK
  - smoke: `rmenu-daemon.exe` starts and `rmenu-daemon.exe --quit` stops it; RSnip is seen during daemon lifetime and no rMenu/RSnip processes remain after quit.
- Remaining manual validation: run `rmenu-daemon.exe`, open rMenu, execute `snip`, and confirm the selected item is the native `RSnip screenshot region` path with no PowerShell flash.

## Previous checkpoint

2026-05-05 20:45 — T040 partial: RSnip lifecycle ownership coordinated

- Implemented integrated lifecycle ownership in `rmenu-daemon`:
  - if RSnip was already running, rMenu leaves it running on `--quit`;
  - if rMenu started RSnip, rMenu sends RSnip shutdown on `--quit`;
  - RSnip remains owner of `Ctrl+Shift+S/R/E` hotkeys in initial integrated mode.
- Updated daemon boundary and handoff docs with lifecycle/hotkey ownership rules.
- Validation:
  - `cargo fmt`: OK
  - `cargo test`: OK, 87 tests passed
  - `cargo check`: OK
  - `cargo build --release`: OK
  - smoke: rMenu-owned RSnip stops on `rmenu-daemon --quit`; pre-existing RSnip remains running after `rmenu-daemon --quit`.
- Remaining for T040: manual validation that `Ctrl+Shift+S/R/E` remain fast and no duplicate hotkey registration appears.

## Previous checkpoint

2026-05-05 20:35 — T039 partial: native RSnip menu provider implemented

- Added native built-in `builtin.rsnip-companion` provider.
- Added `rsnip:` target handling in `launch_target`, dispatching through direct RSnip companion IPC instead of PowerShell or visible console wrappers.
- Exposed native menu items for `snip`, `record`/`rec`/`screen`, and `ocr`/`text` when RSnip is discovered.
- Added explicit missing-RSnip menu feedback if a matching query is typed without RSnip available.
- Disabled transitional `snip.rmod`, `screen-record.rmod`, and `ocr.rmod` wrappers to avoid duplicate commands.
- Validation:
  - `cargo fmt`: OK
  - `cargo test`: OK, 87 tests passed
  - `cargo check`: OK
  - `cargo build --release`: OK
  - `rmenu --modules-dir .\modules --modules-debug`: OK, `builtin.rsnip-companion` loaded and transitional wrappers disabled.
- Remaining for T039: manual interactive validation of rMenu `snip`, `record`, and `ocr` paths; must confirm no console flash and expected RSnip behavior.

## Previous checkpoint

2026-05-05 20:18 — T038 completed: RSnip discovery and direct IPC client

- Added `src/rsnip_companion.rs` with:
  - deterministic RSnip discovery (`RMENU_RSNIP_PATH`, dev path, unambiguous PATH fallback);
  - direct JSON named-pipe protocol client for `\\.\pipe\rsnip`;
  - hidden daemon start/confirm path;
  - structured errors for missing, timeout, protocol, and IO failures;
  - tests for request/response JSON compatibility with RSnip.
- Wired `rmenu-daemon` startup through the companion client instead of the hardcoded RSnip launcher.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 86 tests passed
  - `cargo build --release`: OK
  - manual smoke: `rmenu-daemon` starts RSnip daemon and `--quit` stops rMenu; explicit `rsnip stop` leaves no RSnip process.

## Previous checkpoint

2026-05-05 19:58 — T037 completed: RSnip companion contract specified

- Documented DEC-007 in `DECISIONS.md`:
  - RSnip is an optional native companion capability;
  - rMenu discovers and coordinates RSnip when installed;
  - menu actions should use direct `\\.\pipe\rsnip` IPC, not PowerShell/CLI-wrapper launches;
  - standalone RSnip remains valid when rMenu is absent.
- Updated `docs/ahk-migration/DAEMON_BOUNDARY.md` with RSnip integrated/standalone boundaries, discovery order, lifecycle rules, and target IPC path.
- Updated `docs/ahk-migration/HANDOFF.md` with native RSnip companion direction for the next implementation tasks.
- Validation: documentation task; no code validation required.

## Previous checkpoint

2026-05-05 19:45 — native RSnip companion integration planned

- Replanned `ACTIVE_QUEUE.md` for the intended rMenu + RSnip architecture.
- RSnip is treated as an optional native companion app:
  - standalone RSnip keeps its own daemon/hotkeys when rMenu is absent;
  - rMenu daemon discovers and coordinates RSnip when both are installed;
  - rMenu menu actions should call RSnip through direct IPC, not PowerShell or `rsnip.exe snip` process-launch wrappers.
- Added tasks:
  - T037 specify companion contract;
  - T038 implement RSnip discovery/direct IPC client;
  - T039 add native menu provider/actions for snip/record/OCR;
  - T040 coordinate hotkey ownership/lifecycle;
  - T041 update docs/manual validation.
- No implementation performed in this planning step.

## Previous checkpoint

2026-05-05 12:05 — daemon MVP implemented

- Added `rmenu-daemon.exe` binary.
- Default global hotkey: `Ctrl+Shift+Space`.
- Supports `--hotkey`, `--rmenu`, `--modules-dir`, `--install-startup`, `--uninstall-startup`, and `--quit`.
- Daemon is resident, single-instance guarded, and hidden-window based.
- Corrected daemon architecture to resident-prewarmed mode: config, launcher index, modules, and external module hosts are loaded once in the daemon process; `Alt+Space` shows the UI without spawning a fresh `rmenu.exe` and 14 module hosts each time.
- `--rmenu` remains accepted for startup command compatibility but is not used for per-hotkey spawning in resident-prewarmed mode.
- Startup install writes HKCU Run value using the current daemon/rmenu/modules paths.
- Logs to `%APPDATA%\\rmenu\\rmenu-daemon.log`.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 82 tests passed
  - `cargo run --bin rmenu-daemon -- --quit`: OK
- Manual hotkey/startup validation not run.

## Previous checkpoint

2026-05-05 11:37 — T022 completed

- Added `docs/ahk-migration/HANDOFF.md`.
- Summarized completed core primitives, modules, docs, validation, blocked manual validation, and next-wave backlog.
- Validation not run: documentation-only task.

## Previous checkpoint

2026-05-05 11:34 — T021 completed

- Ran final validation sweep for the migration wave.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK, 77 tests passed
  - `cargo run --bin rmenu -- --modules-debug`: OK, 14 external descriptors loaded
- T020 remains blocked on manual interactive Windows launcher/UAC validation.

## Previous checkpoint

2026-05-05 11:27 — T019 completed

- Added `docs/ahk-migration/COMMAND_INVENTORY.md` mapping AHK intents to implemented modules and future replacements.
- Added `docs/ahk-migration/MANUAL_VALIDATION.md` with safe launch, UAC, missing-helper, and deferred validation checklists.
- Validation not run: documentation-only task.

## Previous checkpoint

2026-05-05 11:22 — T033 completed

- Added `modules/color-picker.rmod` for `color`, `cp`, and `picker`.
- Module launches a configured external helper if present and reports missing helper via `ctx.toast` and warning item.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 11:19 — T032 completed

- Added `modules/ocr.rmod` for `ocr` and `text`.
- Module sends the SnipTool `ocr` command flag.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 11:16 — T031 completed

- Added `modules/screen-record.rmod` for `rec`, `record`, and `screen`.
- Module sends the SnipTool `record` command flag.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 11:13 — T030 completed

- Added `modules/snip.rmod` for `snip` and `snipd`.
- `snip` sends the SnipTool command flag; `snipd` starts the helper daemon if present.
- Missing helper path reports via `ctx.toast` and warning item.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 11:09 — T029 completed

- Added `modules/rclone-backup.rmod` for `backup`, `rclone`, and `bk`.
- Module uses generic config defaults and launches rclone in Windows Terminal.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 11:06 — T028 completed

- Added `modules/pi-launcher.rmod` for `pi` and `api`.
- `api` uses the new `runas:` target prefix.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK
- Manual UAC validation not run.

## Previous checkpoint

2026-05-05 11:03 — T027 completed

- Added `modules/terminal.rmod` for `ter`, `terminal`, `ater`, and `aterminal`.
- Admin terminal aliases use the new `runas:` target prefix.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK
- Manual UAC validation not run.

## Previous checkpoint

2026-05-05 11:00 — T026 completed

- Added `modules/yandex-open.rmod` for `ya`.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 10:58 — T025 completed

- Added `modules/chatgpt-open.rmod` for `chatgpt` and `gpt`.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 10:55 — T024 completed

- Added `modules/url-open.rmod` for `w <url>`.
- Module normalizes URLs by adding `https://` when no scheme is present.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 10:52 — T023 completed

- Added `modules/web-query.rmod` for `g <query>` and `y <query>`.
- Module uses explicit scoped intent with `ctx.replaceItems(...)` and input accessory feedback.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 10:48 — T018 completed

- Updated `MODULES_AUTHORING_GUIDE.md` with `runas:` target guidance and helper-backed module pattern.
- Updated `CTX_ACTIONS_SPEC_V1.md` to clarify rmenu-style toast rendering.
- Existing README and operations docs already cover modules-dir from T002/T006.
- Validation not run: documentation-only task.

## Previous checkpoint

2026-05-05 10:43 — T014 completed

- Added `docs/ahk-migration/DAEMON_BOUNDARY.md`.
- Documented future `rmenu-daemon.exe` responsibilities and out-of-core AHK resident automations.
- Documented phased AHK retirement and rollback path.
- Validation not run: documentation-only task.

## Previous checkpoint

2026-05-05 10:40 — T011 completed

- Added `docs/ahk-migration/SHORTCUTS.md` describing how AHK `Config.AppBindings` maps to existing `shortcuts.rmod` and `shortcuts.user.json`.
- Confirmed no new shortcuts module is needed for this wave.
- Validation:
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 10:36 — T036 completed

- Added external IPC `Toast` action.
- Wired external JS `ctx.toast(message)` to module runtime feedback.
- Implemented toast feedback as a high-priority rmenu-style input accessory for this wave.
- Added unit test proving toast feedback does not require `input-accessory` capability.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK
  - `cargo run --bin rmenu -- --modules-debug`: OK
- Manual visual toast validation remains covered by T020.

## Previous checkpoint

2026-05-05 10:29 — T035 completed

- Implemented reserved `runas:<target-and-args>` launch prefix.
- `launch_target` now uses `ShellExecuteW` verb `runas` for elevated targets and preserves normal `open` behavior for existing targets.
- Added launcher unit test for `runas:` target parsing.
- Documented `runas:` target convention in `MODULES_API_SPEC_V1.md`.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK
- Manual UAC validation not run; requires explicit user approval.

## Previous checkpoint

2026-05-05 10:23 — T034 completed

- Added minimal primitive details to `docs/ahk-migration/DECISION.md`.
- Chosen admin-launch representation for this wave: reserved target prefix `runas:<target-and-args>`.
- Specified `ctx.toast(message)` as rmenu-style visual feedback, with input-accessory fallback allowed for this wave.
- Marked T034 done in `ACTIVE_QUEUE.md`.
- Validation not run: documentation-only task.

## Previous checkpoint

2026-05-05 10:18 — T006 completed

- Added `--modules-dir <PATH>` CLI option.
- Added `RMENU_MODULES_DIR`, `%APPDATA%\rmenu\modules`, executable-dir `modules`, and cwd `modules` resolution fallback.
- Wired resolved module path into module loading.
- Added resolved `modules_dir` to `--modules-debug` output.
- Added settings unit test for module directory candidate selection.
- Validation:
  - `cargo fmt`: OK
  - `cargo check`: OK
  - `cargo test`: OK
  - `cargo run --bin rmenu -- --modules-debug`: OK

## Previous checkpoint

2026-05-05 10:10 — T002 completed

- Documented module directory resolution order in `MODULES_OPERATIONS_GUIDE.md`.
- Added README note for `--modules-dir` and resolution order.
- Marked T002 done in `ACTIVE_QUEUE.md`.
- Validation not run: documentation-only task.

## Previous checkpoint

2026-05-05 10:06 — T001 completed

- Added `docs/ahk-migration/DECISION.md` with the accepted migration boundary for this wave.
- Added `DEC-006` to `DECISIONS.md` summarizing minimal approved core primitives:
  - module directory resolution;
  - `runas`;
  - rmenu-style toast feedback.
- Marked T001 done in `ACTIVE_QUEUE.md`.
- Validation not run: documentation-only task.

## Previous checkpoint

2026-05-05 10:00 — `/plan-cont` replan

- Replanned `ACTIVE_QUEUE.md` based on latest core-scope decisions:
  - core local module config is not required for this wave;
  - helper-backed modules should be autocontained and use relative/default config inside each `.rmod`;
  - `runas` support is still required and valuable;
  - rmenu-style toast/`ctx.toast` is approved for feedback;
  - full `ItemAction`, `onSubmit`, `copy`, keep-open, and broad bridge parity are deferred.
- Cancelled additional superseded tasks:
  - T003, T004, T005, T007, T008, T009, T015, T016, T017.
- Added minimal core tasks:
  - T034 — specify minimal runas and toast primitives;
  - T035 — implement minimal runas support;
  - T036 — implement rmenu-style toast and ctx.toast support.
- Validation not run: planning/documentation-only changes.

## Active continuity session

```text
CONT-2026-05-04-1945-ahk-suite-rmenu-migration
```

## Active goal

Migrar la suite AHK hacia `rmenu` de forma nativa:

1. Minimal core for current wave:
   - module dir resolution for dev/debug and installed mode;
   - minimal `runas` support for admin launch commands;
   - rmenu-style toast/`ctx.toast` feedback.
2. Active modules replacing Command Center pieces:
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
   - existing `shortcuts.rmod` review/guidance only.
3. Future modules/backlog:
   - Anytype split modules;
   - TweetFlow split modules;
   - daemon-backed modules for global hooks.
4. Future daemon boundary:
   - WindowManager, global hotkeys, Thorium gestures, taskbar volume, TextExpander, AlwaysOnTop, and Snip hotkeys belong in future `rmenu-daemon.exe`, not in core `.rmod` modules.

## Planning decisions

- Current modules can mostly use existing `target` launch behavior.
- Generic open behavior stays in core; only explicit `w <url>` gets `url-open.rmod`.
- `.rmod` modules may use internal `config.json` and relative helper paths; core local config loading is deferred.
- `runas` is the only admin/elevation primitive needed now.
- Toast should use rmenu visual language, not native Windows notification style.
- Anytype and TweetFlow remain future-only for this wave.
- WindowManager/global hotkeys remain daemon work, not core/module work.

## Validation status

Not run for `/plan-cont`; only queue/state Markdown was updated.

Expected validation after future code changes:

```bash
cargo fmt
cargo check
cargo test
cargo run --bin rmenu -- --modules-debug
```

## Known blockers

- T020 is blocked by manual interactive Windows launcher validation after implementation.
- Manual UAC/runas validation is blocked unless user explicitly approves testing an elevated launch.
- The core is documented as frozen v1; T001/T034 must keep changes general and additive.
- Anytype module work is deferred; do not implement it in this wave unless user re-scopes.
- Full AHK WindowManager parity requires future daemon/helper work, not immediate core/module work.

## Next recommended step

Run:

```text
/start-cont
```

This should begin with:

```text
T001 — Record minimal migration core-change decision
```

## Previous continuity context preserved

Before this session, repo continuity files were idle after archived session:

```text
last_archived_session: CONT-2026-04-25-0858-wave0-packaging-release
archive_path: docs/continuity/archive/CONT-2026-04-25-0858-wave0-packaging-release/
```

Archived summary from prior session:

- Wave 0 packaging/release completed locally.
- Queue summary: 8 done, 0 blocked, 0 pending.
- Local validation passed: `cargo fmt`, `cargo test`, `cargo check`, `cargo build --release`.
- Release publishing remained external/manual.
- A later local release attempt for `0.2.2` failed before commit/tag/release due to disk space exhaustion.

## Checkpoints

### 2026-05-04 19:45 — init-cont

Files changed:

- `AUTONOMOUS_EXECUTION.md`
- `ACTIVE_QUEUE.md`
- `STATE.md`

Validation:

- Not run; documentation/continuity initialization only.

Next:

- `/plan-cont` to produce official executable plan.

### 2026-05-04 20:04 — plan-cont

Files changed:

- `ACTIVE_QUEUE.md`
- `STATE.md`

Validation:

- Not run; planning/documentation-only changes.

Queue summary:

- 21 pending.
- 1 blocked.
- 0 done.

Next:

- `/start-cont` beginning with T001.

### 2026-05-05 09:49 — plan-cont replan

Files changed:

- `ACTIVE_QUEUE.md`
- `STATE.md`

Validation:

- Not run; planning/documentation-only changes.

Queue summary:

- 29 pending.
- 1 blocked.
- 3 cancelled.
- 0 done.

Next:

- `/start-cont` beginning with T001.

### 2026-05-05 10:00 — plan-cont replan minimal core

Files changed:

- `ACTIVE_QUEUE.md`
- `STATE.md`

Validation:

- Not run; planning/documentation-only changes.

Queue summary:

- 23 pending.
- 1 blocked.
- 12 cancelled.
- 0 done.

Next:

- `/start-cont` beginning with T001.

## 2026-05-06 20:35 — Companion/rMods finalization pass

- Limited RSnip public aliases to `snip`, `rec`, and `ocr`.
- Limited RTasks public panel alias to `tasks`; embedded `t ` task capture remains the task input path.
- Added external module `ctx.moduleStateDir()` and documented stable state storage under `<data_dir>\state\modules\<module-name>`.
- Moved the live `shortcuts` rpack user-data target to `ctx.moduleStateDir()` in the rMods registry repo and published `shortcuts 0.3.3`.
- Removed generated/local-only cleanup artifacts from the rMenu worktree: `dist/`, local installed `modules/*` copies, old transitional snip/screen/OCR wrappers, `installer/`, and `codebase-report.md`.
- Added comprehensive companion/rMods workflow docs.
- Validation: `cargo fmt --all`, `cargo check`, `cargo test` all passed.

### 2026-05-06 21:00 — plan-cont daemon latency replan

Files changed:

- `ACTIVE_QUEUE.md`
- `STATE.md`

Validation:

- Not run; planning/documentation-only changes.

Context:

- User measured direct `rmenu.exe --metrics`: window visible/input ready around 23ms, search p95 under 1ms, dataset 1091 items.
- `--modules-debug` showed 4 external descriptors/hosts loaded and healthy.
- Conclusion: perceived 1-3s latency is likely in daemon hotkey/reopen lifecycle, not fuzzy search or direct launcher startup.

Plan added:

- T066 instrumentation for daemon hotkey-to-visible latency.
- T067 avoid repeated module `onLoad` on every daemon reopen.
- T068 keep external module hosts hot across daemon UI sessions.
- T069 cache RSnip/RTasks companion discovery in hot paths.
- T070 defer noncritical provider/decorator work until after input is visible.
- T071 resident show/hide daemon UI if measurement proves it is needed.
- T072 blocked manual release-mode performance acceptance.

Next:

- `/start-cont` beginning with T066.

### 2026-05-06 21:05 — T066 claimed

Task: T066 — Instrument daemon hotkey-to-visible latency

Scope:

- Add release-safe timing instrumentation for daemon hotkey open/reopen path.
- Measure pre-window setup, module on-load, matching update, window visible/first paint, close/runtime handoff.

Validation planned:

- `cargo fmt --all`
- `cargo check`
- `cargo test`
- targeted daemon log smoke if feasible without manual interaction.

### 2026-05-06 21:25 — T066-T068 daemon timing checkpoint

Tasks:

- T066 — Instrument daemon hotkey-to-visible latency: done.
- T067 — Avoid repeated module `onLoad` work on every daemon reopen: done by evidence/no code change.
- T068 — Keep external module hosts hot across daemon UI sessions: done by evidence/no code change.

Files changed:

- `src/ui_win32.rs`
- `src/daemon_main.rs`
- `ACTIVE_QUEUE.md`
- `STATE.md`

Validation:

- `cargo fmt --all`: OK
- `cargo check`: OK
- `cargo test`: OK
- `cargo build --release`: OK
- Release daemon smoke: OK; timing logs emitted.

Evidence:

- Controlled release daemon smoke with temporary `ctrl+alt+f12` hotkey produced daemon timing lines.
- Window visible time was about 63-66ms; first paint/input ready about 65-70ms.
- `pre_window_setup_ms`, `module_on_load_ms`, and `initial_matching_update_ms` were around 0-1ms in the smoke.
- Host-count smoke showed `rmenu-module-host.exe` count stabilized at 4 and stayed stable across repeated opens.

Next:

- Continue with T069 if hot-path companion discovery still needs cache work.

### 2026-05-06 21:30 — T069 claimed

Task: T069 — Cache companion discovery in the daemon/runtime hot path

Scope:

- Add short-lived cached discovery for RSnip/RTasks companion paths.
- Invalidate/update cache after installs.
- Preserve normal explicit install/update behavior.

Validation planned:

- `cargo fmt --all`
- `cargo check`
- `cargo test`

### 2026-05-06 21:45 — T069 done, T070 claimed

Tasks:

- T069 — Cache companion discovery in daemon/runtime hot path: done.
- T070 — Defer noncritical module/provider work until after input is visible: claimed.

Files changed for T069:

- `src/rsnip_companion.rs`
- `src/rtasks_companion.rs`

Validation:

- `cargo fmt --all`: OK
- `cargo check`: OK
- `cargo test`: OK
- `cargo build --release`: OK

Next:

- Implement T070 by skipping initial provider/query work for empty daemon UI input before the first window is visible.

### 2026-05-06 22:00 — T070 done, T071 cancelled

Tasks:

- T070 — Defer noncritical module/provider work until after input is visible: done.
- T071 — Convert daemon embedded UI to resident show/hide window if needed: cancelled for now.

Files changed:

- `src/ui_win32.rs`
- `src/daemon_main.rs`
- `src/rsnip_companion.rs`
- `src/rtasks_companion.rs`
- `ACTIVE_QUEUE.md`
- `STATE.md`

Validation:

- `cargo fmt --all`: OK
- `cargo check`: OK
- `cargo test`: OK
- `cargo build --release`: OK
- Release daemon timing smoke: OK

Evidence:

- After T070, controlled release smoke showed `window_visible_ms` about 62-64ms and `first_paint_ms`/`input_ready_ms` about 63-69ms.
- Warm total timing in smoke remains ~1.5s only because the script intentionally waits before closing the UI; visible/input-ready timings are under target.
- Resident show/hide is not justified yet by measured data.

Notes:

- There is still an intentional 50ms sleep in `WM_CREATE` for non-measure mode, which accounts for most of `create_window_ms`. Removing it may improve latency further, but it appears intentional focus-stabilization code and should be changed only with explicit approval or a focused task.

Next:

- T072 remains blocked on user manual release-mode validation with the real hotkey.

### 2026-05-06 22:05 — RSnip partial tasks closed from user validation

Tasks:

- T039 — Add native rMenu provider/actions for RSnip menu commands: done.
- T040 — Coordinate RSnip hotkey ownership with rMenu daemon: done.
- T041 — Update docs and manual validation for native RSnip integration: done.

Evidence:

- User confirmed RSnip command path has no PowerShell/CMD flash.
- User confirmed RSnip hotkeys work.
- User requested final public aliases be limited to `snip`, `rec`, and `ocr`; implementation and docs now match.

Validation:

- No new code validation for this checkpoint; it records prior implementation plus user manual validation.

### 2026-05-06 22:15 — Performance UX accepted by user

Task:

- T072 — Release-mode performance validation and UX acceptance: done.

Evidence:

- User confirmed first daemon startup can still take time, which is acceptable as system-startup cost.
- User confirmed subsequent hotkey opens happen effectively at the same time as pressing the shortcut.

Validation already completed before acceptance:

- `cargo fmt --all`: OK
- `cargo check`: OK
- `cargo test`: OK
- `cargo build --release`: OK
- Release daemon timing smoke: OK
- Host-count smoke: OK

Next:

- Commit and push the performance instrumentation/optimization changes.
