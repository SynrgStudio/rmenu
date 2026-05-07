# RELEASE CHECKLIST — rmenu

Status: active
Applies to: Windows x64 zip releases and installer preparation
Current release target: `0.3.0`

---

## 1. Release principles

`rmenu` core v1 is frozen. Release work must not add product behavior to the core unless it is already planned as a generic platform primitive.

The `0.3.0` release scope is the daemon/rmods/resident-helper wave: warm daemon launch, persistent data root, native companions, `/rmods`, `rpack`, resident helpers, validated registry rpacks, installer packaging, and updater foundation. Fully automatic/background updates remain out of scope.

A release is ready when:

- validation commands pass;
- release artifact contents are deterministic;
- module examples are packaged safely;
- checksums are generated;
- install/update docs are current;
- changelog/release notes are current;
- manual smoke checks are repeated or explicitly accepted from the latest validation pass.

---

## 2. Preferred maintainer release flow

Primary local release command:

```powershell
.\scripts\release-local.ps1
```

The script interactively:

1. reads and displays the current `Cargo.toml` version;
2. asks for the target release version;
3. updates `Cargo.toml` if the target version differs;
4. asks for a multiline commit message terminated by `END`;
5. runs validation;
6. creates the Windows x64 zip and SHA256 checksums;
7. stages exact changed files, excluding `dist/` and `target/`;
8. commits with the provided message;
9. pushes the current branch;
10. creates the GitHub Release and remote tag with `gh release create`;
11. fetches tags back from `origin`.

Package-only dry run for local artifact validation:

```powershell
.\scripts\release-local.ps1 -Version 0.2.0 -PackageOnly
```

Notes:

- The script does not use `git add .` or `git add -A`; it stages exact changed paths after showing them for confirmation.
- The script creates the GitHub Release directly with `gh`; the GitHub Actions workflow remains a manual reproducible fallback and is not triggered by release tags.
- Pass `-IncludeInstaller` to include `rmenu-setup-v<VERSION>.exe` and its SHA256 in the release assets.
- The script requires `gh auth login` before publishing.
- `dist/` artifacts are uploaded to the GitHub Release, not committed.

---

## 3. Pre-release checklist

### Repository state

- [ ] Confirm branch and worktree state:

```powershell
git status --short
```

- [ ] Confirm version in `Cargo.toml`.
- [ ] For `0.3.0`, leave the version bump to `scripts/release-local.ps1` or the final release task; do not bump during planning-only work.
- [ ] Confirm `CHANGELOG.md` has an entry for the release.
- [ ] Confirm `CORE_FREEZE_V1.md` still reflects the current core policy.
- [ ] Confirm `INSTALL.md` and `README.md` links are current.

### Automated validation

Run from the repository root:

```powershell
cargo fmt
cargo test
cargo check
cargo build --release
```

Expected:

- [ ] `cargo fmt` completes.
- [ ] `cargo test` passes.
- [ ] `cargo check` passes.
- [ ] `cargo build --release` produces:
  - `target\release\rmenu.exe`
  - `target\release\rmenu-daemon.exe`
  - `target\release\rmenu-module-host.exe`
  - `target\release\rmenu-updater.exe`

### Release diagnostics

Run from the repository root after release build:

```powershell
.\target\release\rmenu.exe --metrics
.\target\release\rmenu.exe --modules-debug
.\target\release\rmenu.exe --debug-ranking blender
.\target\release\rmenu.exe --reindex --metrics
```

Expected:

- [ ] metrics command exits successfully;
- [ ] modules debug command exits successfully;
- [ ] ranking debug command exits successfully;
- [ ] reindex metrics command exits successfully.

### Manual smoke checks

These flows have already been validated by the user during core closure and should be repeated before public release when practical:

- [ ] launcher opens;
- [ ] search works;
- [ ] Enter launches the selected item;
- [ ] Esc cancels/closes;
- [ ] stdin mode returns the selected item;
- [ ] running from repository root with modules works:
  - calculator module works;
  - local scripts scoped mode works;
  - shortcuts module works if configured;
- [ ] running from an empty working directory without external modules works;
- [ ] builtin-only `--modules-debug` reports `external_descriptors: 0` and `running_hosts: 0`.

Example stdin check:

```powershell
"uno`ndos`ntres" | .\target\release\rmenu.exe
```

---

## 4. Windows x64 artifact specification

Release zip name:

```text
rmenu-v<VERSION>-windows-x64.zip
```

Recommended artifact layout:

```text
rmenu-v<VERSION>-windows-x64/
├── rmenu.exe
├── rmenu-daemon.exe
├── rmenu-module-host.exe
├── rmenu-updater.exe
├── config_example.ini
├── README.md
├── INSTALL.md
├── CHANGELOG.md
├── CORE_FREEZE_V1.md
├── MODULES_QUICKSTART.md
├── MODULES_AUTHORING_GUIDE.md
├── MODULES_OPERATIONS_GUIDE.md
├── MODULES_API_SPEC_V1.md
├── RMOD_SPEC_V1.md
├── MANIFEST_SPEC_V1.md
├── CTX_ACTIONS_SPEC_V1.md
├── PROVIDER_EXECUTION_POLICY.md
├── ERROR_ISOLATION_POLICY.md
├── MODULES_CAPABILITIES_MATRIX.md
├── module-examples/
│   ├── calculator.rmod
│   ├── local-scripts.rmod
│   └── shortcuts.example.rmod
├── docs/
│   ├── companion-and-rmods-workflow.md
│   ├── rmods-registry.md
│   └── update-workflow.md
└── checksums.txt
```

Installer artifact name when enabled:

```text
rmenu-setup-v<VERSION>.exe
```

Installer expectations:

- installs app binaries under `C:\Program Files\rMenu` by default;
- asks for the reusable data folder, defaulting to `C:\rMenuData`;
- stores the chosen data folder in `HKCU\Software\SynrgStudio\rMenu\DataDir` for future upgrades;
- preserves the chosen data folder during install, upgrade, and uninstall;
- registers `rmenu-daemon.exe` at user startup by default, unless the user unchecks the task;
- launches `rmenu-daemon.exe` after install by default, unless the user unchecks the task;
- Start Menu shortcuts use the rMenu app icon;
- daemon shows a system tray icon with Open and Quit actions;
- installs `rmenu-updater.exe` next to `rmenu.exe`;
- removes app binaries/startup entry on uninstall without deleting the data root.

Notes:

- The zip should not contain active external modules in `modules/` by default.
- Module examples should live under `module-examples/` in the release artifact.
- Users can copy selected examples into a `modules/` directory next to `rmenu.exe` when they want to enable them.
- `shortcuts.rmod` from a developer machine must not be shipped as an active default module because it may contain local app paths.
- Use `shortcuts.example.rmod` with generic targets such as `notepad.exe` and `wt.exe`.

---

## 5. Installer artifact checks

When building the installer locally:

```powershell
.\installer\build-installer.ps1 -Version 0.3.0 -SkipBuild -Force
```

When validating all artifacts locally without publishing:

```powershell
.\scripts\release-local.ps1 -Version 0.3.0 -PackageOnly -SkipValidation -IncludeInstaller
```

Updater smoke validation:

```powershell
# Use file:// fixtures and --dry-run; do not execute a real installer during automated validation.
.\target\debug\rmenu-updater.exe install --version 9.9.9 --release-url https://example.test/release --installer-url file://C:/tmp/rmenu-setup-v9.9.9.exe --checksums-url file://C:/tmp/SHA256SUMS.txt --data-dir C:\Temp\rmenu-updater-smoke --dry-run
```

Expected `dist\SHA256SUMS.txt` entries:

```text
rmenu-v<VERSION>-windows-x64.zip
installers/rmenu-setup-v<VERSION>.exe
```

---

## 6. Checksums

Generate SHA256 checksums for the zip, installer, and key binaries.

PowerShell example:

```powershell
Get-FileHash .\target\release\rmenu.exe -Algorithm SHA256
Get-FileHash .\target\release\rmenu-daemon.exe -Algorithm SHA256
Get-FileHash .\target\release\rmenu-module-host.exe -Algorithm SHA256
Get-FileHash .\dist\rmenu-v<VERSION>-windows-x64.zip -Algorithm SHA256
Get-FileHash .\dist\installers\rmenu-setup-v<VERSION>.exe -Algorithm SHA256
```

The local release script and GitHub Actions workflow both generate checksums automatically.

---

## 7. GitHub Actions validation

The GitHub Actions workflow is a manual fallback only:

```text
GitHub -> Actions -> Release -> Run workflow
```

It is not triggered by release tags. The local release script is the publishing path and uploads the zip/checksums directly with `gh release create`.

GitHub validation checks:

- workflow runs on `windows-latest` when manually dispatched;
- artifacts upload successfully;
- generated zip and checksums match the documented artifact layout.

---

## 8. Release publication checklist

Preferred publication path:

```powershell
.\scripts\release-local.ps1
```

The script handles commit, branch push, remote tag/release creation, and asset upload after confirmation.

Manual fallback:

- [ ] Ensure changelog entry is final.
- [ ] Commit and push release changes.
- [ ] Run `.\scripts\release-local.ps1` to create the release and upload local assets.
- [ ] Optionally run GitHub Actions manually as a reproducibility check.
- [ ] Confirm GitHub Release includes:
  - `rmenu-v<VERSION>-windows-x64.zip`;
  - `rmenu-setup-v<VERSION>.exe` when installer is accepted;
  - `SHA256SUMS.txt`;
  - release notes.
- [ ] Download artifact from GitHub Release.
- [ ] Verify checksum.
- [ ] Smoke test downloaded artifact.

Do not tag or push unless explicitly intended by the maintainer.

---

## 8. Post-release checklist

- [ ] Confirm release page links work.
- [ ] Confirm install docs match artifact layout.
- [ ] Confirm no local-user module config or paths were shipped as active defaults.
- [ ] Record release result in `CHANGELOG.md` or follow-up notes if needed.
- [ ] Start next roadmap item only after release artifact is verified.
