function Get-R7TurnReplayCargoCommand {
    param(
        [Parameter(Mandatory)] [string] $ManifestPath
    )

    $resolvedManifest = (Resolve-Path -LiteralPath $ManifestPath).Path
    $fullTestName = 'tests::test_only_ranks7_turn_k10_replays_verified_step_28_only'
    $selectionArguments = @(
        'test',
        '--manifest-path',
        $resolvedManifest,
        '--release',
        '--lib',
        $fullTestName
    )
    $libtestArguments = @('--ignored', '--exact', '--nocapture')
    [PSCustomObject]@{
        ManifestPath = $resolvedManifest
        FullTestName = $fullTestName
        SelectionArguments = $selectionArguments
        LibtestArguments = $libtestArguments
        BuildArguments = @('test', '--manifest-path', $resolvedManifest, '--release', '--lib', '--no-run')
        DiscoveryArguments = @($selectionArguments + @('--', '--list'))
        ReplayArguments = @($selectionArguments + @('--') + $libtestArguments)
    }
}

function ConvertTo-CargoArgumentString {
    param([Parameter(Mandatory)] [string[]] $Arguments)

    ($Arguments | ForEach-Object {
        if ($_ -match '[\s"]') {
            '"{0}"' -f ($_ -replace '"', '\"')
        } else {
            $_
        }
    }) -join ' '
}

function Test-R7TurnReplayDiscovery {
    param(
        [Parameter(Mandatory)] [int] $CargoExitCode,
        [AllowEmptyString()] [string[]] $OutputLines,
        [Parameter(Mandatory)] [string] $FullTestName
    )

    if ($CargoExitCode -ne 0) {
        return $false
    }
    $expectedLine = "${FullTestName}: test"
    $normalizedLines = @(
        $OutputLines |
            ForEach-Object { $_ -split "`r?`n" } |
            ForEach-Object { $_.Trim() }
    )
    return @($normalizedLines | Where-Object { $_ -ceq $expectedLine }).Count -eq 1
}
