param(
  [string]$Version,
  [switch]$PackageOnly,
  [switch]$SkipValidation,
  [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Step([string]$Message) {
  Write-Host ""
  Write-Host "==> $Message" -ForegroundColor Cyan
}

function Write-Info([string]$Message) {
  Write-Host "    $Message" -ForegroundColor DarkGray
}

function Fail([string]$Message) {
  throw $Message
}

function Invoke-External([string]$Command, [string[]]$Arguments) {
  Write-Info ("$Command " + ($Arguments -join " "))
  if ($DryRun) { return }
  & $Command @Arguments
  if ($LASTEXITCODE -ne 0) {
    Fail "Command failed with exit code ${LASTEXITCODE}: $Command $($Arguments -join ' ')"
  }
}

function Assert-Command([string]$Command) {
  if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) {
    Fail "Required command not found on PATH: $Command"
  }
}

function Assert-RepoRoot() {
  if (-not (Test-Path "Cargo.toml") -or -not (Test-Path ".git")) {
    Fail "Run this script from the rmenu repository root."
  }
}

function Get-CargoVersion() {
  $match = Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
  if (-not $match) { Fail "Could not find package version in Cargo.toml." }
  return $match.Matches[0].Groups[1].Value
}

function Set-CargoVersion([string]$TargetVersion) {
  $content = Get-Content "Cargo.toml" -Raw
  $next = [regex]::Replace($content, '(?m)^version\s*=\s*"[^"]+"', "version = `"$TargetVersion`"", 1)
  if ($next -eq $content) { Fail "Could not update Cargo.toml version." }
  if (-not $DryRun) {
    Set-Content -Path "Cargo.toml" -Value $next -NoNewline
  }
}

function Read-TargetVersion([string]$CurrentVersion) {
  Write-Host "Current Cargo.toml version: $CurrentVersion"
  if ($Version) {
    $target = $Version.Trim()
    Write-Host "Target release version: $target"
  } else {
    $inputValue = Read-Host "Target release version [$CurrentVersion]"
    $target = $inputValue.Trim()
    if (-not $target) { $target = $CurrentVersion }
  }

  if ($target.StartsWith("v")) {
    Fail "Enter the raw version without 'v' prefix. Example: 0.2.1"
  }
  if ($target -notmatch '^\d+\.\d+\.\d+([-.+][0-9A-Za-z.-]+)?$') {
    Fail "Invalid version '$target'. Expected semver-like value, for example 0.2.1."
  }
  return $target
}

function Read-MultilineCommitMessage() {
  Write-Host ""
  Write-Host "Commit message. End with a single line containing only END:" -ForegroundColor Yellow
  $lines = New-Object System.Collections.Generic.List[string]
  while ($true) {
    $line = Read-Host
    if ($line -eq "END") { break }
    $lines.Add($line)
  }
  $message = ($lines -join "`n").Trim()
  if (-not $message) { Fail "Commit message cannot be empty." }
  return $message
}

function Confirm-Yes([string]$Prompt) {
  $answer = Read-Host "$Prompt [y/N]"
  return $answer.Trim().ToLowerInvariant() -in @("y", "yes")
}

function Assert-GitHubCli() {
  Assert-Command "gh"
  Invoke-External "gh" @("auth", "status")
}

function Get-CurrentBranch() {
  $branch = (& git rev-parse --abbrev-ref HEAD).Trim()
  if ($LASTEXITCODE -ne 0 -or -not $branch) { Fail "Could not determine current git branch." }
  return $branch
}

function Assert-TagAvailable([string]$TagName) {
  $localTag = (@(& git tag --list $TagName) -join "").Trim()
  if ($localTag) { Fail "Local tag already exists: $TagName" }

  $remoteTag = (@(& git ls-remote --tags origin $TagName) -join "").Trim()
  if ($remoteTag) { Fail "Remote tag already exists on origin: $TagName" }

  if (-not $PackageOnly) {
    & gh release view $TagName *> $null
    if ($LASTEXITCODE -eq 0) { Fail "GitHub Release already exists: $TagName" }
    $global:LASTEXITCODE = 0
  }
}

function Invoke-Validation() {
  if ($SkipValidation) {
    Write-Step "Skipping validation by request"
    return
  }

  Write-Step "Running validation"
  Invoke-External "cargo" @("fmt")
  Invoke-External "cargo" @("test")
  Invoke-External "cargo" @("check")
  Invoke-External "cargo" @("build", "--release")
}

function Copy-IfExists([string]$Source, [string]$Destination) {
  if (Test-Path $Source) {
    Copy-Item $Source $Destination -Force
  }
}

function Get-Sha256File([string]$Path) {
  $stream = [System.IO.File]::OpenRead((Resolve-Path $Path))
  try {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
      $hash = $sha.ComputeHash($stream)
      return (($hash | ForEach-Object { $_.ToString("x2") }) -join "").ToUpperInvariant()
    } finally {
      $sha.Dispose()
    }
  } finally {
    $stream.Dispose()
  }
}

function New-ReleasePackage([string]$TargetVersion) {
  Write-Step "Creating release package"
  $packageName = "rmenu-v$TargetVersion-windows-x64"
  $dist = "dist"
  $stage = Join-Path $dist $packageName
  $zipPath = Join-Path $dist "$packageName.zip"
  $releaseSums = Join-Path $dist "SHA256SUMS.txt"

  if (-not $DryRun) {
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
    if (Test-Path $releaseSums) { Remove-Item $releaseSums -Force }

    New-Item -ItemType Directory -Force $stage | Out-Null
    New-Item -ItemType Directory -Force (Join-Path $stage "module-examples") | Out-Null

    Copy-Item "target\release\rmenu.exe" $stage -Force
    Copy-Item "target\release\rmenu-module-host.exe" $stage -Force
    Copy-Item "config_example.ini" $stage -Force

    $docs = @(
      "README.md",
      "INSTALL.md",
      "CHANGELOG.md",
      "CORE_FREEZE_V1.md",
      "MODULES_QUICKSTART.md",
      "MODULES_AUTHORING_GUIDE.md",
      "MODULES_OPERATIONS_GUIDE.md",
      "MODULES_API_SPEC_V1.md",
      "RMOD_SPEC_V1.md",
      "MANIFEST_SPEC_V1.md",
      "CTX_ACTIONS_SPEC_V1.md",
      "PROVIDER_EXECUTION_POLICY.md",
      "ERROR_ISOLATION_POLICY.md",
      "MODULES_CAPABILITIES_MATRIX.md"
    )

    foreach ($doc in $docs) {
      Copy-IfExists $doc $stage
    }

    Copy-IfExists "modules\calculator.rmod" (Join-Path $stage "module-examples\calculator.rmod")
    Copy-IfExists "modules\local-scripts.rmod" (Join-Path $stage "module-examples\local-scripts.rmod")
    Copy-IfExists "modules\examples\shortcuts.example.rmod" (Join-Path $stage "module-examples\shortcuts.example.rmod")

    $checksumPath = Join-Path $stage "checksums.txt"
    Get-ChildItem $stage -File -Recurse |
      Where-Object { $_.Name -ne "checksums.txt" } |
      Sort-Object FullName |
      ForEach-Object {
        $hash = Get-Sha256File $_.FullName
        $relative = Resolve-Path -Relative $_.FullName
        "$hash  $relative"
      } | Set-Content -Path $checksumPath -Encoding utf8

    Compress-Archive -Path $stage -DestinationPath $zipPath
    $zipHash = Get-Sha256File $zipPath
    "$zipHash  $packageName.zip" | Set-Content -Path $releaseSums -Encoding utf8

    $notesPath = Join-Path $dist "RELEASE_NOTES-v$TargetVersion.md"
    @(
      "# rmenu v$TargetVersion",
      "",
      "See CHANGELOG.md in the release artifact for details.",
      "",
      "Artifacts:",
      "- $packageName.zip",
      "- SHA256SUMS.txt"
    ) | Set-Content -Path $notesPath -Encoding utf8
  }

  return [pscustomobject]@{
    PackageName = $packageName
    Stage = $stage
    ZipPath = $zipPath
    ChecksumPath = $releaseSums
    NotesPath = (Join-Path $dist "RELEASE_NOTES-v$TargetVersion.md")
  }
}

function Get-ReleasableChanges() {
  $raw = git status --porcelain=v1
  $paths = New-Object System.Collections.Generic.List[string]

  foreach ($line in $raw) {
    if (-not $line.Trim()) { continue }
    $path = $line.Substring(3).Trim()
    if ($path.Contains(" -> ")) {
      $path = ($path -split " -> ")[-1]
    }
    $normalized = $path.Replace("\", "/")
    if ($normalized.StartsWith("dist/") -or $normalized.StartsWith("target/")) { continue }
    $paths.Add($path)
  }

  return $paths | Sort-Object -Unique
}

function Stage-ExactPaths([string[]]$Paths) {
  if (-not $Paths -or $Paths.Count -eq 0) { return }
  Write-Step "Staging release files"
  foreach ($path in $Paths) {
    Write-Info "git add -- $path"
    if (-not $DryRun) {
      & git add -- $path
      if ($LASTEXITCODE -ne 0) { Fail "Failed to stage path: $path" }
    }
  }
}

function New-ReleaseCommit([string]$Message) {
  Write-Step "Creating release commit"
  $tempFile = [System.IO.Path]::GetTempFileName()
  try {
    Set-Content -Path $tempFile -Value $Message -NoNewline
    Invoke-External "git" @("commit", "--file", $tempFile)
  } finally {
    Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
  }
}

function Publish-Release([string]$TargetVersion, [object]$Package) {
  $tag = "v$TargetVersion"
  $branch = Get-CurrentBranch

  Write-Step "Publishing branch"
  Invoke-External "git" @("push", "origin", $branch)

  $targetSha = (& git rev-parse HEAD).Trim()
  if (-not $targetSha) { Fail "Could not determine release commit SHA." }

  Write-Step "Creating GitHub Release $tag"
  Invoke-External "gh" @(
    "release", "create", $tag,
    $Package.ZipPath,
    $Package.ChecksumPath,
    "--target", $targetSha,
    "--title", "rmenu $tag",
    "--notes-file", $Package.NotesPath
  )

  Write-Step "Fetching tags"
  Invoke-External "git" @("fetch", "--tags", "origin")

  Write-Host ""
  Write-Host "Release published:" -ForegroundColor Green
  Write-Host "https://github.com/SynrgStudio/rmenu/releases/tag/$tag"
}

Assert-RepoRoot
Assert-Command "git"
Assert-Command "cargo"

$currentVersion = Get-CargoVersion
$targetVersion = Read-TargetVersion $currentVersion
$tagName = "v$targetVersion"

Write-Step "Release target"
Write-Info "Current version: $currentVersion"
Write-Info "Target version:  $targetVersion"
Write-Info "Tag:             $tagName"

if (-not $PackageOnly) {
  Assert-GitHubCli
}

Assert-TagAvailable $tagName

if ($targetVersion -ne $currentVersion) {
  Write-Step "Updating Cargo.toml version"
  Write-Info "$currentVersion -> $targetVersion"
  Set-CargoVersion $targetVersion
}

$commitMessage = $null
if (-not $PackageOnly) {
  $commitMessage = Read-MultilineCommitMessage
}

Invoke-Validation
$package = New-ReleasePackage $targetVersion

Write-Step "Package created"
Write-Info $package.ZipPath
Write-Info $package.ChecksumPath

if ($PackageOnly) {
  Write-Host ""
  Write-Host "Package-only mode complete. No commit, tag, push, or GitHub Release was created." -ForegroundColor Green
  exit 0
}

$paths = @(Get-ReleasableChanges)
if ($paths.Count -eq 0) {
  Write-Host "No releasable worktree changes to commit." -ForegroundColor Yellow
  if (-not (Confirm-Yes "Continue by publishing the current HEAD")) {
    Fail "Release cancelled."
  }
} else {
  Write-Step "Files that will be staged"
  $paths | ForEach-Object { Write-Host "  $_" }
  if (-not (Confirm-Yes "Stage these exact files and continue")) {
    Fail "Release cancelled."
  }
  Stage-ExactPaths $paths

  Write-Step "Staged files"
  git diff --cached --name-only

  if (-not (Confirm-Yes "Commit, push, create tag/release $tagName")) {
    Fail "Release cancelled."
  }
  New-ReleaseCommit $commitMessage
}

Publish-Release $targetVersion $package
