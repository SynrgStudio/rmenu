param(
    [int]$MetricsRuns = 5,
    [string]$OutputPath = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..")
Set-Location $repoRoot

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $outDir = Join-Path $repoRoot "artifacts\audits"
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
    $OutputPath = Join-Path $outDir "audit-$timestamp.txt"
}

$report = [System.Text.StringBuilder]::new()
$checks = New-Object System.Collections.Generic.List[object]

function Add-Line {
    param([string]$Text = "")
    [void]$report.AppendLine($Text)
}

function Add-Section {
    param([string]$Title)
    Add-Line ""
    Add-Line "============================================================"
    Add-Line $Title
    Add-Line "============================================================"
}

function Add-Check {
    param(
        [string]$Name,
        [bool]$Passed,
        [string]$Detail
    )
    $checks.Add([pscustomobject]@{
        Name = $Name
        Passed = $Passed
        Detail = $Detail
    }) | Out-Null
}

function Convert-ToCommandArgumentString {
    param([string[]]$Arguments)

    $escaped = $Arguments | ForEach-Object {
        if ($_ -match '[\s"'']') {
            '"' + ($_ -replace '"', '\\"') + '"'
        } else {
            $_
        }
    }

    return [string]::Join(' ', $escaped)
}

function Invoke-CapturedCommand {
    param(
        [string]$Command,
        [string[]]$Arguments = @()
    )

    $output = @()
    $exitCode = 0
    try {
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = $Command
        $psi.Arguments = Convert-ToCommandArgumentString -Arguments $Arguments
        $psi.UseShellExecute = $false
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $true
        $psi.CreateNoWindow = $true
        $psi.WorkingDirectory = "$repoRoot"

        $process = New-Object System.Diagnostics.Process
        $process.StartInfo = $psi

        [void]$process.Start()
        $stdout = $process.StandardOutput.ReadToEnd()
        $stderr = $process.StandardError.ReadToEnd()
        $process.WaitForExit()

        $exitCode = [int]$process.ExitCode

        if (-not [string]::IsNullOrWhiteSpace($stdout)) {
            $output += $stdout -split "`r?`n"
        }
        if (-not [string]::IsNullOrWhiteSpace($stderr)) {
            $output += $stderr -split "`r?`n"
        }

        if ($output.Count -eq 0) {
            $output = @("<no output>")
        }
    } catch {
        $output = @($_.Exception.Message)
        $exitCode = 1
    }

    [pscustomobject]@{
        Command = "$Command $($Arguments -join ' ')".Trim()
        ExitCode = [int]$exitCode
        Output = ($output | ForEach-Object { "$_" })
    }
}

function Get-Percentile {
    param(
        [double[]]$Values,
        [double]$Percentile
    )
    if (-not $Values -or $Values.Count -eq 0) { return 0 }
    $sorted = @($Values | Sort-Object)
    $rank = [math]::Ceiling(($Percentile / 100.0) * $sorted.Count)
    $idx = [math]::Max(1, [math]::Min($rank, $sorted.Count)) - 1
    return [double]$sorted[$idx]
}

function Get-ResourceSnapshot {
    param([string]$Label)

    $cpuLoad = $null
    try {
        $cpuLoad = [double](Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
    } catch {
        $cpuLoad = -1
    }

    $os = Get-CimInstance Win32_OperatingSystem
    $totalMemMB = [math]::Round($os.TotalVisibleMemorySize / 1024, 2)
    $freeMemMB = [math]::Round($os.FreePhysicalMemory / 1024, 2)
    $usedMemMB = [math]::Round($totalMemMB - $freeMemMB, 2)

    $diskTime = -1
    $diskQueue = -1
    try {
        $diskTime = [math]::Round((Get-Counter '\PhysicalDisk(_Total)\% Disk Time').CounterSamples[0].CookedValue, 2)
    } catch {}
    try {
        $diskQueue = [math]::Round((Get-Counter '\PhysicalDisk(_Total)\Current Disk Queue Length').CounterSamples[0].CookedValue, 2)
    } catch {}

    $gpuUtil = -1
    try {
        $gpuCounters = (Get-Counter '\GPU Engine(*)\Utilization Percentage').CounterSamples |
            Where-Object { $_.InstanceName -match 'engtype_3D|engtype_Compute|engtype_Copy|engtype_Video' }
        if ($gpuCounters.Count -gt 0) {
            $gpuUtil = [math]::Round(($gpuCounters | Measure-Object -Property CookedValue -Sum).Sum, 2)
        }
    } catch {}

    $topCpu = Get-Process |
        ForEach-Object {
            $cpuValue = 0.0
            try { $cpuValue = [double]$_.CPU } catch { $cpuValue = 0.0 }
            [pscustomobject]@{
                Name = $_.Name
                Id = $_.Id
                CPU = [math]::Round($cpuValue, 3)
                WS_MB = [math]::Round($_.WorkingSet64 / 1MB, 1)
            }
        } |
        Sort-Object -Property CPU -Descending |
        Select-Object -First 5

    [pscustomobject]@{
        Label = $Label
        Timestamp = (Get-Date).ToString("s")
        ProcessCount = (Get-Process).Count
        CpuLoadPct = $cpuLoad
        MemoryUsedMB = $usedMemMB
        MemoryFreeMB = $freeMemMB
        DiskTimePct = $diskTime
        DiskQueue = $diskQueue
        GpuUtilPct = $gpuUtil
        TopCpu = $topCpu
    }
}

function Parse-MetricsBlock {
    param([string[]]$Lines)

    $map = @{}
    foreach ($line in $Lines) {
        if ($line -match '^\s*-\s*([a-zA-Z0-9_]+):\s*(.+)$') {
            $map[$matches[1]] = $matches[2].Trim()
        }
    }
    return $map
}

Add-Line "rmenu unified audit report"
Add-Line "Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Add-Line "Repo root: $repoRoot"
Add-Line "Metrics runs: $MetricsRuns"

Add-Section "Host hardware information"
try {
    $cs = Get-CimInstance Win32_ComputerSystem
    $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
    $gpus = Get-CimInstance Win32_VideoController
    $memModules = Get-CimInstance Win32_PhysicalMemory
    $disks = Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3"

    Add-Line "Machine: $($cs.Manufacturer) $($cs.Model)"
    Add-Line "CPU: $($cpu.Name.Trim())"
    Add-Line "CPU cores/logical: $($cpu.NumberOfCores)/$($cpu.NumberOfLogicalProcessors)"
    Add-Line "Installed RAM (GB): $([math]::Round(($cs.TotalPhysicalMemory / 1GB), 2))"

    Add-Line "GPU(s):"
    foreach ($gpu in $gpus) {
        Add-Line "  - $($gpu.Name) | Driver: $($gpu.DriverVersion)"
    }

    Add-Line "Physical memory modules:"
    foreach ($m in $memModules) {
        Add-Line "  - $([math]::Round($m.Capacity / 1GB, 2)) GB @ $($m.Speed) MHz"
    }

    Add-Line "Disk volumes:"
    foreach ($d in $disks) {
        $sizeGB = if ($d.Size) { [math]::Round($d.Size / 1GB, 2) } else { 0 }
        $freeGB = if ($d.FreeSpace) { [math]::Round($d.FreeSpace / 1GB, 2) } else { 0 }
        Add-Line "  - $($d.DeviceID) size=${sizeGB}GB free=${freeGB}GB FS=$($d.FileSystem)"
    }

    Add-Check -Name "Hardware info" -Passed $true -Detail "Collected"
} catch {
    Add-Line "Failed to collect hardware info: $($_.Exception.Message)"
    Add-Check -Name "Hardware info" -Passed $false -Detail $_.Exception.Message
}

$snapshots = New-Object System.Collections.Generic.List[object]
$snapshots.Add((Get-ResourceSnapshot -Label "start")) | Out-Null

Add-Section "Toolchain info"
$toolCommands = @(
    @{ Name = "rustc --version"; Cmd = "rustc"; Args = @("--version") },
    @{ Name = "cargo --version"; Cmd = "cargo"; Args = @("--version") }
)
foreach ($t in $toolCommands) {
    $result = Invoke-CapturedCommand -Command $t.Cmd -Arguments $t.Args
    Add-Line "> $($result.Command)"
    foreach ($line in $result.Output) { Add-Line $line }
    Add-Line "exit_code=$($result.ExitCode)"
    Add-Check -Name $t.Name -Passed ($result.ExitCode -eq 0) -Detail "exit=$($result.ExitCode)"
}

Add-Section "Build & test audits"
$buildCommands = @(
    @{ Name = "cargo check"; Cmd = "cargo"; Args = @("check") },
    @{ Name = "cargo test"; Cmd = "cargo"; Args = @("test") },
    @{ Name = "cargo build --release"; Cmd = "cargo"; Args = @("build", "--release") }
)
foreach ($c in $buildCommands) {
    $result = Invoke-CapturedCommand -Command $c.Cmd -Arguments $c.Args
    Add-Line "> $($result.Command)"
    foreach ($line in $result.Output) { Add-Line $line }
    Add-Line "exit_code=$($result.ExitCode)"
    Add-Line ""
    Add-Check -Name $c.Name -Passed ($result.ExitCode -eq 0) -Detail "exit=$($result.ExitCode)"
}

$snapshots.Add((Get-ResourceSnapshot -Label "post-build")) | Out-Null

$releaseExe = Join-Path $repoRoot "target\release\rmenu.exe"

Add-Section "CLI diagnostics audits"
$cliCommands = @(
    @{ Name = "rmenu --help"; Cmd = $releaseExe; Args = @("--help") },
    @{ Name = "rmenu --debug-ranking pow"; Cmd = $releaseExe; Args = @("--debug-ranking", "pow") },
    @{ Name = "rmenu --debug-ranking paint"; Cmd = $releaseExe; Args = @("--debug-ranking", "paint") },
    @{ Name = "rmenu --debug-ranking code"; Cmd = $releaseExe; Args = @("--debug-ranking", "code") }
)

foreach ($c in $cliCommands) {
    $result = Invoke-CapturedCommand -Command $c.Cmd -Arguments $c.Args
    Add-Line "> $($result.Command)"
    foreach ($line in $result.Output) { Add-Line $line }
    Add-Line "exit_code=$($result.ExitCode)"
    Add-Line ""
    Add-Check -Name $c.Name -Passed ($result.ExitCode -eq 0) -Detail "exit=$($result.ExitCode)"
}

Add-Section "Metrics audits"
$metricMaps = New-Object System.Collections.Generic.List[hashtable]
for ($i = 1; $i -le $MetricsRuns; $i++) {
    $result = Invoke-CapturedCommand -Command $releaseExe -Arguments @("--metrics")
    Add-Line "Run #$i"
    foreach ($line in $result.Output) { Add-Line $line }
    Add-Line "exit_code=$($result.ExitCode)"
    Add-Line ""

    if ($result.ExitCode -eq 0) {
        $metricMaps.Add((Parse-MetricsBlock -Lines $result.Output)) | Out-Null
    }

    Add-Check -Name "rmenu --metrics run #$i" -Passed ($result.ExitCode -eq 0) -Detail "exit=$($result.ExitCode)"
}

$numericKeys = @(
    "startup_prepare_ms",
    "time_to_window_visible_ms",
    "time_to_first_paint_ms",
    "time_to_input_ready_ms",
    "search_p95_ms",
    "dataset_items",
    "dataset_estimated_bytes",
    "index_cache_bytes"
)

if ($metricMaps.Count -gt 0) {
    Add-Line "Aggregated metrics"
    foreach ($key in $numericKeys) {
        $values = @()
        foreach ($m in $metricMaps) {
            if ($m.ContainsKey($key)) {
                $raw = $m[$key].Replace(',', '.')
                $num = 0.0
                if ([double]::TryParse($raw, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$num)) {
                    $values += $num
                }
            }
        }

        if ($values.Count -gt 0) {
            $min = ($values | Measure-Object -Minimum).Minimum
            $max = ($values | Measure-Object -Maximum).Maximum
            $avg = ($values | Measure-Object -Average).Average
            $p50 = Get-Percentile -Values $values -Percentile 50
            $p95 = Get-Percentile -Values $values -Percentile 95
            Add-Line ("- {0}: min={1} p50={2} avg={3:N3} p95={4} max={5}" -f $key, $min, $p50, $avg, $p95, $max)
        }
    }
}

$snapshots.Add((Get-ResourceSnapshot -Label "post-metrics")) | Out-Null

Add-Section "Index/cache audit"
$cachePath = Join-Path $env:APPDATA "rmenu\index.json"
Add-Line "cache_path=$cachePath"
if (Test-Path $cachePath) {
    try {
        $cacheRaw = Get-Content -Raw -Path $cachePath
        $cacheObj = $cacheRaw | ConvertFrom-Json
        $itemCount = if ($cacheObj.items) { $cacheObj.items.Count } else { 0 }
        Add-Line "cache_exists=true"
        Add-Line "cache_version=$($cacheObj.version)"
        Add-Line "cache_generated_at_unix_ms=$($cacheObj.generated_at_unix_ms)"
        Add-Line "cache_item_count=$itemCount"
        Add-Line "cache_has_env_signature=$([bool]($null -ne $cacheObj.env_signature))"
        Add-Check -Name "Index cache parse" -Passed $true -Detail "version=$($cacheObj.version), items=$itemCount"
    } catch {
        Add-Line "cache_exists=true"
        Add-Line "cache_parse_error=$($_.Exception.Message)"
        Add-Check -Name "Index cache parse" -Passed $false -Detail $_.Exception.Message
    }
} else {
    Add-Line "cache_exists=false"
    Add-Check -Name "Index cache parse" -Passed $false -Detail "index cache file not found"
}

$snapshots.Add((Get-ResourceSnapshot -Label "end")) | Out-Null

Add-Section "Resource snapshots during audit"
foreach ($s in $snapshots) {
    Add-Line "[$($s.Label)] $($s.Timestamp)"
    Add-Line "- process_count: $($s.ProcessCount)"
    Add-Line "- cpu_load_pct: $($s.CpuLoadPct)"
    Add-Line "- memory_used_mb: $($s.MemoryUsedMB)"
    Add-Line "- memory_free_mb: $($s.MemoryFreeMB)"
    Add-Line "- disk_time_pct: $($s.DiskTimePct)"
    Add-Line "- disk_queue: $($s.DiskQueue)"
    Add-Line "- gpu_util_pct: $($s.GpuUtilPct)"
    Add-Line "- top_cpu_processes:"
    foreach ($p in $s.TopCpu) {
        Add-Line "    - $($p.Name) (pid=$($p.Id), cpu=$($p.CPU), ws_mb=$($p.WS_MB))"
    }
    Add-Line ""
}

Add-Section "Audit summary"
$passedCount = @($checks | Where-Object { $_.Passed }).Count
$failed = @($checks | Where-Object { -not $_.Passed })
Add-Line "Total checks: $($checks.Count)"
Add-Line "Passed: $passedCount"
Add-Line "Failed: $($failed.Count)"

if ($failed.Count -gt 0) {
    Add-Line ""
    Add-Line "Failed checks:"
    foreach ($f in $failed) {
        Add-Line "- $($f.Name): $($f.Detail)"
    }
}

$reportText = $report.ToString()
Set-Content -Path $OutputPath -Value $reportText -Encoding UTF8
Write-Host "Audit report generated: $OutputPath"

if ($failed.Count -gt 0) {
    exit 1
}

exit 0
