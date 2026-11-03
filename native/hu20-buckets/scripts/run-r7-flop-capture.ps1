[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutputDirectory,
    [Parameter(Mandatory)] [string] $BaselineDirectory,
    [ValidateRange(1, 86400)] [int] $TimeoutSeconds = 300
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'process-supervision.ps1')
$project = Split-Path -Parent $PSScriptRoot
$testName = 'tests::test_only_ranks7_flop_k10_captures_step_thirty'
$output = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDirectory)
$baseline = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($BaselineDirectory)

if (Test-Path -LiteralPath $output) {
    if ((Get-ChildItem -LiteralPath $output -Force | Measure-Object).Count -ne 0) {
        throw "Output directory is not empty: $output"
    }
} else {
    New-Item -ItemType Directory -Path $output | Out-Null
}

function Get-VerifiedGitProvenance {
    $revision = (git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $revision -notmatch '^[0-9a-fA-F]{40}$') {
        throw 'Unable to verify Git revision.'
    }
    $status = git status --porcelain
    if ($LASTEXITCODE -ne 0 -or $status) {
        throw 'Refusing to run from a dirty working tree.'
    }
    return $revision.ToLowerInvariant()
}

$summaryPath = Join-Path $baseline 'summary.json'
$hashesPath = Join-Path $baseline 'table-hashes.json'
if (-not (Test-Path -LiteralPath $summaryPath -PathType Leaf) -or -not (Test-Path -LiteralPath $hashesPath -PathType Leaf)) {
    throw 'Baseline summary.json or table-hashes.json is missing.'
}
$summary = Get-Content -Raw -LiteralPath $summaryPath | ConvertFrom-Json
$table = @($summary.tables | Where-Object { $_.k -eq 10 })
if ($summary.ranks -ne 7 -or $summary.seed -ne 202610050003 -or $summary.turn_iterations -ne 50 -or $summary.flop_iterations -ne 30 -or $table.Count -ne 1 -or $table[0].river_objective.Count -ne 10 -or $table[0].turn_objective.Count -ne 29 -or $table[0].flop_objective.Count -ne 30) {
    throw 'Baseline does not contain the exact R7/K10 10/50/30 objective history.'
}
$turnHash = (Get-Content -Raw -LiteralPath $hashesPath | ConvertFrom-Json | Where-Object { $_.Path -match 'potential-turn-k10\.bin$' }).Hash
if ($turnHash -cne '625C12BEEEC8A4243C697322AF26F061E5CF8E7E394D568C56B3F9D208A26990') {
    throw 'Baseline turn hash does not match the approved gate-control.'
}

Push-Location $project
try {
    $beforeBuild = Get-VerifiedGitProvenance
    $manifest = (Resolve-Path -LiteralPath (Join-Path $project 'Cargo.toml')).Path
    $build = @('test', '--manifest-path', $manifest, '--release', '--lib', '--no-run')
    & cargo.exe @build
    $buildExitCode = $LASTEXITCODE
    if ($buildExitCode -ne 0) { throw "Release test build failed with code $buildExitCode." }
    $beforeRun = Get-VerifiedGitProvenance
    if ($beforeBuild -cne $beforeRun) { throw 'Git revision changed during the release build.' }
    $selection = @('test', '--manifest-path', $manifest, '--release', '--lib', $testName)
    $discovery = & cargo.exe @($selection + @('--', '--list')) 2>&1
    $discoveryExitCode = $LASTEXITCODE
    $lines = @($discovery | ForEach-Object { $_.ToString() -split "`r?`n" } | ForEach-Object { $_.Trim() })
    $lines | Set-Content -LiteralPath (Join-Path $output 'test-discovery.log') -Encoding utf8
    if ($discoveryExitCode -ne 0 -or @($lines | Where-Object { $_ -ceq "$testName`: test" }).Count -ne 1) {
        throw 'Discovery did not select exactly one capture test.'
    }
    $checkpoint = Join-Path $output 'checkpoint-30.bin'
    $context = Join-Path $output 'launcher-context.txt'
    @(
        "baseline_directory=$baseline"
        "baseline_code_revision=$($summary.code_revision)"
        "baseline_turn_hash=$turnHash"
        "git_revision_before_build=$beforeBuild"
        "git_revision_before_run=$beforeRun"
        'ranks=7'
        'k=10'
        'seed=202610050003'
        'river_iterations=10'
        'turn_iterations=50'
        'flop_capture_iteration=30'
        'continuation=false'
        'production_tables=false'
        'threads=2'
        "timeout_seconds=$TimeoutSeconds"
        "test=$testName"
        'discovery_found=true'
    ) | Set-Content -LiteralPath $context -Encoding utf8
    $previousThreads = $env:RAYON_NUM_THREADS
    $previousCheckpoint = $env:HU20_R7_FLOP_CAPTURE_CHECKPOINT_PATH
    $env:RAYON_NUM_THREADS = '2'
    $env:HU20_R7_FLOP_CAPTURE_CHECKPOINT_PATH = $checkpoint
    try {
        $arguments = (($selection + @('--', '--ignored', '--exact', '--nocapture')) | ForEach-Object {
            if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
        }) -join ' '
        $exitCode = Invoke-SupervisedProcess -FilePath 'cargo.exe' -Arguments $arguments -TimeoutSeconds $TimeoutSeconds `
            -ContextPath $context -StdoutPath (Join-Path $output 'stdout.log') -StderrPath (Join-Path $output 'stderr.log') `
            -MemoryScriptPath (Join-Path $PSScriptRoot 'measure-process-memory.ps1') -MemoryPath (Join-Path $output 'memory.csv')
    } finally {
        $env:RAYON_NUM_THREADS = $previousThreads
        $env:HU20_R7_FLOP_CAPTURE_CHECKPOINT_PATH = $previousCheckpoint
    }
    if ($exitCode -ne 0) { throw "Capture exited with code $exitCode." }
    $stdout = Get-Content -Raw -LiteralPath (Join-Path $output 'stdout.log')
    if ($stdout -notmatch 'running 1 test' -or $stdout -notmatch 'test result: ok\. 1 passed; 0 failed') {
        throw 'Capture did not execute exactly one successful test.'
    }
    if (-not (Test-Path -LiteralPath $checkpoint -PathType Leaf)) { throw 'Capture did not create checkpoint-30.bin.' }
    (Get-FileHash -LiteralPath $checkpoint -Algorithm SHA256).Hash | Set-Content -LiteralPath (Join-Path $output 'checkpoint-30.sha256') -Encoding ascii
    if (@(Get-Process -Name hu20-buckets -ErrorAction SilentlyContinue).Count -ne 0) {
        throw 'Capture left hu20-buckets processes running.'
    }
} finally {
    Pop-Location
}
