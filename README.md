# rmenu

Lightweight Windows launcher inspired by `dmenu`, built in Rust.

`rmenu` has two modes:

1. **Launcher mode (default)**: no `-e` and no piped `stdin` -> auto-loads apps/commands from History + Start Menu + PATH and launches on Enter.
2. **Script/dmenu mode**: provide items via `-e` or `stdin` -> returns selected item to `stdout`.

<p align="center">
  <img src="https://raw.githubusercontent.com/SynrgStudio/rmenu/main/img.jpg" alt="rmenu screenshot" width="600"/>
</p>

## Why rmenu

- Fast fuzzy search with source-aware ranking.
- Friendly labels for apps (uses executable metadata when available).
- Still searchable by technical executable names (`mspaint`, `powershell`, etc.).
- Native Windows launch path via `ShellExecuteW` with controlled fallback.
- JSON index cache with environment signature (auto-invalidation on PATH/Start Menu changes).
- Modular runtime for external providers, commands, decorators and input accessories.
- Built-in diagnostics for ranking, performance and modules (`--debug-ranking`, `--metrics`, `--modules-debug`, `--reindex`).

---

## Current project status

`rmenu` is a native Windows launcher and modular command surface.

The launcher core is implemented and stable:

- Win32 UI extracted into `src/ui_win32.rs`
- Ranking engine in `src/ranking.rs` + `src/fuzzy.rs`
- Source indexing/cache in `src/sources/mod.rs`
- Launch backend in `src/launcher.rs`
- History persistence + source boosts + blacklist controls
- Unicode UI rendering (`TextOutW`)

The modular core is defined around:

- `.rmod` single-file modules and directory modules with `module.toml`
- external module host process
- IPC boundaries
- capabilities enforcement
- provider budgets/timeouts
- command namespacing
- decorations, quick-select and input accessory primitives
- module diagnostics via `--modules-debug`

Architecture and public contracts live in the root specs, starting with `MODULES_ARCHITECTURE.md`. The formal v1 freeze declaration lives in `CORE_FREEZE_V1.md`.

---

## Installation

### From releases

Download the latest Windows x64 zip from:

- <https://github.com/SynrgStudio/rmenu/releases>

See `INSTALL.md` for zip install, PATH setup, checksum verification, and manual update instructions.

Release docs:

- `scripts/release-local.ps1` — interactive one-command maintainer release script.
- `INSTALL.md` — install/update instructions.
- `CHANGELOG.md` — release notes.
- `RELEASE_CHECKLIST.md` — maintainer release process.
- `docs/release/BINARY_SIGNING.md` — current signing/checksum policy.

### Build from source

```bash
git clone https://github.com/SynrgStudio/rmenu.git
cd rmenu
cargo build --release
```

Binary output:

- `target/release/rmenu.exe`

---

## Usage

### Launcher mode (default)

```powershell
rmenu.exe
```

### Script mode with `stdin`

```powershell
"Option 1`nOption 2`nOption 3" | rmenu.exe -p "Pick one"
```

### Script mode with `-e`

```powershell
rmenu.exe -e "Item A,Item B,Item C" -p "Pick one"
```

---

## CLI options

```text
rmenu - A simple dmenu-like launcher for Windows
Usage: rmenu [OPTIONS]

Input Options:
  -e, --elements <LIST>   List of items (delimiter in config.ini, default: ',').
                            If omitted and stdin is piped, rmenu reads stdin (one per line).
                            If omitted and stdin is not piped, launcher mode is used (default).
  -p, --prompt <TEXT>     Text to display as prompt.

Configuration and Behavior Options:
  -c, --config <PATH>     Path to the configuration file (config.ini).
  -s, --silent            Suppress all error/diagnostic messages (stderr).
  --debug-ranking <QUERY> Print ranking breakdown (fuzzy + source boost) and exit.
  --metrics               Print startup/UI/search/dataset metrics and exit.
  --modules-debug         Print module descriptors/hosts/telemetry and exit.
  --reindex               Force index rebuild (ignore cache for this run).
  -h, --help              Show this help.

Geometry and Layout Options (override config.ini):
  --layout <NAME>         custom, top-fullwidth, bottom-fullwidth, center-dialog,
                          top-left, top-right, bottom-left, bottom-right
  --x-pos <POS>           E.g. 100 or r0.5
  --y-pos <POS>           E.g. 0 or r0.3
  --width-percent <FLOAT> 0.0-1.0
  --max-width <PX>
  --height <PX>
  --item-height <PX>
  --padding <PX>
  --border-width <PX>
```

---

## Configuration

Default config path:

- `%APPDATA%\rmenu\config.ini`

If missing, `rmenu` generates one from defaults.

### Minimal launcher section

```ini
[Launcher]
launcher_mode_default = true
enable_history = true
enable_start_menu = true
enable_path = true
history_max_items = 300
source_boost_history = 650
source_boost_start_menu = 480
source_boost_path = 0
blacklist_path_commands = powercfg,where,whoami,icacls,takeown,tasklist,taskkill,wevtutil,sfc,dism,gpupdate,bcdedit,reg,sc,netsh,wmic
```

### Notes

- Increase `source_boost_start_menu` if you want app shortcuts to dominate over PATH tools.
- Keep high-noise CLI commands in `blacklist_path_commands`.

### Index cache

- File: `%APPDATA%\rmenu\index.json`
- Format: versioned JSON + environment signature
- Auto-rebuild: when PATH or Start Menu roots change
- Manual rebuild:

```powershell
rmenu.exe --reindex
```

---

## Modules

`rmenu` supports external modules in two formats:

1. `modules/<name>.rmod` — single-file text module, recommended for distribution.
2. `modules/<name>/module.toml` + JS entry — directory module, recommended for development.

Modules can contribute:

- providers,
- commands,
- item decorations,
- quick-select keys,
- input accessories,
- controlled key hooks.

Module conventions for v1:

- standard local module directory: `modules/` relative to the current working directory;
- examples shipped in this repository: `modules/calculator.rmod`, `modules/local-scripts.rmod`, `modules/shortcuts.rmod`;
- module names should be lowercase kebab-case and stable (`local-scripts`, `shortcuts`);
- commands should use `/module::command` namespacing;
- capabilities must be minimal and explicitly declared;
- `.rmod` is the preferred sharing format;
- module `version` should use semver-like `major.minor.patch` strings and `api_version = 1` for this API generation.

The core remains authoritative over UI, ranking, dedupe, state, execution policy and error isolation.

Quick commands:

```powershell
rmenu.exe --modules-debug
```

Runtime commands inside `rmenu`:

```text
/modules.reload
/modules.list
/modules.telemetry.reset
```

Module documentation:

- `CORE_FREEZE_V1.md` — formal core v1 freeze declaration.
- `MODULES_ARCHITECTURE.md` — core/module boundary and freeze policy.
- `MODULES_QUICKSTART.md` — install, develop and debug modules quickly.
- `MODULES_API_SPEC_V1.md` — hooks, ctx, items and restrictions.
- `RMOD_SPEC_V1.md` — `.rmod` single-file format.
- `MANIFEST_SPEC_V1.md` — `module.toml` directory format.
- `MODULES_CAPABILITIES_MATRIX.md` — permissions and enforcement.
- `MODULES_AUTHORING_GUIDE.md` — module authoring guide.
- `MODULES_OPERATIONS_GUIDE.md` — operations/debugging guide.
- `DECISIONS.md` — accepted architecture decisions.
- `POST_FREEZE_ROADMAP.md` — post-freeze module/product roadmap.
- `RELEASE_CHECKLIST.md` — maintainer release checklist and artifact spec.
- `INSTALL.md` — zip install and manual update guide.
- `CHANGELOG.md` — release notes.
- `docs/release/BINARY_SIGNING.md` — binary signing and checksum policy.

---

## Diagnostics and performance

### Ranking debug

```powershell
rmenu.exe --debug-ranking pow
```

### Metrics

```powershell
rmenu.exe --metrics
```

Output includes:

- `startup_prepare_ms`
- `time_to_window_visible_ms`
- `time_to_first_paint_ms`
- `time_to_input_ready_ms`
- `search_p95_ms`
- `dataset_items`
- `dataset_estimated_bytes`
- `index_cache_bytes`

### Modules debug

```powershell
rmenu.exe --modules-debug
```

Output includes:

- API version
- loaded modules
- external descriptors
- running hosts
- host status
- request/error/timeout/restart counters
- recent host errors

### Minimum performance targets

Current v1 targets on a normal Windows development machine:

| Metric | Target |
| --- | ---: |
| `startup_prepare_ms` with cache | <= 250 ms |
| `startup_prepare_ms` with `--reindex` | <= 1000 ms |
| `time_to_window_visible_ms` | <= 100 ms |
| `time_to_input_ready_ms` | <= 100 ms |
| `search_p95_ms` | <= 10 ms |
| Provider global budget | <= configured `provider_total_budget_ms` |

These are product guardrails, not hard realtime guarantees. Regressions should be investigated when repeated release-mode measurements exceed the target.

### Reproducible benchmark routine

```powershell
cargo build --release
1..5 | ForEach-Object { .\target\release\rmenu.exe --metrics }
.\target\release\rmenu.exe --reindex --metrics
.\target\release\rmenu.exe --debug-ranking pow
.\target\release\rmenu.exe --debug-ranking code
.\target\release\rmenu.exe --debug-ranking calc
```

Baseline measured during Phase 4 verification on a 1108-item dataset:

```text
startup_prepare_ms: 143
startup_prepare_ms with --reindex: 567
time_to_window_visible_ms: 30-31
time_to_input_ready_ms: 36-37
search_p95_ms: 4.271-4.276
```

---

## Audits and reports

### Where audits live

- Unified audit artifacts: `artifacts/audits/`
- Archived codebase snapshot: `docs/audits/codebase-report-2026-04-22.md`
- Historical/private planning docs: `docs/historico/private-docs/`

### Latest audit (current repository snapshot)

- `artifacts/audits/audit-20260422-094201.txt`

Generate a new audit:

```powershell
./scripts/audit.ps1
```

Optional args:

```powershell
./scripts/audit.ps1 -MetricsRuns 10
./scripts/audit.ps1 -OutputPath .\artifacts\audits\audit-custom.txt
```

---

## Project structure

- `src/main.rs` - startup orchestration and mode selection
- `src/ui_win32.rs` - Win32 message loop and rendering
- `src/ranking.rs` - ranking pipeline and item ordering
- `src/fuzzy.rs` - fuzzy scoring primitives
- `src/sources/mod.rs` - history/start-menu/path indexing + cache
- `src/launcher.rs` - target launch backend
- `src/settings.rs` - config + CLI parsing
- `src/modules/` - module runtime, descriptors, IPC, host client, policies and types
- `src/module_host_main.rs` - external module host process

---

## PowerShell integration

See:

- `POWERSHELL_EXAMPLES.md`

---

## License

MIT. See `LICENSE`.
