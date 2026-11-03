[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutputDirectory,
    [ValidateRange(2, 13)] [int] $Ranks = 4,
    [ValidateRange(1, 65535)] [int] $K = 10,
    [ValidateRange(1, 100000)] [int] $Iterations = 10,
    # Ноль означает отсутствие override: turn наследует общий лимит.
    [ValidateRange(0, 100000)] [int] $TurnIterations = 0,
    # Ноль означает отсутствие override: flop наследует общий лимит.
    [ValidateRange(0, 100000)] [int] $FlopIterations = 0,
    [ValidateRange(1, 64)] [int] $Threads = 2,
    [ValidateRange(1, 86400)] [int] $TimeoutSeconds = 3600,
    [UInt64] $Seed = 202610050003,
    [switch] $DiagnoseFinalAssignments,
    [switch] $ProfileFlopEmd
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'process-diagnostics.ps1')
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
if (-not (Test-Path -LiteralPath $output -PathType Container)) {
    throw "Output directory was not created: $output"
}

function Get-VerifiedGitProvenance {
    $revision = (git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $revision -notmatch '^[0-9a-fA-F]{40}$') {
        throw 'Unable to verify the Git revision before measurement.'
    }

    $status = git status --porcelain
    if ($LASTEXITCODE -ne 0) {
        throw 'Unable to verify the Git working tree before measurement.'
    }
    if ($status) {
        throw 'Refusing to measure a dirty working tree. Commit or stash changes first.'
    }

    [PSCustomObject]@{
        Revision = $revision.ToLowerInvariant()
        Dirty = $false
    }
}

Push-Location $project
try {
    $provenanceBeforeBuild = Get-VerifiedGitProvenance
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed with code $LASTEXITCODE."
    }
    $provenanceBeforeLaunch = Get-VerifiedGitProvenance
    if ($provenanceBeforeBuild.Revision -ne $provenanceBeforeLaunch.Revision) {
        throw 'Git revision changed during the release build; refusing to start measurement.'
    }
    $binary = Join-Path $project 'target\release\hu20-buckets.exe'
    if (-not (Test-Path -LiteralPath $binary)) { throw "Release binary was not found: $binary" }

    $arguments = @('build', '--out', $output, '--ranks', $Ranks, '--ks', $K, '--iterations', $Iterations, '--seed', $Seed, '--abstraction', 'potential')
    if ($TurnIterations -gt 0) {
        $arguments += '--turn-iterations', $TurnIterations
    }
    if ($FlopIterations -gt 0) {
        $arguments += '--flop-iterations', $FlopIterations
    }
    if ($DiagnoseFinalAssignments) {
        $arguments += '--diagnose-final-assignments'
    }
    if ($ProfileFlopEmd) {
        $arguments += '--profile-flop-emd'
    }
    $quotedArguments = ($arguments | ForEach-Object {
        if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
    }) -join ' '
    $command = "`"$binary`" $quotedArguments"
    $command | Set-Content -LiteralPath (Join-Path $output 'command.txt') -Encoding utf8
    @(
        "output_directory=$output"
        "launcher_utc=$([DateTime]::UtcNow.ToString('o'))"
        "git_revision_before_build=$($provenanceBeforeBuild.Revision)"
        "git_dirty_before_build=$($provenanceBeforeBuild.Dirty.ToString().ToLowerInvariant())"
        "git_revision_before_launch=$($provenanceBeforeLaunch.Revision)"
        "git_dirty_before_launch=$($provenanceBeforeLaunch.Dirty.ToString().ToLowerInvariant())"
        "powershell_version=$($PSVersionTable.PSVersion)"
        "powershell_edition=$($PSVersionTable.PSEdition)"
        "dotnet_version=$([Environment]::Version)"
        "os_version=$([Environment]::OSVersion.VersionString)"
        "execution_policy_process=$(Get-ExecutionPolicy -Scope Process)"
    ) | Set-Content -LiteralPath (Join-Path $output 'launcher-context.txt') -Encoding utf8

    $previousThreads = $env:RAYON_NUM_THREADS
    $env:RAYON_NUM_THREADS = $Threads.ToString()
    try {
        $exitCode = Invoke-SupervisedProcess -FilePath $binary -Arguments $quotedArguments -TimeoutSeconds $TimeoutSeconds `
            -ContextPath (Join-Path $output 'launcher-context.txt') -StdoutPath (Join-Path $output 'stdout.log') `
            -StderrPath (Join-Path $output 'stderr.log') -MemoryScriptPath (Join-Path $PSScriptRoot 'measure-process-memory.ps1') `
            -MemoryPath (Join-Path $output 'memory.csv')
    } finally {
        $env:RAYON_NUM_THREADS = $previousThreads
    }

    $tables = Get-ChildItem -LiteralPath $output -File -Filter '*.bin'
    if ($tables.Count -ne 3) { throw "Expected exactly three tables, found $($tables.Count)." }
    $tables |
        Get-FileHash -Algorithm SHA256 |
        Select-Object Path, Algorithm, Hash |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'table-hashes.json') -Encoding utf8
    $finalStatus = Get-Content -Raw -LiteralPath (Join-Path $output 'status.json') | ConvertFrom-Json
    if ($finalStatus.stage -ne 'complete' -or -not (Test-Path -LiteralPath (Join-Path $output 'summary.json') -PathType Leaf)) {
        throw 'Generator did not produce a complete summary and status.'
    }
} finally {
    Pop-Location
}
