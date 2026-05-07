# INSTALL — rmenu

Status: active  
Install methods for current release line: Windows x64 zip and Windows installer

---

## 1. Requirements

- Windows 10 or Windows 11.
- A terminal such as PowerShell.
- Optional: a folder on your `PATH` for launching `rmenu.exe` from anywhere.

`rmenu` is currently distributed as unsigned Windows artifacts with SHA256 checksums.

---

## 2. Download

Download the latest Windows x64 zip from:

```text
https://github.com/SynrgStudio/rmenu/releases
```

Expected artifact names:

```text
rmenu-v<VERSION>-windows-x64.zip
rmenu-setup-v<VERSION>.exe
```

---

## 3. Verify checksum

If the release provides `SHA256SUMS.txt`, verify the downloaded zip or installer before using it.

PowerShell example:

```powershell
Get-FileHash .\rmenu-v<VERSION>-windows-x64.zip -Algorithm SHA256
Get-FileHash .\rmenu-setup-v<VERSION>.exe -Algorithm SHA256
```

Compare the hash with the published checksum.

Important:

- SHA256 checksums verify file integrity.
- Checksums do not prove publisher identity.
- See `docs/release/BINARY_SIGNING.md` for the current signing/trust model.

---

## 4. Install from zip

Create an install directory and extract the zip.

Example:

```powershell
New-Item -ItemType Directory -Force "$env:LOCALAPPDATA\rmenu" | Out-Null
Expand-Archive .\rmenu-v<VERSION>-windows-x64.zip "$env:LOCALAPPDATA\rmenu" -Force
```

Depending on the zip layout, binaries should be available under a versioned folder, for example:

```text
%LOCALAPPDATA%\rmenu\rmenu-v<VERSION>-windows-x64\rmenu.exe
%LOCALAPPDATA%\rmenu\rmenu-v<VERSION>-windows-x64\rmenu-daemon.exe
%LOCALAPPDATA%\rmenu\rmenu-v<VERSION>-windows-x64\rmenu-module-host.exe
```

You may also extract to any folder you control, for example:

```text
C:\Tools\rmenu
```

---

## 5. Install with Windows installer

When available, `rmenu-setup-v<VERSION>.exe` installs the binaries under:

```text
C:\Program Files\rMenu
```

The installer asks where to store the reusable data root. The default is:

```text
C:\rMenuData
```

The chosen data folder stores modules, companions, config, and state. It is saved for future upgrades and is intentionally preserved on uninstall.

The installer creates one user-facing Start Menu shortcut: `rMenu`. That shortcut talks to the daemon and opens rMenu without exposing helper executables. `Start rMenu daemon when Windows starts` is enabled by default because rMenu is intended to be a resident launcher. The installer can also launch the daemon after install. When the daemon is running, rMenu shows a system tray icon; double-click opens rMenu, right-click opens a menu with Open and Quit. Uninstall removes app binaries and startup registration, but intentionally preserves the selected data root so installed modules, companions, config, and state can be reused by future installs.

---

## 6. Add to PATH

To run `rmenu.exe` from anywhere, add the extracted folder containing `rmenu.exe` to your user `PATH`.

PowerShell example for a versioned install folder:

```powershell
$RmenuDir = "$env:LOCALAPPDATA\rmenu\rmenu-v<VERSION>-windows-x64"
[Environment]::SetEnvironmentVariable(
  "Path",
  [Environment]::GetEnvironmentVariable("Path", "User") + ";" + $RmenuDir,
  "User"
)
```

Open a new terminal and verify:

```powershell
rmenu.exe --metrics
```

---

## 7. Configuration

Default config path:

```text
%APPDATA%\rmenu\config.ini
```

If the file is missing, `rmenu` creates one from defaults.

The release zip includes:

```text
config_example.ini
```

Use it as a reference for colors, layout, launcher sources, behavior, and module policy.

---

## 8. Modules and data root

The default Windows data root is:

```text
C:\rMenuData
```

By default, installed modules live under:

```text
C:\rMenuData\modules
```

Module/user state lives under:

```text
C:\rMenuData\state
```

Use `/rmods` in rMenu to install registry packages into the data root. `/rmods` is the unified extension hub for:

- `rmod` single-file modules;
- `rpack` folder modules/helpers;
- `companion` native apps such as RSnip and RTasks.

Companions install under:

```text
C:\rMenuData\companions
```

The rMenu installer does not bundle RSnip or RTasks in this wave. Install/update them from `/rmods`; `/install rsnip` and `/install rtasks` remain compatibility commands. Existing `--modules-dir` and `RMENU_MODULES_DIR` overrides remain available for development and debugging.

The release artifact may include examples under:

```text
module-examples/
```

Examples are not active by default. To enable one manually, copy it into the data-root modules directory.

Example:

```powershell
New-Item -ItemType Directory -Force C:\rMenuData\modules | Out-Null
Copy-Item .\module-examples\calculator.rmod C:\rMenuData\modules\calculator.rmod
rmenu.exe --modules-debug
```

Shortcut examples are intentionally packaged as examples, not active defaults, because real shortcuts usually contain machine-specific application paths.

---

## 9. Updates

rMenu can show a non-intrusive startup update notice when cached GitHub Release metadata says a newer version is available:

- `Enter`: starts `rmenu-updater.exe` with the cached installer/checksum URLs;
- `Ctrl+Enter`: opens the GitHub Release changelog;
- any other key: dismisses the notice for the current rMenu open only.

The updater downloads the installer and `SHA256SUMS.txt` under `<data_dir>\state\updates\downloads\`, verifies SHA256, requests daemon shutdown, runs the verified installer, and logs to `<data_dir>\state\updates\updater.log`.

Manual update remains supported:

1. Download the newer zip.
2. Verify checksum.
3. Extract it to a new versioned folder.
4. Update your `PATH` if needed.
5. Keep using the existing `C:\rMenuData` data root; do not copy package files into it unless you are manually managing modules.
6. Run smoke checks:

```powershell
rmenu.exe --metrics
rmenu.exe --modules-debug
```

If you use modules, validate them with:

```powershell
rmenu.exe --modules-debug
```

---

## 10. Future install options

Planned or possible future distribution channels:

- Scoop manifest;
- winget package;
- PowerShell install script.

Not planned for Wave 0:

- MSI installer;
- fully automatic/background updates.

Those can be reconsidered after the installer and updater flow is reliable.
