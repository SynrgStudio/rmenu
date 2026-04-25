# RELEASE CHECKLIST — rmenu

Status: active  
Applies to: Windows x64 zip releases  
Current release line: `0.2.x`

---

## 1. Release principles

`rmenu` core v1 is frozen. Release work must not add product behavior to the core.

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
- The script creates the GitHub Release directly with `gh`; the GitHub Actions workflow remains a reproducible fallback.
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
  - `target\release\rmenu-module-host.exe`

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
├── rmenu-module-host.exe
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
└── checksums.txt
```

Notes:

- The zip should not contain active external modules in `modules/` by default.
- Module examples should live under `module-examples/` in the release artifact.
- Users can copy selected examples into a `modules/` directory next to `rmenu.exe` when they want to enable them.
- `shortcuts.rmod` from a developer machine must not be shipped as an active default module because it may contain local app paths.
- Use `shortcuts.example.rmod` with generic targets such as `notepad.exe` and `wt.exe`.

---

## 5. Checksums

Generate SHA256 checksums for the zip and key binaries.

PowerShell example:

```powershell
Get-FileHash .\target\release\rmenu.exe -Algorithm SHA256
Get-FileHash .\target\release\rmenu-module-host.exe -Algorithm SHA256
Get-FileHash .\dist\rmenu-v<VERSION>-windows-x64.zip -Algorithm SHA256
```

The local release script and GitHub Actions workflow both generate checksums automatically.

---

## 6. GitHub Actions validation

The release workflow is validated in two stages:

1. Local validation:
   - docs reviewed;
   - commands and paths checked;
   - `cargo fmt`, `cargo test`, `cargo check`, `cargo build --release` pass.
2. GitHub validation:
   - workflow runs on `windows-latest`;
   - artifacts upload successfully;
   - tag releases attach zip and checksums correctly.

Full workflow validation can only be completed on GitHub.

---

## 7. Release publication checklist

Preferred publication path:

```powershell
.\scripts\release-local.ps1
```

The script handles commit, branch push, remote tag/release creation, and asset upload after confirmation.

Manual/GitHub Actions fallback:

- [ ] Ensure changelog entry is final.
- [ ] Commit and push release changes.
- [ ] Create and push a version tag:

```powershell
git tag v<VERSION>
git push origin v<VERSION>
```

- [ ] Confirm GitHub Actions release workflow completes.
- [ ] Confirm GitHub Release includes:
  - `rmenu-v<VERSION>-windows-x64.zip`;
  - checksum artifact or `checksums.txt`;
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
