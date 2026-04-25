# Changelog

All notable changes to `rmenu` are documented here.

This project uses pragmatic versioning during the `0.x` line. Public module API compatibility is governed by `CORE_FREEZE_V1.md` and the v1 specs.

---

## [Unreleased]

### Added

- Wave 0 packaging/release documentation and workflow preparation.

### Changed

- Future product expansion is tracked in `POST_FREEZE_ROADMAP.md` and should happen through modules first.

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

[Unreleased]: https://github.com/SynrgStudio/rmenu/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/SynrgStudio/rmenu/releases/tag/v0.2.0
