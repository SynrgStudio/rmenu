param(
  [Parameter(Mandatory=$true)][string]$Command,
  [Parameter(Mandatory=$true)][string]$StateDir,
  [int]$Seconds = 0,
  [string]$Label = "Timer",
  [string]$AlarmPath = ""
)

$ErrorActionPreference = "SilentlyContinue"
New-Item -ItemType Directory -Force -Path $StateDir | Out-Null
$StatePath = Join-Path $StateDir "state.json"
$StopPath = Join-Path $StateDir "stop.flag"

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class RMenuTimerNative {
  [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
  public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
}
"@

function Test-RMenuOpen {
  return [RMenuTimerNative]::FindWindow("rmenu_class_layout", $null) -ne [IntPtr]::Zero
}

function Write-State($State, $SecondsValue, $LabelValue) {
  $deadlineDate = (Get-Date).ToUniversalTime().AddSeconds($SecondsValue)
  $deadline = $deadlineDate.ToString("o")
  $deadlineEpochMs = ([DateTimeOffset]$deadlineDate).ToUnixTimeMilliseconds()
  [pscustomobject]@{
    state = $State
    label = $LabelValue
    seconds = $SecondsValue
    deadline = $deadline
    deadline_epoch_ms = $deadlineEpochMs
    pid = $PID
    updated_at = (Get-Date).ToUniversalTime().ToString("o")
  } | ConvertTo-Json -Compress | Set-Content -Path $StatePath -Encoding UTF8
}

function Stop-Timer {
  New-Item -ItemType File -Force -Path $StopPath | Out-Null
  [pscustomobject]@{
    state = "stopped"
    pid = $PID
    updated_at = (Get-Date).ToUniversalTime().ToString("o")
  } | ConvertTo-Json -Compress | Set-Content -Path $StatePath -Encoding UTF8
}

if ($Command -eq "stop") {
  Stop-Timer
  exit 0
}

if ($Command -ne "start" -or $Seconds -le 0) {
  exit 1
}

Remove-Item -Force $StopPath
Write-State "running" $Seconds $Label

$deadline = (Get-Date).AddSeconds($Seconds)
while ((Get-Date) -lt $deadline) {
  if (Test-Path $StopPath) {
    Stop-Timer
    exit 0
  }
  Start-Sleep -Milliseconds 250
}

Write-State "ringing" 0 $Label
$player = $null
if ($AlarmPath -and (Test-Path $AlarmPath)) {
  try {
    $player = New-Object System.Media.SoundPlayer $AlarmPath
    $player.PlayLooping()
  } catch {
    $player = $null
  }
}

while ($true) {
  if ((Test-Path $StopPath) -or (Test-RMenuOpen)) {
    if ($player) { $player.Stop(); $player.Dispose() }
    Stop-Timer
    exit 0
  }

  if (-not $player) {
    [console]::Beep(880, 220)
    Start-Sleep -Milliseconds 120
    if ((Test-Path $StopPath) -or (Test-RMenuOpen)) {
      Stop-Timer
      exit 0
    }
    [console]::Beep(660, 220)
  }

  Start-Sleep -Milliseconds 650
}
