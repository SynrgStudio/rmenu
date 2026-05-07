# DECISIONS — rmenu

Short record of active architectural decisions.

Historical or exploratory decisions may live under `docs/historico/`. This file summarizes current decisions that affect the public contract.

---

## DEC-001 — Modular core v1 is frozen around small primitives

Status: Accepted
Date: 2026-04-24

### Context

`rmenu` evolved from a native launcher into an extensible command surface. To avoid accidental core growth, the module architecture needs an explicit boundary.

### Decision

Core v1 is stabilized around these primitives:

- providers,
- commands,
- decorations,
- input accessory,
- capabilities,
- key hooks,
- runtime actions.

New features must be attempted as modules first. A new primitive is accepted only with evidence from multiple real modules and a documented decision.

### Consequences

- The core stops receiving feature-specific behavior.
- Functional expansion happens through modules.
- The public contract is documented in frozen v1 specs.
- Breaking changes require a new API version or an explicit decision.

---

## DEC-002 — `.rmod` v1 is plain text

Status: Accepted
Date: 2026-04-24

### Context

Modules need a distributable format that is easy to audit.

### Decision

`.rmod` v1 is always plain UTF-8 text with the `#!rmod/v1` magic line, key-value headers, and named blocks.

If a packed or binary format exists in the future, it must use a different extension.

### Consequences

- Modules are readable and auditable.
- The format is easy to version.
- The loader can normalize `.rmod` files and module directories into the same descriptor.

---

## DEC-003 — The core renders; modules declare intent

Status: Accepted
Date: 2026-04-24

### Context

Allowing arbitrary rendering from modules would compromise stability, performance, and visual consistency.

### Decision

Modules cannot draw UI, access Win32/GDI, or modify global layout. They can only provide declarative primitives: items, badges, hints, quick-select keys, and input accessories.

### Consequences

- UI remains consistent.
- The core keeps control of layout and performance.
- Complex visual features must wait for an official primitive or live outside the core.

---

## DEC-004 — Fail the module, not the launcher

Status: Accepted
Date: 2026-04-24

### Context

External modules can fail, hang, or return invalid payloads.

### Decision

Errors, timeouts, and invalid payloads are isolated per module/host. The launcher must remain operational.

### Consequences

- Runtime applies timeouts, backoff, and disable policies.
- `--modules-debug` exposes telemetry.
- The core prioritizes stability over completing results from faulty modules.

---

## DEC-005 — Local intents use an explicit prefix and `replaceItems`

Status: Accepted
Date: 2026-04-24

### Context

Some modules, such as `local-scripts`, need a scoped search where their results do not compete with History, Start Menu, PATH, or global fuzzy ranking. `ProviderDef.priority` only controls provider execution order, and `dedupe_source_priority` only resolves duplicates; neither should be interpreted as general visual priority.

### Decision

Local-intent modules must use an explicit query prefix and `ctx.replaceItems(...)` to enter a scoped result mode. For `local-scripts`, the v1 prefix is `>`.

Examples:

- `>` lists local scripts.
- `> bu` filters local scripts.
- `build` keeps the normal global launcher behavior.

### Consequences

- The global launcher is not polluted by local results without explicit intent.
- The module can pre-sort exact/prefix/contains matches inside its scoped list.
- No magic boosts, semantic hints, or core ranking changes are added.

---

## DEC-006 — AHK suite migration uses minimal general core primitives

Status: Accepted
Date: 2026-05-05

### Context

The AHK suite migration needs several Command Center-style modules and helper-backed tools. The core is frozen v1, so the migration must not add local workflow behavior directly to the core.

### Decision

The current migration wave accepts only these general core changes:

- robust module directory resolution for dev/debug and installed usage;
- minimal elevated launch support through Windows `runas`;
- lightweight rmenu-style toast feedback for module/helper status.

The active module wave is split into small cohesive modules such as `web-query`, `url-open`, `terminal`, `pi-launcher`, helper launchers, and existing `shortcuts` guidance.

### Consequences

- Anytype, TweetFlow, full item actions, `onSubmit`, copy actions, keep-open flows, local module secret config, and daemon features remain deferred.
- Global hotkeys, window management, text expansion, browser gestures, taskbar volume, and similar resident automation stay out of the core and ordinary `.rmod` modules.
- See `docs/ahk-migration/DECISION.md` for the session-specific migration boundary.

---

## DEC-007 — RSnip is an optional native companion capability

Status: Accepted
Date: 2026-05-05

### Context

RSnip is a separate native snipping/recording/OCR tool. It can run standalone with its own daemon and global hotkeys, but the intended combined product experience is that installing RSnip next to rMenu makes RSnip feel like part of rMenu.

A launcher-target wrapper is not enough for that experience. A command such as `snip` in rMenu must not shell out through PowerShell or a visible console, and should not rely on `rsnip.exe snip` as the normal integration path. The menu path and hotkey path should converge on the same RSnip daemon action semantics.

### Decision

When RSnip is installed, rMenu treats it as a first-class optional companion capability:

- rMenu discovers RSnip through an explicit and deterministic discovery order.
- rMenu daemon coordinates RSnip lifecycle in integrated mode.
- rMenu menu actions for snip, record, and OCR call RSnip through direct IPC, not through PowerShell or generic process-launch targets.
- Standalone RSnip remains valid: if rMenu is absent, RSnip keeps running its own daemon and global hotkeys.
- Integrated mode must have one clear owner/coordinator for global hotkeys to avoid duplicate registration and stale shortcut capture.

Discovery order for the current implementation plan:

1. Explicit rMenu config or environment override, if added by the implementation task.
2. Development path: `C:\rSnip\target\release\rsnip.exe`.
3. Future packaged install location or registry marker.
4. PATH fallback only if it is safe and unambiguous.

The IPC contract is RSnip's named-pipe protocol on `\\.\pipe\rsnip`, with JSON commands for `snip`, `record`, `ocr`, and shutdown/status-style reachability as supported by RSnip.

### Consequences

- Transitional `.rmod` wrappers for `snip`, `record`, and `ocr` are not the target architecture.
- rMenu needs a small native RSnip companion client for discovery, lifecycle, IPC, and error reporting.
- The core does not absorb RSnip capture/record/OCR implementation; RSnip remains the owner of those features.
- User-facing errors should say whether RSnip is missing, unreachable, or version-incompatible.
- Documentation and manual validation must cover both standalone and integrated modes.

---

## DEC-008 — rMenu uses a persistent data root for modules, companions, config, and state

Status: Accepted
Date: 2026-05-05

### Context

Users need a simple persistent rMenu setup. Modules, optional native companions, configuration, and state should live outside the application install directory. The default Windows data root is `C:\rMenuData`, and a later rMenu install should be able to reuse that folder without overwriting existing data.

RSnip is the first native companion to use this model. Future companions such as rTask should fit the same layout.

### Decision

rMenu has a persistent data root, referred to as `<data_dir>`. On Windows, the default is `C:\rMenuData`. The official layout is:

```text
<data_dir>\
  modules\
  companions\
    rsnip\
      rsnip.exe
      config\
      state\
      logs\
  config\
  state\
```

Directory purposes:

- `modules\`: `.rmod` files and directory modules discovered by rMenu.
- `companions\`: native companion applications managed or discovered by rMenu.
- `companions\rsnip\`: rMenu-managed RSnip install root.
- `config\`: rMenu-level configuration owned by the selected data root.
- `state\`: rMenu-level runtime state, metadata, install records, and future non-cache state.

`modules_dir` is derived from `<data_dir>\modules` by default. Existing `--modules-dir` and `RMENU_MODULES_DIR` behavior remains valid as an explicit modules-only override for development, debugging, and migration.

A future installer should default to `C:\rMenuData` and optionally write a small bootstrap pointer under `%APPDATA%\rmenu` if the user selects another data root. The selected folder is reused as-is when it already contains rMenu data. The installer must not overwrite existing modules, companions, config, or state without explicit user approval.

### RSnip companion layout

The rMenu-managed RSnip executable path is:

```text
<data_dir>\companions\rsnip\rsnip.exe
```

Future portable RSnip config/state/logs should live under:

```text
<data_dir>\companions\rsnip\config\
<data_dir>\companions\rsnip\state\
<data_dir>\companions\rsnip\logs\
```

Until RSnip supports portable config/state overrides, rMenu may still install and run the executable from this location while RSnip uses its current standalone config behavior.

### Consequences

- rMenu-managed companion installs are portable with the data root.
- RSnip discovery must prefer `<data_dir>\companions\rsnip\rsnip.exe` once data-dir support exists.
- The data root is a general UX primitive, not an RSnip-specific path.
- The installer can support "choose existing data folder" without requiring module or companion reinstall.

---

## DEC-009 — rMods registry is core-owned

Status: Accepted
Date: 2026-05-06

### Context

Installing and updating modules requires filesystem write access to the rMenu data root. Implementing a store as a normal `.rmod` would make a module privileged enough to install, update, or remove other modules.

### Decision

`/rmods` is owned by rMenu core. It fetches the generated GitHub registry, verifies package integrity, stages installs, updates installed metadata, and reloads modules. Registry modules can be single-file `rmod` packages or folder-based `rpack` packages.

### Consequences

- Modules cannot self-grant install permissions.
- `registry.json` is generated by the registry repository workflow and is not edited manually.
- `rpack` updates replace package contents, so user-created module data belongs in `<data_dir>\state\modules\<module-name>`.

---

## DEC-010 — RTasks is backend/panel companion; rMenu owns task input

Status: Accepted
Date: 2026-05-06

### Context

RTasks has its own UI, but opening a second quick-add bar from rMenu creates redundant input. rMenu is already the active text input surface.

### Decision

Typing `t ` in rMenu enters embedded RTasks mode. Enter sends an `add_task` IPC request to RTasks. Status and priority are edited in rMenu with `Alt+1/2/3` and `Alt+Q/W/E`. The only public launcher alias for the RTasks panel is `tasks`; the daemon-level `Ctrl+Space` hotkey also toggles the panel.

### Consequences

- rMenu does not open RTasks Quick Add for integrated capture.
- RTasks remains useful standalone.
- Integrated panel focus restoration belongs to the rMenu/RTasks companion boundary.

---

## DEC-011 — RSnip launcher aliases are intentionally minimal

Status: Accepted
Date: 2026-05-06

### Context

RSnip exposes three user actions through rMenu. Extra synonyms make stale history/cache entries and accidental matches harder to reason about.

### Decision

The rMenu-facing RSnip aliases are limited to:

```text
snip
rec
ocr
```

These dispatch to direct RSnip IPC and do not shell through PowerShell or CMD.

### Consequences

- `record`, `screen`, `screenshot`, and `text` are not public rMenu aliases.
- The old wrapper `.rmod` files are not part of the integrated path.

---

## DEC-012 — Resident rpack helpers are lifecycle-managed, not core features

Status: Accepted
Date: 2026-05-06

### Context

Some AHK modules are resident OS integrations rather than launcher commands. Examples include taskbar volume mouse-wheel behavior and Thorium tab mouse gestures. These should work while rMenu is running in the background, but embedding each behavior in rMenu core would grow the core with feature-specific automation.

### Decision

rMenu may add a generic resident-helper lifecycle primitive for directory/rpack modules. A module can declare one resident helper in `module.toml` under `[resident]`. `rmenu-daemon` starts, tracks, syncs, and stops helpers. Feature logic remains entirely inside the module's helper executable.

The core is allowed to know:

- module name;
- module directory;
- module state directory;
- helper relative command and static args;
- process lifecycle state.

The core is not allowed to know feature semantics such as taskbar volume, Thorium, browser tabs, text expansion, or window snapping.

### Consequences

- TaskbarVolume and ThoriumTabs migrate as rpacks with native helpers.
- Future resident features can reuse the same lifecycle primitive.
- Resident helpers are a stronger trust boundary than ordinary JS modules because they may install global hooks or interact with the OS.
- `/rmods` and docs should make resident-helper behavior visible enough for users to decide whether to install the module.

---

## DEC-013 — Updates are explicit, non-intrusive, and checksum-verified

Status: Accepted
Date: 2026-05-07

### Context

rMenu will be distributed through GitHub Releases with a portable zip and Windows installer. Once users install rMenu, they should be notified when a newer release exists, but the launcher must remain fast and non-intrusive.

### Decision

rMenu may show a startup update notice when cached release metadata says a newer version exists. The notice is dismissed for the current open by pressing any normal key. `Enter` starts the install flow. `Ctrl+Enter` opens the GitHub Release changelog. If the user does not install, the notice can appear again on the next rMenu open.

Updates are not installed silently in the MVP. Installation is delegated to a separate `rmenu-updater.exe` process so running binaries can be replaced safely. The updater must verify SHA256 from the same GitHub Release before running an installer.

### Consequences

- Normal rMenu use is never blocked by update checks or prompts.
- Network update checks should happen in background/daemon paths or through explicit forced checks, not directly in the hot launcher open path.
- A failed update check or failed install must be recoverable and logged.
- SHA256 verification is required before installer execution; Authenticode signing remains future hardening.

---

## DEC-014 — `/rmods` is the unified extension hub for modules and companions

Status: Accepted
Date: 2026-05-07

### Context

RSnip and RTasks are native companion applications, not ordinary JavaScript modules. They still need the same user-facing distribution lifecycle as rMods: discovery, install, update, uninstall, checksum verification, and data-root awareness.

Adding a separate `/companions` UI would duplicate registry, filtering, status, and install logic. Bundling companions directly into the rMenu installer would make the installer a package manager and would make portable installs weaker.

### Decision

`/rmods` is the unified rMenu extension hub. The registry supports explicit package kinds:

- `rmod`: single-file module package installed under `<data_dir>\modules`;
- `rpack`: folder module package installed under `<data_dir>\modules`;
- `companion`: native managed application installed under `<data_dir>\companions\<id>`.

Companion entries must be visually distinct in `/rmods` with a `COMPANION` badge/tone and copy that says companion install/update/remove. The rMenu installer remains simple: install rMenu, preserve/select the data root, and optionally start the daemon. It does not bundle RSnip or RTasks in this wave.

Existing `/install rsnip` and `/install rtasks` commands may remain as temporary compatibility aliases, but the primary UX is `/rmods`.

### Consequences

- Users have one place to manage extensions.
- rMenu keeps installer responsibilities small.
- Companion package metadata must include enough information for secure install and clear display.
- Native companion runtime behavior remains IPC-backed; installing via `/rmods` does not turn companions into modules.
