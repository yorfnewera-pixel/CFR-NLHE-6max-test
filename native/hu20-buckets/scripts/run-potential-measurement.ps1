[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutputDirectory,
    [ValidateRange(2, 13)] [int] $Ranks = 4,
    [ValidateRange(1, 65535)] [int] $K = 10,
    [ValidateRange(1, 100000)] [int] $Iterations = 10,
    [ValidateRange(1, 64)] [int] $Threads = 2,
    [ValidateRange(1, 86400)] [int] $TimeoutSeconds = 3600,
    [UInt64] $Seed = 202610050003
)

$ErrorActionPreference = 'Stop'
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

Push-Location $project
try {
    if (git status --porcelain) {
        throw 'Refusing to measure a dirty working tree. Commit or stash changes first.'
    }
    cargo build --release
    $binary = Join-Path $project 'target\release\hu20-buckets.exe'
    if (-not (Test-Path -LiteralPath $binary)) { throw "Release binary was not found: $binary" }

    $arguments = @('build', '--out', $output, '--ranks', $Ranks, '--ks', $K, '--iterations', $Iterations, '--seed', $Seed, '--abstraction', 'potential')
    $quotedArguments = ($arguments | ForEach-Object {
        if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
    }) -join ' '
    $command = "`"$binary`" $quotedArguments"
    $command | Set-Content -LiteralPath (Join-Path $output 'command.txt') -Encoding utf8
    "output_directory=$output" | Set-Content -LiteralPath (Join-Path $output 'launcher-context.txt') -Encoding utf8

    $previousThreads = $env:RAYON_NUM_THREADS
    $env:RAYON_NUM_THREADS = $Threads.ToString()
    $process = $null
    $memoryJob = $null
    try {
        $process = Start-Process -FilePath $binary -ArgumentList $quotedArguments -PassThru -WindowStyle Hidden `
            -RedirectStandardOutput (Join-Path $output 'stdout.log') `
            -RedirectStandardError (Join-Path $output 'stderr.log')
        $memoryJob = Start-Job -ScriptBlock $ExecutionContext.InvokeCommand.NewScriptBlock("& '$PSScriptRoot\measure-process-memory.ps1' -Process (Get-Process -Id $($process.Id)) -Path '$output\memory.csv'")
        if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
            & taskkill.exe /pid $process.Id /t /f | Out-Null
            throw "Timed out after $TimeoutSeconds seconds; process was stopped."
        }
        Receive-Job -Job $memoryJob -Wait -AutoRemoveJob | Out-Null
        $process.Refresh()
        $exitCode = $process.ExitCode
        if ($null -eq $exitCode) {
            $summaryPath = Join-Path $output 'summary.json'
            $statusPath = Join-Path $output 'status.json'
            $completed = (Test-Path -LiteralPath $summaryPath -PathType Leaf) -and
                (Test-Path -LiteralPath $statusPath -PathType Leaf) -and
                ((Get-Content -Raw -LiteralPath $statusPath | ConvertFrom-Json).stage -eq 'complete')
            if ($completed) {
                $exitCode = 0
            } else {
                throw "Generator process exited but its exit code is unavailable. See $output\stderr.log"
            }
        }
        "process_id=$($process.Id)`nexit_code=$exitCode" | Add-Content -LiteralPath (Join-Path $output 'launcher-context.txt') -Encoding utf8
        if ($exitCode -ne 0) { throw "Generator exited with code $exitCode. See $output\stderr.log" }
    } finally {
        if ($memoryJob) {
            if ($memoryJob.State -eq 'Running') { Stop-Job -Job $memoryJob }
            Remove-Job -Job $memoryJob -Force -ErrorAction SilentlyContinue
        }
        if ($process -and -not $process.HasExited) { & taskkill.exe /pid $process.Id /t /f | Out-Null }
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
