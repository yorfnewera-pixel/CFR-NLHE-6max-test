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
