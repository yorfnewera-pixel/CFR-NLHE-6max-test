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
$output = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $output) {
    if ((Get-ChildItem -LiteralPath $output -Force | Measure-Object).Count -ne 0) {
        throw "Output directory is not empty: $output"
    }
} else {
    New-Item -ItemType Directory -Path $output | Out-Null
}

Push-Location $project
try {
    cargo build --release
    $binary = Join-Path $project 'target\release\hu20-buckets.exe'
    if (-not (Test-Path -LiteralPath $binary)) { throw "Release binary was not found: $binary" }

    $arguments = @('build', '--out', $output, '--ranks', $Ranks, '--ks', $K, '--iterations', $Iterations, '--seed', $Seed, '--abstraction', 'potential')
    $quotedArguments = ($arguments | ForEach-Object {
        if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
    }) -join ' '
    $command = "`"$binary`" $quotedArguments"
    $command | Set-Content -LiteralPath (Join-Path $output 'command.txt') -Encoding utf8

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
            Stop-Process -Id $process.Id -Force
            throw "Timed out after $TimeoutSeconds seconds; process was stopped."
        }
        Receive-Job -Job $memoryJob -Wait -AutoRemoveJob | Out-Null
        if ($process.ExitCode -ne 0) { throw "Generator exited with code $($process.ExitCode). See stderr.log" }
    } finally {
        if ($memoryJob) {
            if ($memoryJob.State -eq 'Running') { Stop-Job -Job $memoryJob }
            Remove-Job -Job $memoryJob -Force -ErrorAction SilentlyContinue
        }
        if ($process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force }
        $env:RAYON_NUM_THREADS = $previousThreads
    }

    Get-ChildItem -LiteralPath $output -File -Filter '*.bin' |
        Get-FileHash -Algorithm SHA256 |
        Select-Object Path, Algorithm, Hash |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'table-hashes.json') -Encoding utf8
} finally {
    Pop-Location
}
