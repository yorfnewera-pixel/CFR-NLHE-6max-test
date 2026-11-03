function Get-R7TurnReplayCargoCommand {
    param(
        [Parameter(Mandatory)] [string] $ManifestPath
    )

    $resolvedManifest = (Resolve-Path -LiteralPath $ManifestPath).Path
    $testName = 'test_only_ranks7_turn_k10_replays_verified_step_28_only'
    $quotedManifest = '"{0}"' -f $resolvedManifest
    [PSCustomObject]@{
        ManifestPath = $resolvedManifest
        TestName = $testName
        BuildArguments = "test --manifest-path $quotedManifest --release --no-run"
        DiscoveryArguments = "test --manifest-path $quotedManifest --release $testName -- --list"
        ReplayArguments = "test --manifest-path $quotedManifest --release $testName -- --ignored --exact --nocapture"
    }
}

function Test-R7TurnReplayDiscovery {
    param(
        [Parameter(Mandatory)] [int] $CargoExitCode,
        [AllowEmptyString()] [string[]] $OutputLines,
        [Parameter(Mandatory)] [string] $TestName
    )

    if ($CargoExitCode -ne 0) {
        return $false
    }
    $expectedLine = "tests::${TestName}: test"
    $normalizedLines = @(
        $OutputLines |
            ForEach-Object { $_ -split "`r?`n" } |
            ForEach-Object { $_.Trim() }
    )
    return @($normalizedLines | Where-Object { $_ -ceq $expectedLine }).Count -gt 0
}
