[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutputDirectory,
    [Parameter(Mandatory)] [string] $CheckpointPath,
    [ValidateRange(1, 86400)] [int] $TimeoutSeconds = 300
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'process-supervision.ps1')
$expectedCheckpointSha256 = 'C65A5C0B011B6841D0BB0F0DDB23710938D4EDDE8EE1DF9A5DC80495A4B5FC65'
$project = Split-Path -Parent $PSScriptRoot
$testName = 'tests::test_only_ranks7_flop_k10_continues_checkpoint_thirty_to_fifty'
$sourceCheckpoint = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($CheckpointPath)
$output = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDirectory)
if (-not (Test-Path -LiteralPath $sourceCheckpoint -PathType Leaf)) {
    throw "Checkpoint does not exist: $sourceCheckpoint"
}
$actualCheckpointSha256 = (Get-FileHash -LiteralPath $sourceCheckpoint -Algorithm SHA256).Hash
if ($actualCheckpointSha256 -cne $expectedCheckpointSha256) {
    throw "Checkpoint SHA-256 mismatch: expected $expectedCheckpointSha256, got $actualCheckpointSha256"
}
if (Test-Path -LiteralPath $output) {
    if ((Get-ChildItem -LiteralPath $output -Force | Measure-Object).Count -ne 0) {
        throw "Output directory is not empty: $output"
    }
} else {
    New-Item -ItemType Directory -Path $output | Out-Null
}

function Get-VerifiedGitRevision {
    $revision = (git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $revision -notmatch '^[0-9a-fA-F]{40}$') {
        throw 'Unable to verify Git revision.'
    }
    $status = git status --porcelain
    if ($LASTEXITCODE -ne 0 -or $status) {
        throw 'Refusing to run from a dirty working tree.'
    }
    $revision.ToLowerInvariant()
}

$checkpoints = Join-Path $output 'checkpoints'
New-Item -ItemType Directory -Path $checkpoints | Out-Null
$trace = Join-Path $output 'flop-trace.log'
$context = Join-Path $output 'launcher-context.txt'
Push-Location $project
try {
    $beforeBuild = Get-VerifiedGitRevision
    $manifest = (Resolve-Path -LiteralPath (Join-Path $project 'Cargo.toml')).Path
    & cargo.exe test --manifest-path $manifest --release --lib --no-run
    $buildExitCode = $LASTEXITCODE
    if ($buildExitCode -ne 0) { throw "Release test build failed with code $buildExitCode." }
    $beforeRun = Get-VerifiedGitRevision
    if ($beforeBuild -cne $beforeRun) { throw 'Git revision changed during the release build.' }
    $selection = @('test', '--manifest-path', $manifest, '--release', '--lib', $testName)
    $discovery = & cargo.exe @($selection + @('--', '--list')) 2>&1
    $discoveryExitCode = $LASTEXITCODE
    $lines = @($discovery | ForEach-Object { $_.ToString() -split "`r?`n" } | ForEach-Object { $_.Trim() })
    $lines | Set-Content -LiteralPath (Join-Path $output 'test-discovery.log') -Encoding utf8
    if ($discoveryExitCode -ne 0 -or @($lines | Where-Object { $_ -ceq "$testName`: test" }).Count -ne 1) {
        throw 'Discovery did not select exactly one continuation test.'
    }
    @(
        "source_checkpoint=$sourceCheckpoint"
        "source_checkpoint_sha256=$actualCheckpointSha256"
        "git_revision_before_build=$beforeBuild"
        "git_revision_before_run=$beforeRun"
        'ranks=7'
        'k=10'
        'seed=202610050003'
        'continuation_start=31'
        'continuation_end=50'
        'production_tables=false'
        'threads=2'
        "timeout_seconds=$TimeoutSeconds"
        "test=$testName"
        'discovery_found=true'
    ) | Set-Content -LiteralPath $context -Encoding utf8
    $previousThreads = $env:RAYON_NUM_THREADS
    $previousCheckpoint = $env:HU20_R7_FLOP_CONTINUATION_CHECKPOINT_PATH
    $previousDirectory = $env:HU20_R7_FLOP_CONTINUATION_CHECKPOINT_DIR
    $previousTrace = $env:HU20_R7_FLOP_CONTINUATION_TRACE_PATH
    $env:RAYON_NUM_THREADS = '2'
    $env:HU20_R7_FLOP_CONTINUATION_CHECKPOINT_PATH = $sourceCheckpoint
    $env:HU20_R7_FLOP_CONTINUATION_CHECKPOINT_DIR = $checkpoints
    $env:HU20_R7_FLOP_CONTINUATION_TRACE_PATH = $trace
    try {
        $arguments = (($selection + @('--', '--ignored', '--exact', '--nocapture')) | ForEach-Object {
            if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
        }) -join ' '
        $exitCode = Invoke-SupervisedProcess -FilePath 'cargo.exe' -Arguments $arguments -TimeoutSeconds $TimeoutSeconds `
            -ContextPath $context -StdoutPath (Join-Path $output 'stdout.log') -StderrPath (Join-Path $output 'stderr.log') `
            -MemoryScriptPath (Join-Path $PSScriptRoot 'measure-process-memory.ps1') -MemoryPath (Join-Path $output 'memory.csv')
    } finally {
        $env:RAYON_NUM_THREADS = $previousThreads
        $env:HU20_R7_FLOP_CONTINUATION_CHECKPOINT_PATH = $previousCheckpoint
        $env:HU20_R7_FLOP_CONTINUATION_CHECKPOINT_DIR = $previousDirectory
        $env:HU20_R7_FLOP_CONTINUATION_TRACE_PATH = $previousTrace
    }
    if ($exitCode -ne 0) { throw "Continuation exited with code $exitCode." }
    $stdout = Get-Content -Raw -LiteralPath (Join-Path $output 'stdout.log')
    if ($stdout -notmatch 'running 1 test' -or $stdout -notmatch 'test result: ok\. 1 passed; 0 failed') {
        throw 'Continuation did not execute exactly one successful test.'
    }
    if (-not (Test-Path -LiteralPath $trace -PathType Leaf)) { throw 'Continuation did not create a trace.' }
    foreach ($iteration in 31..50) {
        $checkpoint = Join-Path $checkpoints ("checkpoint-{0:D2}.bin" -f $iteration)
        if (-not (Test-Path -LiteralPath $checkpoint -PathType Leaf)) {
            throw "Continuation did not create checkpoint for step $iteration."
        }
    }
    Get-ChildItem -LiteralPath $checkpoints -File -Filter 'checkpoint-*.bin' |
        Get-FileHash -Algorithm SHA256 |
        Select-Object Path, Algorithm, Hash |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'checkpoint-hashes.json') -Encoding utf8
    if (@(Get-Process -Name cargo -ErrorAction SilentlyContinue).Count -ne 0) {
        throw 'Continuation left cargo processes running.'
    }
} finally {
    Pop-Location
}
