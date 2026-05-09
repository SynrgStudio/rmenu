# Changelog

All notable changes to `rmenu` are documented here.

This project uses pragmatic versioning during the `0.x` line. Public module API compatibility is governed by `CORE_FREEZE_V1.md` and the v1 specs.

---

## [Unreleased]

## [0.4.1] - 2026-05-09

### Fixed

- rMenu now hides immediately and launches selected launcher targets from a worker thread, preventing Windows "Not Responding" hangs when applications such as Blender take over during startup.
- Daemon test compilation now excludes warm-launch UI code consistently with its Windows UI dependencies.

## [0.4.0] - 2026-05-07

### Added

- Daemon-backed warm launcher mode with prewarmed modules and global hotkeys for rMenu and RTasks panel.
- Persistent data-root layout under `C:\rMenuData` for modules, companions, config, and state.
- Native RSnip companion discovery/install workflow with direct IPC actions for `snip`, `rec`, and `ocr`.
- Native RTasks companion workflow with embedded `t ` task capture and `tasks` panel alias.
- Core-owned `/rmods` registry UI for installing, updating, and removing `.rmod` and `rpack` packages from the SynrgStudio registry.
- `rpack` folder-package support with integrity-checked file lists and module state storage under `<data_dir>\state\modules\<module-name>`.
- Generic resident-helper lifecycle for rpack/directory modules managed by `rmenu-daemon`.
- Resident helper rpacks in the registry: `taskbar-volume` and `thorium-tabs`.
- Isolated native `color-picker` rpack helper with screen magnifier, precision mode, and clipboard copy.
- `/rmods` status badges with colored install states.
- Local-only installed rmods are shown in `/rmods` even when missing from the remote registry.
- Timer rpack with premade/custom timers, running countdown accessory, and alarm stop handling when rMenu opens.

### Changed

- Future product expansion is tracked in `POST_FREEZE_ROADMAP.md` and should happen through modules first.
- RSnip public aliases are intentionally minimal: `snip`, `rec`, and `ocr`.
- RTasks public panel alias is intentionally minimal: `tasks`; task creation is handled through embedded `t ` input.
- Shortcuts rpack user data now lives in module state and migrates legacy package-local files automatically.
- Release target for this wave is `0.3.0`; version bump and GitHub Release publication are deferred to the release task.

### Fixed

- Color picker preview flicker, text overflow, click passthrough, mouse hook lag, and precision cursor behavior.
- Daemon warm-open latency by caching companion discovery and avoiding empty-input provider work.
- TaskbarVolume resident helper now passes middle-click through on taskbar app icons while retaining mute on empty taskbar background.
- ThoriumTabs resident helper now avoids Windows Alt+Tab UI and suppresses right-button release so the browser context menu does not open.
- `/rmods` status badge colors remain stable after moving the selection.
- Hidden module actions are excluded from launcher history.
- Input accessory clearing is scoped to the owning module.

### Release scope

- `0.3.0` should ship portable zip packaging first, then installer artifacts once installer validation passes.
- Auto-updater installation is not part of the initial `0.3.0` release unless the updater tasks are explicitly completed and accepted.

---

## [0.2.0] - 2026-04-24

### Added

- Core Closed v1 freeze declaration in `CORE_FREEZE_V1.md`.
- Modular runtime with `.rmod` and directory `module.toml` support.
- External module host process and IPC boundary.
- Capability enforcement for providers, commands, input accessories, and key hooks.
- Runtime actions including input accessory updates, item replacement, and query updates.
- Module diagnostics through `--modules-debug`.
- Performance diagnostics through `--metrics`.
- Ranking diagnostics through `--debug-ranking <query>`.
- Index rebuild support through `--reindex`.
- Example/validated modules:
  - `calculator.rmod`;
  - `local-scripts.rmod`;
  - `shortcuts.rmod`.
- Module authoring, operations, architecture, API, manifest, `.rmod`, capability, provider, error-isolation, and action specs/guides.

### Changed

- `rmenu` is now documented as a native Windows launcher and frozen modular command surface.
- Future feature work must be attempted as modules first.
- Core changes are limited by the v1 freeze policy.
- Public root documentation/specs are organized for GitHub readiness.

### Fixed

- External module actions now apply to the real launcher state where specified by the v1 contract.
- Input accessory rendering no longer exposes kind labels such as `[success]` in user-facing text.
- Undeclared external hooks are not invoked, avoiding permission-denied spam.
- External module failure, timeout, restart, backoff, auto-disable, and payload handling paths are covered by tests.
- Quoted executable targets with spaces are parsed correctly for launch.

### Validation

- Core closure completed through Phases 1-6.
- Current automated validation baseline: 74 tests passed, 0 failed.
- Manual validation completed for launcher mode, stdin/script mode, calculator, local scripts, shortcuts, and builtin-only/no-external-module mode.

---

[Unreleased]: https://github.com/SynrgStudio/rmenu/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/SynrgStudio/rmenu/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/SynrgStudio/rmenu/compare/v0.3.2...v0.4.0
[0.2.0]: https://github.com/SynrgStudio/rmenu/releases/tag/v0.2.0
