param(
  [switch]$Force,
  [switch]$SkipBuild,
  [switch]$DryRun,
  [string]$Version,
  [string]$DataRoot = "C:\rMenuData",
  [string]$InnoSetupPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$appKey = "rmenu"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$issPath = Join-Path $PSScriptRoot "rmenu.iss"
$distDir = Join-Path $repoRoot "dist\installers"

function Fail([string]$Message) { throw $Message }
function Write-Step([string]$Message) { Write-Host "==> $Message" -ForegroundColor Cyan }
function Write-Info([string]$Message) { Write-Host "    $Message" -ForegroundColor DarkGray }

function Get-CargoVersion() {
  $cargoToml = Join-Path $repoRoot "Cargo.toml"
  $match = Select-String -Path $cargoToml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
  if (-not $match) { Fail "Could not find package version in Cargo.toml." }
  return $match.Matches[0].Groups[1].Value
}

function Find-InnoSetup([string]$ConfiguredPath) {
  if ($ConfiguredPath) {
    if (Test-Path $ConfiguredPath) { return (Resolve-Path $ConfiguredPath).Path }
    Fail "Configured Inno Setup compiler not found: $ConfiguredPath"
  }

  $cmd = Get-Command "iscc" -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }

  $candidates = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 5\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 5\ISCC.exe"
  )

  foreach ($candidate in $candidates) {
    if ($candidate -and (Test-Path $candidate)) { return $candidate }
  }

  Fail "Inno Setup compiler not found. Install Inno Setup 6 or pass -InnoSetupPath 'C:\Path\ISCC.exe'."
}

function Invoke-External([string]$Command, [string[]]$Arguments) {
  Write-Info ("$Command " + ($Arguments -join " "))
  if ($DryRun) { return }
  & $Command @Arguments
  if ($LASTEXITCODE -ne 0) {
    Fail "Command failed with exit code ${LASTEXITCODE}: $Command $($Arguments -join ' ')"
  }
}

Push-Location $repoRoot
try {
  $cargoVersion = Get-CargoVersion
  $versionValue = if ($Version) { $Version.Trim().TrimStart("v") } else { $cargoVersion }
  if ($versionValue -notmatch '^\d+\.\d+\.\d+([-.+][0-9A-Za-z.-]+)?$') {
    Fail "Invalid version '$versionValue'. Expected semver-like value, for example 0.3.0."
  }
  if (-not $DataRoot.Trim()) { Fail "DataRoot cannot be empty." }

  $installerName = "$appKey-setup-v$versionValue.exe"
  $installerPath = Join-Path $distDir $installerName

  Write-Step "Installer target"
  Write-Info "App: $appKey"
  Write-Info "Cargo version: $cargoVersion"
  Write-Info "Installer version: $versionValue"
  Write-Info "Data root: $DataRoot"
  Write-Info "Output: $installerPath"

  if ((Test-Path $installerPath) -and -not $Force) {
    Write-Host "Installer already exists. Skipping. Use -Force to rebuild." -ForegroundColor Yellow
    Write-Output "RINSTALLER_RESULT|$appKey|skipped|$versionValue|$installerPath"
    return
  }

  if (-not $SkipBuild) {
    Write-Step "Building Rust release binaries"
    Invoke-External "cargo" @("build", "--release")
  } else {
    Write-Step "Skipping Rust build by request"
  }

  foreach ($binary in @("rmenu.exe", "rmenu-daemon.exe", "rmenu-module-host.exe")) {
    $binaryPath = Join-Path $repoRoot "target\release\$binary"
    if (-not (Test-Path $binaryPath)) { Fail "Missing target\release\$binary" }
  }

  if (-not $DryRun) { New-Item -ItemType Directory -Force $distDir | Out-Null }
  if ($DryRun) {
    $iscc = if ($InnoSetupPath) { $InnoSetupPath } else { "ISCC.exe" }
  } else {
    $iscc = Find-InnoSetup $InnoSetupPath
  }

  Write-Step "Compiling Inno Setup installer"
  $previousRepoRoot = $env:RINSTALLER_REPO_ROOT
  $previousVersion = $env:RINSTALLER_APP_VERSION
  $previousDataRoot = $env:RINSTALLER_DATA_ROOT
  try {
    $env:RINSTALLER_REPO_ROOT = $repoRoot
    $env:RINSTALLER_APP_VERSION = $versionValue
    $env:RINSTALLER_DATA_ROOT = $DataRoot
    Invoke-External $iscc @("/Qp", $issPath)
  } finally {
    $env:RINSTALLER_REPO_ROOT = $previousRepoRoot
    $env:RINSTALLER_APP_VERSION = $previousVersion
    $env:RINSTALLER_DATA_ROOT = $previousDataRoot
  }

  if (-not $DryRun -and -not (Test-Path $installerPath)) { Fail "Installer was not created: $installerPath" }
  Write-Output "RINSTALLER_RESULT|$appKey|built|$versionValue|$installerPath"
} finally {
  Pop-Location
}
