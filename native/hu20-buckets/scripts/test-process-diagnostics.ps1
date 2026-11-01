$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$scripts = $PSScriptRoot
$diagnosticsScript = Join-Path $scripts 'process-diagnostics.ps1'
$supervisionScript = Join-Path $scripts 'process-supervision.ps1'
$launcherScript = Join-Path $scripts 'run-potential-measurement.ps1'
$testRoot = Join-Path ([System.IO.Path]::GetTempPath()) ('hu20-process-diagnostics-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
$passed = $false

function Assert-Contains {
    param([string] $Text, [string] $Expected, [string] $Message)
    if (-not $Text.Contains($Expected)) { throw "$Message Expected: $Expected" }
}

try {
    . $diagnosticsScript
    . $supervisionScript

    function Invoke-Smoke { param([string] $Name, [string] $Arguments, [int] $TimeoutSeconds = 10, [string] $MemoryScript = (Join-Path $scripts 'measure-process-memory.ps1'), [switch] $DisableMemoryMonitor)
        $directory = Join-Path $testRoot $Name; New-Item -ItemType Directory -Path $directory | Out-Null
        $context = Join-Path $directory 'context.txt'; '' | Set-Content -LiteralPath $context
        $null = Invoke-SupervisedProcess -FilePath $env:ComSpec -Arguments $Arguments -TimeoutSeconds $TimeoutSeconds -ContextPath $context -StdoutPath (Join-Path $directory 'stdout.log') -StderrPath (Join-Path $directory 'stderr.log') -MemoryScriptPath $MemoryScript -MemoryPath (Join-Path $directory 'memory.csv') -DisableMemoryMonitor:$DisableMemoryMonitor
        return $context
    }

    $zeroContext = Invoke-Smoke -Name 'zero' -Arguments '/c ping -n 2 127.0.0.1 > nul & exit 0'
    Assert-Contains -Text (Get-Content -Raw -LiteralPath $zeroContext) -Expected 'exit_code=0' -Message 'Exit code 0 was not recorded.'

    $failed = $false; try { $null = Invoke-Smoke -Name 'seven' -Arguments '/c ping -n 2 127.0.0.1 > nul & exit 7' } catch { $failed = $true }; if (-not $failed) { throw 'Exit 7 was accepted.' }
    Assert-Contains -Text (Get-Content -Raw -LiteralPath (Join-Path $testRoot 'seven\context.txt')) -Expected 'exit_code=7' -Message 'Exit code 7 was not recorded.'

    $failed = $false; try { $null = Invoke-Smoke -Name 'memory-error' -Arguments '/c exit 0' -MemoryScript (Join-Path $testRoot 'missing-memory.ps1') } catch { $failed = $true }; if (-not $failed) { throw 'Memory error was accepted.' }
    $memoryContext = Get-Content -Raw -LiteralPath (Join-Path $testRoot 'memory-error\context.txt')
    Assert-Contains -Text $memoryContext -Expected 'exit_code=0' -Message 'Exit code was not retained after memory failure.'
    Assert-Contains -Text $memoryContext -Expected 'memory_job_error=' -Message 'Memory failure was not recorded.'

    $outputContext = Invoke-Smoke -Name 'dual-output' -Arguments '/c (for /l %i in (1,1,2000) do @echo out-%i) & (for /l %i in (1,1,2000) do @echo err-%i 1>&2) & exit /b 0' -DisableMemoryMonitor
    Assert-Contains -Text (Get-Content -Raw -LiteralPath $outputContext) -Expected 'exit_code=0' -Message 'Dual-output exit code was not recorded.'
    Assert-Contains -Text (Get-Content -Raw -LiteralPath (Join-Path $testRoot 'dual-output\stdout.log')) -Expected 'out-2000' -Message 'stdout transfer was incomplete.'
    Assert-Contains -Text (Get-Content -Raw -LiteralPath (Join-Path $testRoot 'dual-output\stderr.log')) -Expected 'err-2000' -Message 'stderr transfer was incomplete.'

    $failed = $false; try { $null = Invoke-Smoke -Name 'timeout' -Arguments '/c ping -n 10 127.0.0.1 > nul' -TimeoutSeconds 1 } catch { $failed = $true }; if (-not $failed) { throw 'Timeout was accepted.' }
    $timeoutDirectory = Join-Path $testRoot 'timeout'
    Assert-Contains -Text (Get-Content -Raw -LiteralPath (Join-Path $timeoutDirectory 'context.txt')) -Expected 'timeout=true' -Message 'Timeout was not recorded.'
    foreach ($file in @('stdout.log', 'stderr.log')) {
        $stream = [IO.File]::Open((Join-Path $timeoutDirectory $file), 'Open', 'ReadWrite', 'None')
        $stream.Dispose()
    }

    $launcherText = Get-Content -Raw -LiteralPath $launcherScript
    Assert-Contains -Text $launcherText -Expected 'process-supervision.ps1' -Message 'Launcher does not load the supervision helper.'
    Assert-Contains -Text $launcherText -Expected 'Invoke-SupervisedProcess' -Message 'Launcher does not use the smoke-tested supervision path.'

    $passed = $true
} finally {
    if ($passed -and (Test-Path -LiteralPath $testRoot)) {
        try { Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction Stop } catch { Write-Warning "Smoke artifacts retained: $testRoot" }
    } elseif (Test-Path -LiteralPath $testRoot) {
        Write-Warning "Smoke failure artifacts retained: $testRoot"
    }
}
if ($passed) { Write-Output 'PASS: process diagnostics smoke tests.' }
