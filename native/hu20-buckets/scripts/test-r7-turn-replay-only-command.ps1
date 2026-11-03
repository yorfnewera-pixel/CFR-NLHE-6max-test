$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'r7-turn-replay-only-command.ps1')

$manifestPath = Join-Path (Split-Path -Parent $PSScriptRoot) 'Cargo.toml'
$command = Get-R7TurnReplayCargoCommand -ManifestPath $manifestPath

if (-not [IO.Path]::IsPathFullyQualified($command.ManifestPath)) {
    throw 'Manifest path must be absolute.'
}
if ($command.FullTestName -ne 'tests::test_only_ranks7_turn_k10_replays_verified_step_28_only') {
    throw "Unexpected replay-only full test name: $($command.FullTestName)"
}
foreach ($required in @('--manifest-path', '--release', '--lib')) {
    if ($command.SelectionArguments -notcontains $required) {
        throw "Shared selection arguments are missing $required."
    }
}
if ($command.SelectionArguments[-1] -ne $command.FullTestName) {
    throw 'Shared selection arguments are missing the exact full test name.'
}
foreach ($required in @('--ignored', '--exact')) {
    if ($command.LibtestArguments -notcontains $required) {
        throw "Libtest arguments are missing $required."
    }
}

$targetLine = "$($command.FullTestName): test"
$mixedOutput = @(
    '',
    'Running unittests src\\lib.rs',
    'tests::other_test: test',
    $targetLine,
    '1 test, 0 benchmarks'
)
if (-not (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines $mixedOutput -FullTestName $command.FullTestName)) {
    throw 'Discovery must accept the exact target among multi-line output.'
}
foreach ($similarLine in @(
    "prefix_$($command.FullTestName): test",
    "$($command.FullTestName)_suffix: test",
    "$($command.FullTestName): benchmark"
)) {
    if (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines @($similarLine) -FullTestName $command.FullTestName) {
        throw "Discovery must reject a similar test line: $similarLine"
    }
}
foreach ($missingOutput in @(@(), @(''), @('tests::other_test: test'))) {
    if (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines $missingOutput -FullTestName $command.FullTestName) {
        throw 'Discovery must reject missing or empty output.'
    }
}
if (Test-R7TurnReplayDiscovery -CargoExitCode 1 -OutputLines @($targetLine) -FullTestName $command.FullTestName) {
    throw 'Discovery must reject a non-zero Cargo exit code.'
}
foreach ($lineEnding in @("header`r`n$targetLine`r`nsummary", "header`n$targetLine`nsummary")) {
    if (-not (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines @($lineEnding) -FullTestName $command.FullTestName)) {
        throw 'Discovery must accept CRLF and LF multi-line output.'
    }
}
if (Test-R7TurnReplayDiscovery -CargoExitCode 0 -OutputLines @($targetLine, $targetLine) -FullTestName $command.FullTestName) {
    throw 'Discovery must reject multiple selected tests.'
}
