# STATE — rmenu

Date: 2026-04-24  
Branch: `main`

---

## Executive summary

`rmenu` has a working launcher core and three real modules validated:

- `calculator.rmod`
- `local-scripts.rmod`
- `shortcuts.rmod`

The module system supports external `.rmod` modules and directory modules with `module.toml`, external host, IPC, capabilities, providers, commands, input accessories, key hooks, decorations, quick select, dedupe, diagnostics, and runtime actions.

Validated functionality:

- stable launcher core;
- real `calculator` module working;
- real `local-scripts` module working;
- real `shortcuts` module working;
- `InputAccessory` from external modules end-to-end;
- `ctx.replaceItems([])` from external modules to suppress fuzzy results during calculations;
- `ctx.replaceItems(items)` from external modules to implement scoped intent mode;
- input accessory renders without `[success]` prefix;
- no `permission_denied` spam for undeclared hooks.

---

## Latest technical validation

Commands executed for the local-scripts/core action work:

```bash
cargo check
cargo test
cargo run --bin rmenu -- --modules-debug
```

Current test result:

```text
72 tests passed
0 failed
```

Phase 3 hardening passes added targeted tests and diagnostics improvements, then `cargo fmt`, `cargo test`, `cargo check`, and `cargo run --bin rmenu -- --modules-debug` were run successfully.

Latest `/start-cont Completa Phase 3` checkpoint: added `AUTONOMOUS_EXECUTION.md`, expanded host/client hardening tests, validated request timeout kill behavior, host request/response payload handling, hot reload/debounce, reload health/backoff reset, provider/command/input-accessory capability allow/deny, disabled modules, deterministic ordering, module-host invalid-state behavior, external-host error isolation, key capability routing, auto-disable thresholds, and restart backoff. Phase 3 is complete.

After translating root documentation and updating `build.rs`, `cargo check` was run again successfully.

`build.rs` no longer emits `cargo:warning` for successful copies of `config_example.ini` and `README.md`.

---

## Current module status

### 1. Calculator

File:

```text
modules/calculator.rmod
```

Status: working and manually validated.

Expected behavior:

- open `rmenu`;
- type a simple calculation with no prefix:

```text
2+2
10/2
(2+3)*4
```

Result:

- shows the result in the input bar, aligned to the right;
- format: `=<result>`;
- color: green through `InputAccessoryKind::Success`;
- no `[success]` text prefix;
- lower list is cleared while the calculation is valid;
- no unrelated fuzzy results appear during calculation.

Declared capabilities:

```text
input-accessory
```

Important: the core contains no calculator logic. The core only provides generic primitives. Calculation logic lives in the `.rmod`.

### 2. Local scripts

File:

```text
modules/local-scripts.rmod
```

Status: working and manually validated.

Expected behavior:

- `>` lists local scripts;
- `> bu` filters local scripts;
- `> build` shows `build` first with `exact` badge and `build-prod` below;
- `build` without prefix keeps normal global launcher behavior;
- Enter on a script runs its `target` through the normal launcher path.

Demo scripts:

```text
modules/local-scripts/scripts/build.ps1
modules/local-scripts/scripts/build-prod.ps1
modules/local-scripts/scripts/test.bat
modules/local-scripts/scripts/lint.ps1
modules/local-scripts/scripts/open-logs.cmd
```

Key decision: `local-scripts` v2 does not compete as a global provider. It uses explicit intent with the `>` prefix and `ctx.replaceItems(items)` to enter scoped mode. This avoids ranking, boost, and global dedupe changes.

### 3. Shortcuts

File:

```text
modules/shortcuts.rmod
```

Status: working and manually validated, including user-created bindings.

Expected behavior:

- `b` shows Blender as the only shortcut result with `[b]` as the visual cue;
- `1` also shows Blender and keeps `[b]` as the visual cue;
- `bl`, `b foo`, and `1 foo` do not activate the shortcut and fall back to normal launcher search;
- Enter on the shortcut launches:

```text
C:\Program Files\Blender Foundation\Blender 5.0\blender-launcher.exe
```

Default shortcuts:

```text
1 / b -> Blender
2 / t -> Terminal
```

Add-shortcut flow:

1. Search/select a normal launcher item.
2. Press `Ctrl+B`.
3. `shortcuts.rmod` reads `ctx.selectedItem()` from the external ctx snapshot.
4. The core applies external `ctx.setQuery('/shortcuts::bind ')`.
5. Type an alias, for example `/shortcuts::bind bs`, and press Enter.
6. The module writes the binding to `modules/shortcuts.user.json`.
7. Typing the alias launches the saved target.

Key decision: shortcuts v1 are exact search aliases, not global hotkeys. Binding uses the general `keys`, `commands`, ctx snapshot, and `setQuery` action path; no shortcut-specific core behavior is added.

---

## Functional core changes present

- external hosts can return actions to the core;
- `ctx.setInputAccessory(...)` works from external modules;
- `ctx.clearInputAccessory()` works from external modules;
- `ctx.replaceItems(...)` works from external modules;
- UI cycle respects `replaceItems([])` and `replaceItems(items)`;
- `input_accessory_text()` renders only `accessory.text`;
- external hooks without capability are not invoked and do not generate `permission_denied` spam;
- external `ReplaceItems` actions update the real visible `AppState`;
- external ctx snapshots expose query, items, selected item/index, and mode to JS modules;
- external `SetQuery` actions can update the input, enabling command-prefill flows such as shortcut binding;
- restart backoff remains enforced silently to avoid noisy operational stderr during normal launcher use;
- plain text key presses are not dispatched to module key hooks, avoiding input latency while preserving modified keys like `Ctrl+B`;
- hot query snapshots omit item lists, while key/command snapshots include selected item context, preserving fast input and binding UX.

---

## Official documentation and specs

Root specs/guides:

```text
MODULES_ARCHITECTURE.md
MODULES_API_SPEC_V1.md
RMOD_SPEC_V1.md
MANIFEST_SPEC_V1.md
CTX_ACTIONS_SPEC_V1.md
PROVIDER_EXECUTION_POLICY.md
ERROR_ISOLATION_POLICY.md
MODULES_CAPABILITIES_MATRIX.md
MODULES_AUTHORING_GUIDE.md
MODULES_OPERATIONS_GUIDE.md
MODULES_QUICKSTART.md
DECISIONS.md
CORE_CLOSURE_CHECKLIST.md
```

Historical material is preserved in:

```text
docs/historico/
```

---

## Current task list

Primary source:

```text
CORE_CLOSURE_CHECKLIST.md
```

### Phase 1 — Documentation, vocabulary, and architecture boundary

Status: mostly complete.

Completed:

- modular architecture documented;
- v1 specs in root;
- public root docs/specs translated to English for GitHub readiness;
- historical docs moved to `docs/historico/`;
- README updated;
- `Cargo.toml` metadata cleaned;
- `.gitignore` reviewed.

### Phase 2 — Real module validation

#### 2.1 Calculator

Status: complete except optional future copy/use-result UX.

Completed:

- `.rmod` module created;
- simple calculation detection without `=` prefix;
- result via `InputAccessory`;
- lower list cleared with `replaceItems([])`;
- minimal capabilities;
- friction documented.

Optional pending:

- define UX for copying/using result with Enter or an official command.

#### 2.2 Scripts/commands

Status: validated for `local-scripts` v2 except optional namespaced commands.

Completed:

- `.rmod` module created;
- explicit `>` prefix for local intent;
- script listing and filtering;
- exact match first with `exact` badge;
- subtitles/hints/badges for metadata;
- Enter execution manually validated;
- global dedupe/ranking untouched because the module uses `ctx.replaceItems(items)`;
- scoped intent mode documented in `MODULES_AUTHORING_GUIDE.md`, `MODULES_QUICKSTART.md`, `MODULES_OPERATIONS_GUIDE.md`, and `DECISIONS.md`.

Optional pending:

- register namespaced commands, for example `/local-scripts::list` or `/local-scripts::reload`.

#### 2.3 Shortcuts / quick actions

Status: validated with `shortcuts.rmod`.

Completed:

- `.rmod` module created;
- exact alias matching for `key` and `alias`;
- `b` and `1` activate Blender;
- non-exact inputs such as `bl`, `b foo`, and `1 foo` fall back to normal launcher search;
- visual cue badge shows `[b]`;
- Blender launch path with spaces was fixed by preserving quotes in `launch_target` parsing;
- `Ctrl+B` binding flow implemented using `keys`, `commands`, external ctx snapshot, and `SetQuery` action;
- user shortcuts persist to `modules/shortcuts.user.json`;
- plain text input remains immediate with `shortcuts` loaded;
- user-created bindings were manually validated;
- no shortcut-specific core behavior is required.

#### 2.4 Friction review

Status: current known frictions classified and resolved.

Known frictions:

1. External modules needed real actions back into core.
   - Resolved for `setInputAccessory`, `clearInputAccessory`, and `replaceItems`.

2. Input accessory rendered `[success]`.
   - Resolved: render only text.

3. Undeclared hooks generated noisy `permission_denied`.
   - Resolved: external hooks without capability are not invoked.

4. Provider exact intent vs global fuzzy/core ranking.
   - Resolved for `local-scripts` v2 without a new primitive: explicit `>` prefix + `ctx.replaceItems(items)` scoped intent mode.

5. Shortcut keybinding vs key hooks.
   - Resolved for v1 as exact search aliases using `ctx.replaceItems([item])`, with `Ctrl+B` used only to start an explicit bind command flow. Plain text keys are not dispatched to module key hooks and hot query snapshots omit items to preserve input latency.

---

## Later phases pending

### Phase 3 — Functional hardening

Pending validation/hardening:

- Phase 3 checklist is complete.
- Remaining work moves to Phase 4 verification, Phase 5 polish/performance/manual UX checks, and Phase 6 freeze preparation.

Latest `/start-cont Completa Phase 3` completed the external-host fixture coverage needed to close Phase 3: error isolation, key capability routing, timeout/kill behavior, restart backoff, auto-disable thresholds, and worker payload limits are now covered and green.

Completed in latest Phase 3 passes:

- `--modules-debug` now exposes policy, capabilities, health, telemetry, and recent errors;
- provider total-budget exhaustion no longer records an error against an innocent later host that was skipped due to global budget;
- restart telemetry now counts actual restart attempts after backoff, not every suppressed failure;
- input accessory capability allow/deny path has direct test coverage;
- provider item cap is covered by direct test coverage;
- provider and command registration capability allow/deny paths have direct test coverage;
- disabled external modules are discovered but not loaded;
- disabled hosts are not restarted;
- descriptor ordering by priority then name has direct test coverage;
- hot reload detection and debounce behavior have direct test coverage;
- successful reload resets health/backoff counters;
- host request timeout kills an unresponsive child process;
- oversized host requests fail before waiting for a response;
- host response reader ignores invalid JSON and stops on oversized payloads;
- module-host payload env parsing and unloaded request errors have direct test coverage;
- invalid `.rmod` loader errors are covered;
- valid/invalid `module.toml` parser behavior has direct test coverage.

### Phase 4 — Tests and verification

Existing tests: 72 total across `rmenu` and `rmenu-module-host` test binaries.

Covered by current tests includes: `.rmod` parser basics, duplicate/missing blocks, invalid `.rmod` loader error, valid/invalid `module.toml`, quoted executable target parsing, mixed loader, deterministic module ordering, disabled modules not loaded, hot reload/debounce, reload health/backoff reset, external provider error isolation, external key capability routing, external timeout auto-disable, external restart backoff, dedupe, command collisions, provider item cap, quick-select duplicate behavior, input accessory kind/priority mapping, provider/command/input accessory capability allow/deny, host health disable after timeouts, host request timeout with child kill, oversized host request rejection, disabled host no-restart behavior, restart backoff attempt accounting, modules debug report contents, runtime module commands, external `ReplaceItems` action applying to visible items, external `SetQuery` action updating input, key-hook dispatch filtering for plain text input, host response invalid JSON tolerance, host request/response payload limit handling, module-host worker response payload limit handling, module-host payload env parsing, unloaded request errors, and shutdown state cleanup.

Pending tests:

- more external host action paths.

### Phase 5 — Product/core polish

Pending:

- review full `config_example.ini`;
- document `[Modules]` better;
- validate base UX without modules;
- validate performance;
- confirm safe defaults.

### Phase 6 — Freeze declaration

Do not start yet.

Blockers:

- need manual validation of the third real module (`shortcuts`);
- need hardening;
- need specific tests;
- need minimum performance validation.

---

## Recommended next step

Phase 3 is complete. Move next to Phase 4 verification cleanup and Phase 5 product/core polish:

- add remaining external host action path tests where useful;
- run/manual-check launcher mode and stdin/script mode;
- validate `--metrics`, `--debug-ranking <query>`, and performance targets;
- review `config_example.ini` and `[Modules]` docs;
- prepare for Phase 6 freeze only after Phase 4/5 blockers are resolved.

---

## Useful commands

Non-visual validation:

```bash
cargo check
cargo test
```

Latest result: 72 passed, 0 failed.

Release build:

```powershell
cargo build --release
.\target\release\rmenu.exe
```

Module diagnostics:

```powershell
.\target\release\rmenu.exe --modules-debug
```

Note: `rmenu` searches modules in `modules/` relative to the current working directory. Run from:

```powershell
cd D:\rmenu
.\target\release\rmenu.exe
```
