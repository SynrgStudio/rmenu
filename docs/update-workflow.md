# rMenu update workflow

Status: implemented foundation / manual validation pending  
Applies to: rMenu `0.3.x` updater flow

## Goals

- Notify the user when a newer GitHub Release is available.
- Keep the prompt non-intrusive and dismissible for the current rMenu open.
- Let the user view release notes before installing.
- Install only after explicit user action.
- Verify SHA256 before running any installer.

## Source of truth

The updater uses GitHub Releases for:

```text
https://api.github.com/repos/SynrgStudio/rmenu/releases/latest
```

Expected release assets:

```text
rmenu-v<VERSION>-windows-x64.zip
rmenu-setup-v<VERSION>.exe
SHA256SUMS.txt
```

`SHA256SUMS.txt` must contain entries for every installable asset used by the updater.

## Cache

Update metadata is cached under the data root:

```text
<data_dir>\state\updates.json
```

Default Windows path:

```text
C:\rMenuData\state\updates.json
```

The cache stores at least:

```json
{
  "last_checked": "2026-05-07T00:00:00Z",
  "latest_version": "0.3.1",
  "release_url": "https://github.com/SynrgStudio/rmenu/releases/tag/v0.3.1",
  "installer_asset_url": "https://github.com/.../rmenu-setup-v0.3.1.exe",
  "checksums_asset_url": "https://github.com/.../SHA256SUMS.txt"
}
```

A normal launcher open reads cached state only. Network refresh happens through the daemon/update-check path or an explicit forced check, so opening rMenu does not block on GitHub.

## Startup prompt UX

When cached metadata says a newer version is available, rMenu opens with a startup notice instead of immediately showing normal search results:

```text
Update available: rMenu v0.3.1
Enter install now    Ctrl+Enter view changelog    Any key continue
```

Key behavior:

- `Enter`: launch `rmenu-updater.exe` with cached update metadata and close the current rMenu UI.
- `Ctrl+Enter`: open the GitHub Release URL in the browser; do not install.
- any other key: dismiss the notice for this rMenu open only and continue normal input handling.

If the user dismisses the notice, it may appear again on the next rMenu open if the update is still available. No persistent dismissal is planned for MVP.

## Installer flow

The installer flow runs in a separate process:

```text
rmenu-updater.exe
```

The updater process:

1. Downloads the installer asset into:

   ```text
   <data_dir>\state\updates\downloads\
   ```

2. Downloads `SHA256SUMS.txt`.
3. Verifies the installer SHA256.
4. Requests daemon shutdown:

   ```powershell
   rmenu-daemon.exe --quit
   ```

5. Waits briefly for rMenu/daemon processes to exit where needed.
6. Runs the verified installer with `/NORESTART`.
7. Attempts to restart the daemon after installation when the updated binaries are available.

Logs are appended to:

```text
<data_dir>\state\updates\updater.log
```

Downloaded update assets are kept under:

```text
<data_dir>\state\updates\downloads\
```

The updater also supports `file://` installer/checksum URLs and `--dry-run` for fixture validation without executing a real installer.

## Security model

- No automatic update in MVP.
- No install without explicit `Enter` from the startup prompt or an explicit update command.
- No installer execution unless SHA256 matches `SHA256SUMS.txt` from the same release.
- SHA256 protects integrity, not publisher identity.
- Authenticode signing is a future hardening task.

## Failure behavior

Failures must be recoverable:

- update check failure: log/cache error, open rMenu normally;
- missing installer asset: show feedback, do not install;
- missing checksum asset: abort install;
- SHA256 mismatch: abort install and report error;
- installer failure: log output/status and leave current installation untouched where possible.

## Troubleshooting

### Prompt appears but `Enter` fails

Check that `rmenu-updater.exe` exists next to `rmenu.exe`. Portable and installer releases should include it. Missing updater is reported as recoverable rMenu feedback.

### Download fails

Check internet access, GitHub Releases availability, and the cached URLs in:

```text
<data_dir>\state\updates.json
```

Then retry after reopening rMenu or manually installing the release from GitHub.

### SHA256 mismatch

Do not run the downloaded installer manually. Delete:

```text
<data_dir>\state\updates\downloads\
```

Then retry. If the mismatch persists, treat the release as invalid and install manually only after verifying the published checksums.

### Installer starts but upgrade does not complete

Review:

```text
<data_dir>\state\updates\updater.log
```

Then download the installer from the GitHub Release page and run it manually. The data folder is preserved by design.

## Manual validation

End-to-end validation requires at least two release versions or a controlled fixture:

1. install older release;
2. populate or fetch update cache for newer release;
3. open rMenu and confirm prompt appears;
4. press any key and confirm normal use continues;
5. reopen and press `Ctrl+Enter`, confirming release notes open;
6. reopen and press `Enter`, confirming download, verification, daemon shutdown, installer run, and version update.
