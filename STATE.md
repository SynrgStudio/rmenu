# STATE — rmenu

Date: 2026-04-24  
Branch: `main`

---

## Executive summary

`rmenu` has a working launcher core and two real modules validated:

- `calculator.rmod`
- `local-scripts.rmod`

The module system supports external `.rmod` modules and directory modules with `module.toml`, external host, IPC, capabilities, providers, commands, input accessories, key hooks, decorations, quick select, dedupe, diagnostics, and runtime actions.

Validated functionality:

- stable launcher core;
- real `calculator` module working;
- real `local-scripts` module working;
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
44 tests passed
0 failed
```

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

---

## Functional core changes present

- external hosts can return actions to the core;
- `ctx.setInputAccessory(...)` works from external modules;
- `ctx.clearInputAccessory()` works from external modules;
- `ctx.replaceItems(...)` works from external modules;
- UI cycle respects `replaceItems([])` and `replaceItems(items)`;
- `input_accessory_text()` renders only `accessory.text`;
- external hooks without capability are not invoked and do not generate `permission_denied` spam;
- external `ReplaceItems` actions update the real visible `AppState`.

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

#### 2.3 Hotkeys/quick actions

Status: pending.

Pending:

- create module that uses quick-select keys or key hooks;
- declare `keys` capability;
- confirm denial without capability;
- confirm duplicate quick-select behavior;
- confirm warnings/debug output;
- confirm no core change is required.

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

---

## Later phases pending

### Phase 3 — Functional hardening

Pending validation/hardening:

- real isolation per external module;
- timeout per request;
- hung host kill/degrade;
- auto-restart with backoff;
- auto-disable after thresholds;
- reload counter reset;
- module errors do not break other modules;
- sufficient `--modules-debug` output.

### Phase 4 — Tests and verification

Existing tests: 44.

Covered by current tests includes: `.rmod` parser basics, duplicate/missing blocks, mixed loader, dedupe, command collisions, quick-select duplicate behavior, input accessory kind/priority mapping, host health disable after timeouts, runtime module commands, and external `ReplaceItems` action applying to visible items.

Pending tests:

- valid/invalid `module.toml`;
- allow/deny capabilities;
- provider timeout;
- provider item cap;
- hot reload;
- host restart/backoff;
- broader auto-disable scenarios;
- payload limits;
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

- need at least three real modules validated;
- need hardening;
- need specific tests;
- need minimum performance validation.

---

## Recommended next step

Move to Phase 2.3: hotkeys/quick actions.

Suggested module goal:

- validate `keys` capability routing;
- validate denial without `keys`;
- validate quick-select/key-hook UX;
- confirm duplicate quick-select warnings remain understandable;
- avoid adding new core primitives.

After that, proceed with Phase 3 hardening and targeted tests.

---

## Useful commands

Non-visual validation:

```bash
cargo check
cargo test
```

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
