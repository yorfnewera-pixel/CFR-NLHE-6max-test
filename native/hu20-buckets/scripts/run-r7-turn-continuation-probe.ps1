[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutputDirectory,
    [Parameter(Mandatory)] [string] $CheckpointPath,
    [ValidateRange(1, 86400)] [int] $TimeoutSeconds = 300,
    [switch] $VerifyOnly
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'process-supervision.ps1')
. (Join-Path $PSScriptRoot 'r7-turn-continuation-command.ps1')

$expectedCheckpointSha256 = '94251D3D492741A62D1E38216F4C472597BAF6A4F12FDE71FB203C9AB3B213FC'
$project = Split-Path -Parent $PSScriptRoot
$cargoCommand = Get-R7TurnContinuationCargoCommand -ManifestPath (Join-Path $project 'Cargo.toml') -Inspection:$VerifyOnly
$sourceCheckpoint = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($CheckpointPath)
if (-not (Test-Path -LiteralPath $sourceCheckpoint -PathType Leaf)) {
    throw "Checkpoint does not exist: $sourceCheckpoint"
}
$actualCheckpointSha256 = (Get-FileHash -LiteralPath $sourceCheckpoint -Algorithm SHA256).Hash
if ($actualCheckpointSha256 -cne $expectedCheckpointSha256) {
    throw "Checkpoint SHA-256 mismatch: expected $expectedCheckpointSha256, got $actualCheckpointSha256"
}

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
    "checkpoint_path=$sourceCheckpoint"
    "checkpoint_sha256=$actualCheckpointSha256"
    "timeout_seconds=$TimeoutSeconds"
    'ranks=7'
    'k=10'
    'seed=202610050003'
    'threads=2'
    'continuation_start=29'
    'continuation_end=50'
    'flop=false'
    'production_tables=false'
    "verify_only=$VerifyOnly"
    "inspection=$($cargoCommand.Inspection)"
    "manifest_path=$($cargoCommand.ManifestPath)"
    "test=$($cargoCommand.FullTestName)"
    "selection_arguments=$(ConvertTo-CargoArgumentString $cargoCommand.SelectionArguments)"
    "run_arguments=$(ConvertTo-CargoArgumentString $cargoCommand.ContinuationArguments)"
) | Set-Content -LiteralPath $context -Encoding utf8

Push-Location $project
try {
    & cargo.exe @($cargoCommand.BuildArguments)
    $buildExitCode = $LASTEXITCODE
    "release_build_exit_code=$buildExitCode" | Add-Content -LiteralPath $context -Encoding utf8
    if ($buildExitCode -ne 0) { throw "Release test build failed with code $buildExitCode." }

    $discovery = & cargo.exe @($cargoCommand.DiscoveryArguments) 2>&1
    $discoveryExitCode = $LASTEXITCODE
    "discovery_exit_code=$discoveryExitCode" | Add-Content -LiteralPath $context -Encoding utf8
    $discoveryLines = @($discovery | ForEach-Object { $_.ToString() })
    $discoveryLines | Set-Content -LiteralPath (Join-Path $output 'test-discovery.log') -Encoding utf8
    if (-not (Test-R7TurnContinuationDiscovery -CargoExitCode $discoveryExitCode -OutputLines $discoveryLines -FullTestName $cargoCommand.FullTestName)) {
        throw "Continuation discovery did not select exactly $($cargoCommand.FullTestName)."
    }
    'discovery_found=true' | Add-Content -LiteralPath $context -Encoding utf8
    $previousThreads = $env:RAYON_NUM_THREADS
    $previousTrace = $env:HU20_R7_TURN_PROBE_TRACE_PATH
    $previousCheckpoints = $env:HU20_R7_TURN_PROBE_CHECKPOINT_DIR
    $previousCheckpoint = $env:HU20_R7_TURN_CONTINUATION_CHECKPOINT_PATH
    $env:RAYON_NUM_THREADS = '2'
    $env:HU20_R7_TURN_PROBE_TRACE_PATH = $trace
    $env:HU20_R7_TURN_PROBE_CHECKPOINT_DIR = $checkpoints
    $env:HU20_R7_TURN_CONTINUATION_CHECKPOINT_PATH = $sourceCheckpoint
    try {
        $exitCode = Invoke-SupervisedProcess -FilePath 'cargo.exe' `
            -Arguments (ConvertTo-CargoArgumentString $cargoCommand.ContinuationArguments) `
            -TimeoutSeconds $TimeoutSeconds -ContextPath $context `
            -StdoutPath (Join-Path $output 'stdout.log') -StderrPath (Join-Path $output 'stderr.log') `
            -MemoryScriptPath (Join-Path $PSScriptRoot 'measure-process-memory.ps1') `
            -MemoryPath (Join-Path $output 'memory.csv')
    } finally {
        $env:RAYON_NUM_THREADS = $previousThreads
        $env:HU20_R7_TURN_PROBE_TRACE_PATH = $previousTrace
        $env:HU20_R7_TURN_PROBE_CHECKPOINT_DIR = $previousCheckpoints
        $env:HU20_R7_TURN_CONTINUATION_CHECKPOINT_PATH = $previousCheckpoint
    }
    if ($exitCode -ne 0) { throw "Continuation exited with code $exitCode." }
    $stdout = Get-Content -Raw -LiteralPath (Join-Path $output 'stdout.log')
    if ($stdout -notmatch 'running 1 test' -or $stdout -notmatch 'test result: ok\. 1 passed; 0 failed') {
        throw 'Continuation did not execute exactly one successful test.'
    }
    if (-not (Test-Path -LiteralPath $trace -PathType Leaf)) { throw 'Continuation did not create a trace.' }
    if ($VerifyOnly) {
        $inspectionTrace = Get-Content -Raw -LiteralPath $trace
        if ($inspectionTrace -notmatch 'checkpoint_gate_passed=true continuation_started=false') {
            throw 'Checkpoint inspection did not pass the gate.'
        }
        'checkpoint_gate_passed=true' | Add-Content -LiteralPath $context -Encoding utf8
        'continuation_started=false' | Add-Content -LiteralPath $context -Encoding utf8
        'verify_only_complete=true' | Add-Content -LiteralPath $context -Encoding utf8
        return
    }
    'continuation_started=true' | Add-Content -LiteralPath $context -Encoding utf8
    foreach ($iteration in 29..50) {
        $checkpoint = Join-Path $checkpoints ("checkpoint-{0:D2}.bin" -f $iteration)
        if (-not (Test-Path -LiteralPath $checkpoint -PathType Leaf)) {
            throw "Continuation did not create checkpoint for step $iteration."
        }
    }
    if (@(Get-Process -Name hu20-buckets -ErrorAction SilentlyContinue).Count -ne 0) {
        throw 'Continuation left hu20-buckets processes running.'
    }
} finally {
    Pop-Location
}
