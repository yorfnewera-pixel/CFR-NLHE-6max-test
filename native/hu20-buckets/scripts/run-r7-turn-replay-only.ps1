[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutputDirectory,
    [ValidateRange(1, 86400)] [int] $TimeoutSeconds = 300
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'process-supervision.ps1')
$project = Split-Path -Parent $PSScriptRoot
$output = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDirectory)
if (Test-Path -LiteralPath $output) {
    if ((Get-ChildItem -LiteralPath $output -Force | Measure-Object).Count -ne 0) {
        throw "Output directory is not empty: $output"
    }
} else {
    New-Item -ItemType Directory -Path $output | Out-Null
}

$trace = Join-Path $output 'turn-trace.log'
$checkpoints = Join-Path $output 'checkpoints'
New-Item -ItemType Directory -Path $checkpoints | Out-Null
$context = Join-Path $output 'launcher-context.txt'
@(
    "output_directory=$output"
    "timeout_seconds=$TimeoutSeconds"
    "ranks=7"
    "k=10"
    "seed=202610050003"
    "threads=2"
    "turn_iterations=28"
    "continuation=false"
    "flop=false"
    "production_tables=false"
    "test=test_only_ranks7_turn_k10_replays_verified_step_28_only"
) | Set-Content -LiteralPath $context -Encoding utf8

Push-Location $project
try {
    cargo test --release --no-run
    if ($LASTEXITCODE -ne 0) { throw "Release test build failed with code $LASTEXITCODE." }
    $previousThreads = $env:RAYON_NUM_THREADS
    $previousTrace = $env:HU20_R7_TURN_PROBE_TRACE_PATH
    $previousCheckpoints = $env:HU20_R7_TURN_PROBE_CHECKPOINT_DIR
    $env:RAYON_NUM_THREADS = '2'
    $env:HU20_R7_TURN_PROBE_TRACE_PATH = $trace
    $env:HU20_R7_TURN_PROBE_CHECKPOINT_DIR = $checkpoints
    try {
        $exitCode = Invoke-SupervisedProcess -FilePath 'cargo.exe' `
            -Arguments 'test --release test_only_ranks7_turn_k10_replays_verified_step_28_only -- --ignored --nocapture' `
            -TimeoutSeconds $TimeoutSeconds -ContextPath $context `
            -StdoutPath (Join-Path $output 'stdout.log') -StderrPath (Join-Path $output 'stderr.log') `
            -MemoryScriptPath (Join-Path $PSScriptRoot 'measure-process-memory.ps1') `
            -MemoryPath (Join-Path $output 'memory.csv')
    } finally {
        $env:RAYON_NUM_THREADS = $previousThreads
        $env:HU20_R7_TURN_PROBE_TRACE_PATH = $previousTrace
        $env:HU20_R7_TURN_PROBE_CHECKPOINT_DIR = $previousCheckpoints
    }
    if ($exitCode -ne 0) { throw "Replay exited with code $exitCode." }
    if (-not (Test-Path -LiteralPath $trace -PathType Leaf)) { throw 'Replay did not create a trace.' }
    $checkpoint = Join-Path $checkpoints 'checkpoint-28.bin'
    if (-not (Test-Path -LiteralPath $checkpoint -PathType Leaf)) {
        throw 'Replay did not create checkpoint-28.bin.'
    }
    (Get-FileHash -LiteralPath $checkpoint -Algorithm SHA256).Hash |
        Set-Content -LiteralPath (Join-Path $output 'checkpoint-28.sha256') -Encoding ascii
    if (@(Get-Process -Name hu20-buckets -ErrorAction SilentlyContinue).Count -ne 0) {
        throw 'Replay left hu20-buckets processes running.'
    }
} finally {
    Pop-Location
}
