# rmenu - PowerShell scripting examples

## Introduction

`rmenu` is designed to be a versatile tool that integrates easily into PowerShell scripts. By capturing `rmenu` standard output (`stdout`) and checking its exit code, scripts can obtain interactive user selections.

## Basic usage

1. **Run `rmenu.exe`**

   Run `rmenu.exe` like any other command. Ensure it is in your PATH or provide the full executable path.

2. **Capture output**

   The user's selection is written to `stdout`. You can capture it in a PowerShell variable.

   ```powershell
   $selection = rmenu.exe -p "Select:"
   ```

3. **Check the exit code (`$LASTEXITCODE`)**

   Immediately after `rmenu.exe` exits, `$LASTEXITCODE` contains:

   - `0`: the user selected an item with Enter.
   - `1`: the user cancelled with Escape, or the window closed without an explicit selection.

   ```powershell
   if ($LASTEXITCODE -eq 0) {
       Write-Host "Selected: $selection"
   } else {
       Write-Host "Selection cancelled"
   }
   ```

4. **Provide items**

   From a comma-separated string:

   ```powershell
   $selection = rmenu.exe -p "Choose:" -e "Option A,Option B,Option C"
   ```

   Or from standard input, one item per line:

   ```powershell
   $options = @("One", "Two", "Three")
   $selection = $options | rmenu.exe -p "Number:"
   ```

---

## Practical examples

### 1. Launch a selected command

```powershell
$commands = @("notepad.exe", "calc.exe", "mspaint.exe")
$selection = $commands | rmenu.exe -p "Run:"

if ($LASTEXITCODE -eq 0 -and $selection) {
    try {
        Start-Process $selection
    } catch {
        Write-Error "Could not launch '$selection'. Ensure it is installed and in PATH."
    }
} else {
    Write-Host "Selection cancelled."
}
```

### 2. Application launcher map

Lets the user select a friendly name and launches the mapped executable.

```powershell
$apps = @{
    "Notepad" = "notepad.exe"
    "Calculator" = "calc.exe"
    "Paint" = "mspaint.exe"
    "Command Prompt (CMD)" = "cmd.exe"
}

$appNames = $apps.Keys | Sort-Object
$selectedName = $appNames | rmenu.exe -p "Launch app:"

if ($LASTEXITCODE -eq 0 -and $selectedName) {
    $target = $apps[$selectedName]
    if ($target) {
        Start-Process $target
    } else {
        Write-Error "Internal error: app '$selectedName' was not found in the script map."
    }
}
```

### 3. File selector

Shows file names from the current directory and opens the selected file.

```powershell
$files = Get-ChildItem -File | Select-Object -ExpandProperty Name

if (-not $files) {
    Write-Host "No files found in the current directory."
    exit 0
}

$selectedFile = $files | rmenu.exe -p "Open file:"

if ($LASTEXITCODE -eq 0 -and $selectedFile) {
    Start-Process $selectedFile
} else {
    Write-Host "Selection cancelled."
}
```

### 4. Theme selector (conceptual)

A script that could change an application theme.

```powershell
$availableThemes = "Light,Dark,Metallic Blue,Forest Green"
$currentTheme = "Light"

$selectedTheme = rmenu.exe -p "Theme ($currentTheme):" -e $availableThemes

if ($LASTEXITCODE -eq 0 -and $selectedTheme) {
    Write-Host "Applying theme: $selectedTheme"
    # Example: Set-AppTheme -Name $selectedTheme
}
```

---

## Tips

- **Silent mode:** Use `rmenu.exe -s ...` when the script needs a clean operation and non-critical `rmenu` diagnostics should not appear on `stderr`.
- **Delimiter handling:** When passing items through `-e`, any item containing the configured delimiter will be split. If an item must contain a literal comma, change `element_delimiter` in `config.ini` and use the new delimiter.
- **Quoting and spaces:** PowerShell usually passes variables with spaces correctly to external executables.
- **Large lists:** `rmenu` is fast, but hundreds of thousands of options can affect performance, especially through `-e` vs `stdin`. Consider pre-filtering for extremely large lists.
- **Character encoding:** PowerShell and `rmenu` generally handle Unicode well. Ensure your terminal and scripts use the required encoding when working with non-ASCII characters.

These examples should be a useful starting point for integrating `rmenu` into your own PowerShell scripts.
