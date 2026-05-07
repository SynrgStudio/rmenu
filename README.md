# rmenu

`rmenu` is a lightweight native Windows launcher and modular command surface inspired by `dmenu`, built in Rust.

It is designed to stay fast and small at the core while letting features grow as modules, rpacks, resident helpers, and native companions.

<p align="center">
  <img src="https://raw.githubusercontent.com/SynrgStudio/rmenu/main/img.jpg" alt="rmenu screenshot" width="600"/>
</p>

---

## What rmenu does

rMenu can be used in two ways:

1. **Launcher mode**: open rMenu, fuzzy-search apps/commands from History, Start Menu, and PATH, then launch the selected item.
2. **Script/dmenu mode**: pass items with `stdin` or `-e`, choose one in the UI, and print the selected item to `stdout`.

On top of that base launcher, rMenu is also an extension host:

- `.rmod` modules add single-file JavaScript behavior.
- `rpack` packages add folder-based modules with config, assets, scripts, and optional helpers.
- resident helper rpacks run background helpers through `rmenu-daemon`.
- native companions such as RSnip and RTasks are installed and managed by rMenu but run as separate applications.
- `/rmods` is the built-in extension manager for modules, rpacks, and companions.

The product goal is a command center that stays instant for everyday use while allowing advanced features to live outside the core.

---

## Why rmenu exists

The core philosophy is:

> If a feature can be implemented as a module, it should not be hardcoded into the core.

The core stays responsible for the parts that must be reliable:

- Win32 UI, input, rendering, selection, and scrolling.
- fuzzy matching, ranking, dedupe, and source boosts.
- History, Start Menu, PATH, and direct input sources.
- native Windows launching through `ShellExecuteW` with controlled fallback.
- config, CLI parsing, diagnostics, cache, and performance metrics.
- module loading, IPC, permissions, timeouts, telemetry, and error isolation.
- `/rmods` install/update/remove workflow and package verification.

Feature-specific behavior belongs in extensions:

- calculator logic,
- shortcuts,
- timers,
- taskbar volume hooks,
- browser gestures,
- screenshots/OCR,
- task management,
- future clipboard/history/window/dev workflows.

This keeps the launcher fast, predictable, and recoverable even if an extension fails.

---

## Current status

rMenu is a native Windows launcher and frozen v1 modular platform.

Implemented core pieces:

- Win32 UI in `src/ui_win32.rs`.
- ranking in `src/ranking.rs` and `src/fuzzy.rs`.
- source indexing/cache in `src/sources/mod.rs`.
- launch backend in `src/launcher.rs`.
- settings, CLI, and data-root handling in `src/settings.rs`.
- module runtime, host process, IPC, capabilities, and telemetry in `src/modules/`.
- daemon and warm launcher path in `src/daemon_main.rs`.
- updater binary in `src/updater_main.rs`.

Public contracts are documented in:

- `CORE_FREEZE_V1.md`
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

---

## Installation

### From releases

Download the latest Windows x64 release from:

- <https://github.com/SynrgStudio/rmenu/releases>

Release assets normally include:

- `rmenu-v<version>-windows-x64.zip`
- `rmenu-setup-v<version>.exe`
- `SHA256SUMS.txt`

See `INSTALL.md` for zip install, installer behavior, startup daemon setup, checksum verification, and updates.

### Build from source

```bash
git clone https://github.com/SynrgStudio/rmenu.git
cd rmenu
cargo build --release
```

Binary output:

- `target/release/rmenu.exe`
- `target/release/rmenu-daemon.exe`
- `target/release/rmenu-module-host.exe`
- `target/release/rmenu-updater.exe`

---

## Quick usage

### Launcher mode

```powershell
rmenu.exe
```

No `-e` and no piped `stdin` means launcher mode. rMenu loads searchable items from History, Start Menu, PATH, direct input, and enabled modules.

### Script mode with `stdin`

```powershell
"Option 1`nOption 2`nOption 3" | rmenu.exe -p "Pick one"
```

### Script mode with `-e`

```powershell
rmenu.exe -e "Item A,Item B,Item C" -p "Pick one"
```

---

## Resident daemon and hotkeys

`rmenu-daemon.exe` is the resident helper. It keeps launcher state and module hosts warm so opening rMenu from a hotkey does not cold-start all module hosts each time.

Default hotkeys:

```text
Ctrl+Shift+Space  open rMenu
Ctrl+Space        open/toggle RTasks panel when RTasks is installed
```

Start daemon:

```powershell
rmenu-daemon.exe
```

Open an already-running daemon, or start one and open once:

```powershell
rmenu-daemon.exe --open
```

Stop daemon:

```powershell
rmenu-daemon.exe --quit
```

Install startup entry:

```powershell
rmenu-daemon.exe --hotkey "ctrl+shift+space" --rmenu "C:\rMenu\rmenu.exe" --install-startup
```

Remove startup entry:

```powershell
rmenu-daemon.exe --uninstall-startup
```

Daemon logs:

```text
%APPDATA%\rmenu\rmenu-daemon.log
```

The daemon also manages resident helper rpacks. It starts/stops helpers, passes module/state/config paths, and logs failures. The feature itself remains owned by the rpack helper, not by rMenu core.

---

## Persistent data root

rMenu keeps mutable product data outside the app directory.

Default Windows data root:

```text
C:\rMenuData
```

Layout:

```text
<data_dir>\
  modules\
  companions\
    rsnip\
      rsnip.exe
      config\
      state\
      logs\
    rtasks\
      rtasks.exe
      config\
      state\
      logs\
  config\
  state\
    modules\
      <module-id>\
    downloads\
    rmods-registry-cache.json
    rmods-installed.json
```

Overrides:

- `--data-dir <PATH>` or `RMENU_DATA_DIR` changes the full data root.
- `--modules-dir <PATH>` or `RMENU_MODULES_DIR` explicitly overrides module discovery.
- if no module override is set, modules load from `<data_dir>\modules`.

Modules should store user-created state in:

```text
<data_dir>\state\modules\<module-name>\
```

JavaScript modules access that path with:

```js
ctx.moduleStateDir()
```

Do not store durable user data inside an installed rpack folder; rpack updates replace package files.

---

## Sources, ranking, and launch behavior

In launcher mode rMenu builds a dataset from:

- History.
- Start Menu shortcuts.
- PATH executables.
- direct typed input.
- loaded module providers.

Ranking combines fuzzy matching with source-aware boosts. Start Menu and History can be boosted above noisy PATH tools. Technical executable names remain searchable, so both friendly names and commands like `mspaint` or `powershell` work.

History entries are persisted unless the target is hidden/internal, for example `hidden:powershell.exe ...` used by modules for background actions.

Index cache:

```text
%APPDATA%\rmenu\index.json
```

The cache is versioned and includes an environment signature. It auto-invalidates when PATH or Start Menu roots change. Force rebuild:

```powershell
rmenu.exe --reindex
```

---

## Modules, rmods, and rpacks

rMenu supports two JavaScript extension formats.

### `.rmod`

A `.rmod` is a single UTF-8 module file.

Install target:

```text
<data_dir>\modules\<id>.rmod
```

Use `.rmod` for compact modules that fit in one file.

### `rpack`

An `rpack` is a folder module with a manifest and files.

Install target:

```text
<data_dir>\modules\<id>\
  module.toml
  module.js
  config.json
  README.md
  bin\
  assets\
  scripts\
```

Use `rpack` when a module needs config, docs, assets, scripts, native helpers, sounds, or multiple files. `rpack` is a folder distribution format, not a zip format.

### Directory modules for development

A local development module uses the same structure:

```text
modules\<name>\module.toml
modules\<name>\module.js
```

Run with:

```powershell
rmenu.exe --modules-dir .\modules --modules-debug
```

Module discovery order:

1. `--modules-dir`
2. `RMENU_MODULES_DIR`
3. `<data_dir>\modules`
4. `modules` next to `rmenu.exe`
5. current working directory `modules` as development fallback

---

## Module API model

Modules run outside the main process through `rmenu-module-host.exe`. The core communicates with hosts over IPC and validates all actions.

Public hooks:

```ts
onLoad(ctx)
onUnload(ctx)
onQueryChange(query, ctx)
onSelectionChange(item, index, ctx)
onKey(event, ctx)
onSubmit(item, ctx)
onCommand(command, args, ctx)
provideItems(query, ctx) -> Item[]
decorateItems(items, ctx) -> Item[]
```

Modules can contribute:

- providers,
- commands,
- decorations,
- quick-select keys,
- input accessories,
- controlled key hooks.

Official capabilities:

```text
providers
commands
decorate-items
input-accessory
keys
```

A module must declare capabilities in `.rmod` or `module.toml`. Operations without the matching capability are rejected.

Item shape:

```ts
type Item = {
  id: string
  title: string
  subtitle?: string
  source?: string
  target?: string
  quickSelectKey?: string
  badge?: string
  hint?: string
}
```

Input accessory shape:

```ts
type InputAccessory = {
  text: string
  kind?: "info" | "success" | "warning" | "error" | "hint"
  priority?: number
}
```

The core remains authoritative over rendering, ranking, dedupe, execution policy, timeouts, and error isolation. Modules cannot draw UI directly, access Win32/GDI, replace ranking, mutate arbitrary state, or bypass capabilities.

Runtime module commands inside rMenu:

```text
/modules.reload
/modules.list
/modules.telemetry.reset
```

Diagnostics:

```powershell
rmenu.exe --modules-debug
```

---

## `/rmods` extension manager

`/rmods` is the built-in extension manager. It is core-owned, not a privileged module.

It can install, update, and remove:

| Kind | Meaning | Install target |
| --- | --- | --- |
| `rmod` | single-file JavaScript module | `<data_dir>\modules\<id>.rmod` |
| `rpack` | folder JavaScript module/helper package | `<data_dir>\modules\<id>\` |
| `companion` | native managed app | `<data_dir>\companions\<id>\` |

Default registry:

```text
https://raw.githubusercontent.com/SynrgStudio/rmods/main/registry.json
```

Registry repo:

```text
https://github.com/SynrgStudio/rmods
```

Registry source layout:

```text
rmods/
  modules/
    example.rmod
  rpacks/
    shortcuts/
      module.toml
      module.js
      config.json
      README.md
  companions/
    rsnip.json
    rtasks.json
  registry.json
  scripts/
    generate-registry.*
  .github/
    workflows/
      update-registry.yml
```

`registry.json` is generated by GitHub Actions from source files and should not be edited by hand.

`/rmods` security and install policy:

- validates registry schema and package kind,
- validates module IDs,
- rejects unsafe paths, absolute paths, and traversal,
- downloads to `<data_dir>\state\downloads`,
- verifies file size and SHA-256,
- stages installs before replacing package files,
- records installed version/kind/checksum in `<data_dir>\state\rmods-installed.json`,
- refreshes runtime state after changes.

Controls:

```text
/rmods          open registry list
/rmods <query>  filter registry list
Up/Down         move cursor
Space           mark/unmark pending change
F5/Ctrl+R       refresh registry
Ctrl+U          mark update-available packages
Enter           apply pending installs/updates/removals
Esc             close rMenu
```

Markers:

```text
[x] installed
[ ] not installed
[/] pending change
```

Companions show a visible `COMPANION` badge. Local-only installed rpacks are also shown even when missing from the remote registry.

See `docs/rmods-registry.md` for the registry schema and generation policy.

---

## Resident helper rpacks

A resident helper rpack is a folder module that declares a background helper in `module.toml`:

```toml
[resident]
enabled = true
command = "bin/helper.exe"
autostart = true
shutdown = "kill"
```

The daemon lifecycle contract is intentionally generic:

- discover resident declarations,
- start helpers from module-local relative paths,
- pass module name, module dir, state dir, and config path,
- stop helpers on daemon quit, uninstall, update, or helper sync,
- log failures without crashing rMenu.

The core must not implement helper-specific behavior. Examples:

| rpack | behavior |
| --- | --- |
| `taskbar-volume` | wheel over taskbar changes volume; middle click mutes |
| `thorium-tabs` | Thorium-specific tab mouse gestures |
| `color-picker` | launches an isolated native picker helper |

Security note: resident helpers may use global hooks or other OS integrations. Installing one is a trust decision.

Troubleshooting:

```text
%APPDATA%\rmenu\rmenu-daemon.log
```

Expected daemon log lines include helper start/stop events.

---

## Native companions

Companions are separate native applications managed by rMenu. They are not JavaScript modules and are not loaded into the module host.

Companions are installed under:

```text
<data_dir>\companions\<id>\
```

They are preferably installed and updated through `/rmods`.

Compatibility commands still exist:

```text
/install rsnip
/install rtasks
```

or CLI:

```powershell
rmenu.exe --install rsnip
rmenu.exe --install rtasks
```

### RSnip

RSnip owns screenshot, recording, and OCR behavior. rMenu exposes simple aliases and dispatches to RSnip through native/IPC integration.

Install target:

```text
<data_dir>\companions\rsnip\rsnip.exe
```

Discovery order:

1. `<data_dir>\companions\rsnip\rsnip.exe`
2. `RMENU_RSNIP_PATH`
3. `C:\rSnip\target\release\rsnip.exe`
4. unambiguous `PATH` match

Aliases:

```text
snip  screenshot region
rec   screen recording
ocr   OCR region
```

### RTasks

RTasks is a native task backend/panel companion. rMenu owns embedded task input.

Install target:

```text
<data_dir>\companions\rtasks\rtasks.exe
```

Embedded task capture:

```text
t comprar pan mañana
```

While input starts with `t `:

```text
Alt+1  toggle TODO
Alt+2  toggle DOING
Alt+3  toggle DONE
Alt+Q  toggle high priority
Alt+W  toggle medium priority
Alt+E  toggle low priority
```

Panel alias:

```text
tasks
```

Daemon panel hotkey:

```text
Ctrl+Space
```

When the panel closes, focus is restored to the previously focused window when possible.

---

## Included and known extension examples

Repository/release examples may include:

| Extension | Kind | Purpose |
| --- | --- | --- |
| `calculator.rmod` | `rmod` | simple calculator example |
| `local-scripts.rmod` | `rmod` | local script launcher example |
| `shortcuts.example.rmod` | `rmod` | example shortcut aliases |
| `timer` | `rpack` | premade/custom timers, countdown accessory, alarm sound |
| `taskbar-volume` | resident `rpack` | taskbar volume control helper |
| `thorium-tabs` | resident `rpack` | Thorium tab mouse gestures |
| `color-picker` | helper `rpack` | native screen color picker |
| `rsnip` | companion | screenshots, recordings, OCR |
| `rtasks` | companion | tasks backend/panel |

The timer rpack demonstrates a multi-file package with config, a PowerShell helper, a sound asset, module state, hidden background actions, and an input accessory countdown.

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
  --modules-dir <PATH>    Override module discovery directory for this run.
  --data-dir <PATH>       Persistent rMenu data root (modules/companions/config/state).
  --install <NAME>        Install native companion (rsnip/rtasks latest GitHub release).
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

```text
%APPDATA%\rmenu\config.ini
```

If missing, rMenu generates one from defaults.

Minimal launcher section:

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

Notes:

- Increase `source_boost_start_menu` if app shortcuts should dominate over PATH tools.
- Keep high-noise CLI commands in `blacklist_path_commands`.
- Use `--data-dir` or `RMENU_DATA_DIR` when running portable/dev layouts.

---

## Diagnostics and performance

Ranking debug:

```powershell
rmenu.exe --debug-ranking pow
```

Metrics:

```powershell
rmenu.exe --metrics
```

Modules debug:

```powershell
rmenu.exe --modules-debug
```

Output includes:

- loaded modules and descriptors,
- external hosts,
- host status,
- request/error/timeout/restart counters,
- recent module errors,
- startup/search/cache timings.

Current v1 performance guardrails on a normal Windows development machine:

| Metric | Target |
| --- | ---: |
| `startup_prepare_ms` with cache | <= 250 ms |
| `startup_prepare_ms` with `--reindex` | <= 1000 ms |
| `time_to_window_visible_ms` | <= 100 ms |
| `time_to_input_ready_ms` | <= 100 ms |
| `search_p95_ms` | <= 10 ms |
| Provider global budget | <= configured `provider_total_budget_ms` |

Benchmark routine:

```powershell
cargo build --release
1..5 | ForEach-Object { .\target\release\rmenu.exe --metrics }
.\target\release\rmenu.exe --reindex --metrics
.\target\release\rmenu.exe --debug-ranking pow
.\target\release\rmenu.exe --debug-ranking code
.\target\release\rmenu.exe --debug-ranking calc
```

---

## Maintainer release workflow

Release docs:

- `scripts/release-local.ps1` — local maintainer release script.
- `.github/workflows/release.yml` — GitHub release artifact workflow.
- `INSTALL.md` — install/update instructions.
- `CHANGELOG.md` — release notes.
- `RELEASE_CHECKLIST.md` — release checklist and artifact spec.
- `docs/release/BINARY_SIGNING.md` — signing/checksum policy.

Local package-only example:

```powershell
cargo build --release
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-local.ps1 -Version 0.4.0 -PackageOnly -SkipValidation -IncludeInstaller
```

---

## Audits and reports

Audit artifacts:

- `artifacts/audits/`
- `docs/audits/codebase-report-2026-04-22.md`
- `docs/historico/private-docs/`

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

```text
src/main.rs              startup orchestration and mode selection
src/ui_win32.rs          Win32 message loop and rendering
src/ranking.rs           ranking pipeline and item ordering
src/fuzzy.rs             fuzzy scoring primitives
src/sources/mod.rs       history/start-menu/path indexing + cache
src/launcher.rs          target launch backend
src/settings.rs          config + CLI parsing + data dirs
src/modules/             module runtime, descriptors, IPC, host client, policies and types
src/module_host_main.rs  external JavaScript module host process
src/daemon_main.rs       resident daemon, hotkeys, warm launcher, helper lifecycle
src/updater_main.rs      updater binary
modules/                 local/example modules and rpacks
scripts/                 audit/release scripts
installer/               installer build files
docs/                    workflow, registry, updater, release docs
```

---

## PowerShell integration

See:

- `POWERSHELL_EXAMPLES.md`

---

## License

MIT. See `LICENSE`.
