$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'r7-turn-continuation-command.ps1')

$manifestPath = Join-Path (Split-Path -Parent $PSScriptRoot) 'Cargo.toml'
$command = Get-R7TurnContinuationCargoCommand -ManifestPath $manifestPath

if (-not [IO.Path]::IsPathFullyQualified($command.ManifestPath)) {
    throw 'Manifest path must be absolute.'
}
if ($command.FullTestName -ne 'tests::test_only_ranks7_turn_k10_continues_after_verified_step_28_only') {
    throw "Unexpected continuation full test name: $($command.FullTestName)"
}
if ($command.SelectionArguments[-1] -ne $command.FullTestName) {
    throw 'Shared selection arguments are missing the exact full test name.'
}

$targetLine = "$($command.FullTestName): test"
if (-not (Test-R7TurnContinuationDiscovery -CargoExitCode 0 -OutputLines @($targetLine) -FullTestName $command.FullTestName)) {
    throw 'Discovery must select exactly one complete test name.'
}
if (Test-R7TurnContinuationDiscovery -CargoExitCode 0 -OutputLines @() -FullTestName $command.FullTestName) {
    throw 'Discovery must reject zero selected tests.'
}
if (Test-R7TurnContinuationDiscovery -CargoExitCode 0 -OutputLines @($targetLine, $targetLine) -FullTestName $command.FullTestName) {
    throw 'Discovery must reject multiple selected tests.'
}
