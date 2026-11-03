$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'r7-turn-replay-only-command.ps1')

$manifestPath = Join-Path (Split-Path -Parent $PSScriptRoot) 'Cargo.toml'
$command = Get-R7TurnReplayCargoCommand -ManifestPath $manifestPath

if (-not [IO.Path]::IsPathFullyQualified($command.ManifestPath)) {
    throw 'Manifest path must be absolute.'
}
if ($command.TestName -ne 'test_only_ranks7_turn_k10_replays_verified_step_28_only') {
    throw "Unexpected replay-only test name: $($command.TestName)"
}
foreach ($required in @('--manifest-path', '--release', '--ignored', '--exact')) {
    if ($command.ReplayArguments -notmatch [regex]::Escape($required)) {
        throw "Replay command is missing $required."
    }
}
if ($command.ReplayArguments -notmatch [regex]::Escape($command.TestName)) {
    throw 'Replay command is missing the exact test name.'
}

$targetLine = "tests::$($command.TestName): test"
$mixedOutput = @(
    '',
    'Running unittests src\\lib.rs',
    'tests::other_test: test',
    $targetLine,
    '1 test, 0 benchmarks'
)
if (-not (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines $mixedOutput -TestName $command.TestName)) {
    throw 'Discovery must accept the exact target among multi-line output.'
}
foreach ($similarLine in @(
    "tests::prefix_$($command.TestName): test",
    "tests::$($command.TestName)_suffix: test",
    "tests::$($command.TestName): benchmark"
)) {
    if (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines @($similarLine) -TestName $command.TestName) {
        throw "Discovery must reject a similar test line: $similarLine"
    }
}
foreach ($missingOutput in @(@(), @(''), @('tests::other_test: test'))) {
    if (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines $missingOutput -TestName $command.TestName) {
        throw 'Discovery must reject missing or empty output.'
    }
}
if (Test-R7TurnReplayDiscovery -CargoExitCode 1 -OutputLines @($targetLine) -TestName $command.TestName) {
    throw 'Discovery must reject a non-zero Cargo exit code.'
}
foreach ($lineEnding in @("header`r`n$targetLine`r`nsummary", "header`n$targetLine`nsummary")) {
    if (-not (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines @($lineEnding) -TestName $command.TestName)) {
        throw 'Discovery must accept CRLF and LF multi-line output.'
    }
}
