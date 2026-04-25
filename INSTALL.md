# INSTALL — rmenu

Status: active  
Install method for current release line: Windows x64 zip

---

## 1. Requirements

- Windows 10 or Windows 11.
- A terminal such as PowerShell.
- Optional: a folder on your `PATH` for launching `rmenu.exe` from anywhere.

`rmenu` is currently distributed as an unsigned zip artifact with SHA256 checksums.

---

## 2. Download

Download the latest Windows x64 zip from:

```text
https://github.com/SynrgStudio/rmenu/releases
```

Expected artifact name:

```text
rmenu-v<VERSION>-windows-x64.zip
```

---

## 3. Verify checksum

If the release provides `checksums.txt`, verify the downloaded zip before extracting it.

PowerShell example:

```powershell
Get-FileHash .\rmenu-v<VERSION>-windows-x64.zip -Algorithm SHA256
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
%LOCALAPPDATA%\rmenu\rmenu-v<VERSION>-windows-x64\rmenu-module-host.exe
```

You may also extract to any folder you control, for example:

```text
C:\Tools\rmenu
```

---

## 5. Add to PATH

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

## 6. Configuration

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

## 7. Modules

`rmenu` loads external modules from a `modules/` directory relative to the current working directory.

The release artifact may include examples under:

```text
module-examples/
```

Examples are not active by default. To enable one, copy it into a `modules/` directory next to where you run `rmenu.exe`, or into the working directory from which you launch `rmenu`.

Example:

```powershell
New-Item -ItemType Directory -Force .\modules | Out-Null
Copy-Item .\module-examples\calculator.rmod .\modules\calculator.rmod
rmenu.exe --modules-debug
```

Shortcut examples are intentionally packaged as examples, not active defaults, because real shortcuts usually contain machine-specific application paths.

---

## 8. Manual update

There is no auto-updater in the current release line.

To update manually:

1. Download the newer zip.
2. Verify checksum.
3. Extract it to a new versioned folder.
4. Update your `PATH` if needed.
5. Copy any user modules/config you intentionally keep.
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

## 9. Future install options

Planned or possible future distribution channels:

- Scoop manifest;
- winget package;
- PowerShell install script.

Not planned for Wave 0:

- MSI installer;
- automatic updater.

Those can be reconsidered after the zip release flow is reliable.
