---
continuity_session: CONT-2026-04-25-0858-wave0-packaging-release
created_at: 2026-04-25 08:58
updated_at: 2026-04-25 09:04
planned_at: 2026-04-25 09:00
status: active
goal: Preparar Wave 0 Packaging/release para rmenu post-freeze
---

# ACTIVE_QUEUE.md

## Current goal

Preparar Wave 0 Packaging/release para `rmenu` post-freeze.

Objective: leave `rmenu` ready for repeatable, trustworthy releases after Core Closed v1.

---

## Planning checkpoint

Planned at: 2026-04-25 09:00

Planning inputs:

- `AUTONOMOUS_EXECUTION.md`
- `STATE.md`
- `POST_FREEZE_ROADMAP.md`
- `Cargo.toml`
- current repository docs/workflow layout

Plan result:

- Wave 0 is executable as a docs/workflow/tooling phase.
- No core runtime behavior changes are required.
- Full GitHub Actions validation is GitHub-only and should be recorded as a remaining external validation after local checks pass.

---

## Queue policy

- Status values: `pending`, `in_progress`, `done`, `blocked`, `partial`, `cancelled`.
- Never renumber existing task IDs.
- Pick first `pending` task whose dependencies are `done`.
- Preserve claims unless stale or explicitly overridden.
- Do not implement core behavior changes unless a real packaging bug requires it and the user approves when needed.
- No commits unless the user explicitly asks.

---

## Product decisions already confirmed

- Manual smoke tests were validated by the user and should be documented as release smoke checks to repeat:
  - launcher opens;
  - search works;
  - Enter launches;
  - Esc cancels;
  - stdin mode returns selected item;
  - repo root with modules works;
  - empty cwd without external modules works.
- Windows release artifact should include `rmenu.exe`, `rmenu-module-host.exe`, `config_example.ini`, README/docs, and module examples where appropriate.
- `shortcuts.rmod` with local hardcoded targets must not ship as an active default module.
- Prefer `modules/examples/shortcuts.example.rmod` or equivalent with generic/documented targets.
- GitHub Actions release workflow should run on Windows and create artifacts/releases from `v*` tags.
- No MSI and no auto-updater in Wave 0.
- Unsigned zip + SHA256 checksums is acceptable initially; binary signing is research/documentation for now.

---

## Queue

### T001 — Release checklist and artifact specification

Status: done  
Claimed by: current-agent  
Started: 2026-04-25 09:01  
Last update: 2026-04-25 09:04  
Scope:
- Create `RELEASE_CHECKLIST.md`.
- Define pre-release and post-release checks.
- Define Windows x64 zip artifact contents.
- Include automated validation and manual smoke checks.
- Include checksum and release notes steps.
- Specify module packaging policy, including examples vs active modules.
DoD:
- `RELEASE_CHECKLIST.md` exists.
- Checklist includes `cargo fmt`, `cargo test`, `cargo check`, `cargo build --release`.
- Checklist records user-validated smoke tests as checks to repeat before release.
- Artifact layout is explicit and names a Windows x64 zip.
- `shortcuts.rmod` active-default risk is documented.
- Checklist states full GitHub workflow validation happens on GitHub.
Validation:
- Review markdown for accurate commands and paths.
Files likely touched:
- `RELEASE_CHECKLIST.md`
- `STATE.md`
Risk: low  
Depends on:
- none
Notes:
- This task should not modify core code.
- This task defines release packaging policy that later tasks consume.

---

### T002 — Install and update documentation

Status: done  
Claimed by: current-agent  
Started: 2026-04-25 09:01  
Last update: 2026-04-25 09:04  
Scope:
- Create `INSTALL.md`.
- Document zip install.
- Document PATH setup.
- Document manual update process.
- Mention future Scoop/winget possibilities.
- State no auto-updater for now.
- Mention unsigned binary/checksum verification and link signing research once available.
DoD:
- `INSTALL.md` exists.
- README links to install docs if appropriate.
- Install/update story is clear for unsigned zip releases.
- User can install by extracting zip and adding the directory to PATH.
- Manual update process says to replace extracted files with a newer release.
Validation:
- Review docs for Windows/PowerShell command correctness.
Files likely touched:
- `INSTALL.md`
- `README.md`
- `STATE.md`
Risk: low  
Depends on:
- T001
Notes:
- Do not implement installer logic in Wave 0.

---

### T003 — Binary signing research documentation

Status: done  
Claimed by: current-agent  
Started: 2026-04-25 09:01  
Last update: 2026-04-25 09:04  
Scope:
- Create `docs/release/BINARY_SIGNING.md`.
- Document current unsigned status.
- Document SmartScreen/trust implications.
- Compare no signing + checksum, standard code signing, EV signing, and future CI signing.
- Recommend current approach.
DoD:
- Binary signing research doc exists.
- Current recommendation is explicit: unsigned zip with SHA256 checksums for now, signing research before broader public distribution.
- Doc explains that checksums verify integrity but not publisher identity.
- Doc explains that signing is not required to complete Wave 0.
Validation:
- Review docs for clear risk/trust language.
Files likely touched:
- `docs/release/BINARY_SIGNING.md`
- `STATE.md`
Risk: low  
Depends on:
- T001
Notes:
- Do not buy/configure certificates in Wave 0.

---

### T004 — Changelog and release notes baseline

Status: done  
Claimed by: current-agent  
Started: 2026-04-25 09:01  
Last update: 2026-04-25 09:04  
Scope:
- Create `CHANGELOG.md` if absent.
- Add `[Unreleased]` section.
- Add initial `0.2.0` / Core Closed v1 release notes.
- Keep notes concise and user-facing.
- Mention module platform, three validated modules, diagnostics, and packaging/release docs.
DoD:
- `CHANGELOG.md` exists.
- It documents Core Closed v1, module system, validation, and release packaging status.
- README links to changelog if appropriate.
- Changelog does not claim a GitHub release was published if it was not.
Validation:
- Review markdown.
Files likely touched:
- `CHANGELOG.md`
- `README.md`
- `STATE.md`
Risk: low  
Depends on:
- T001
Notes:
- Do not bump version unless explicitly needed.

---

### T005 — Module example packaging policy and shortcut example

Status: done  
Claimed by: current-agent  
Started: 2026-04-25 09:01  
Last update: 2026-04-25 09:04  
Scope:
- Define how modules are packaged in the release artifact.
- Prevent local-user `modules/shortcuts.rmod` targets from becoming active release defaults.
- If needed, create a safe example shortcut module/file for packaging.
- Prefer generic targets such as `notepad.exe` and `wt.exe` for examples.
DoD:
- Packaging policy identifies active modules vs example modules.
- `shortcuts.rmod` local Blender target is not packaged as an active default.
- If an example file is created, it is clearly named/documented as example-only.
- Release workflow task can package examples safely.
Validation:
- Review module/example paths and docs.
Files likely touched:
- `modules/examples/shortcuts.example.rmod` or equivalent docs/example file
- `RELEASE_CHECKLIST.md`
- `STATE.md`
Risk: medium  
Depends on:
- T001
Notes:
- This is docs/examples/tooling only; do not change core module semantics.
- If creating an example file from `shortcuts.rmod`, remove user-local targets.

---

### T006 — GitHub Actions release workflow

Status: done  
Claimed by: current-agent  
Started: 2026-04-25 09:01  
Last update: 2026-04-25 09:04  
Scope:
- Create `.github/workflows/release.yml`.
- Use `windows-latest`.
- Trigger on `v*` tags and `workflow_dispatch`.
- Run fmt check, tests, check, release build.
- Stage artifact directory.
- Copy binaries, config, docs, and selected module examples.
- Generate SHA256 checksums.
- Zip artifact.
- Upload artifact.
- Create GitHub Release on tag.
DoD:
- Workflow exists and references correct repository paths.
- Workflow avoids shipping user-local `shortcuts.rmod` as active default.
- Workflow includes checksums.
- Workflow behavior for tag vs manual dispatch is documented or obvious.
- Workflow does not require new dependencies unless already available on `windows-latest` or standard GitHub Actions.
Validation:
- Review YAML syntax and paths locally.
- `cargo fmt`, `cargo test`, `cargo check`, `cargo build --release` pass locally.
- Full workflow execution remains GitHub-only.
Files likely touched:
- `.github/workflows/release.yml`
- `README.md`
- `STATE.md`
Risk: medium  
Depends on:
- T001
- T002
- T003
- T004
- T005
Notes:
- Use PowerShell steps for Windows packaging.
- Prefer official GitHub actions (`actions/checkout`, `actions/upload-artifact`) and `gh` or `softprops/action-gh-release` only if already acceptable without adding repo dependencies.

---

### T007 — Release docs cross-links and roadmap/state updates

Status: done  
Claimed by: current-agent  
Started: 2026-04-25 09:01  
Last update: 2026-04-25 09:04  
Scope:
- Update `README.md` with release/install/changelog links.
- Update `POST_FREEZE_ROADMAP.md` to reflect Wave 0 progress.
- Update `STATE.md` checkpoint.
- Ensure `CORE_FREEZE_V1.md` and release docs are discoverable.
DoD:
- Release docs are discoverable from README.
- Roadmap marks/notes Wave 0 work accurately.
- State has current checkpoint, validation, and next step.
Validation:
- Review docs.
Files likely touched:
- `README.md`
- `POST_FREEZE_ROADMAP.md`
- `STATE.md`
Risk: low  
Depends on:
- T001
- T002
- T003
- T004
- T005
- T006
Notes:
- This is the final documentation consolidation task.

---

### T008 — Final validation for Wave 0

Status: done  
Claimed by: current-agent  
Started: 2026-04-25 09:01  
Last update: 2026-04-25 09:04  
Scope:
- Run final validation commands.
- Record results.
- Identify any GitHub-only validation remaining.
DoD:
- `cargo fmt` passes.
- `cargo test` passes.
- `cargo check` passes.
- `cargo build --release` passes.
- `STATE.md` records final validation.
- `ACTIVE_QUEUE.md` records completed tasks or any remaining blockers.
Validation:
- `cargo fmt`
- `cargo test`
- `cargo check`
- `cargo build --release`
Files likely touched:
- `STATE.md`
- `ACTIVE_QUEUE.md`
Risk: medium  
Depends on:
- T007
Notes:
- Full GitHub Actions run can only be verified on GitHub.

---

## Execution recommendation

First executable task:

```text
T001 — Release checklist and artifact specification
```

Recommended command:

```text
/start-cont Ejecuta Wave 0 Packaging/release
```
