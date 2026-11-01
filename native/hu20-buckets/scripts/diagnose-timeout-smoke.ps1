$ErrorActionPreference = 'Stop'
$root = Join-Path ([IO.Path]::GetTempPath()) ('hu20-timeout-diagnostic-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
$report = Join-Path $root 'report.txt'
function Log([string]$line) { $line | Add-Content -LiteralPath $report -Encoding utf8 }
try {
    Log "started_utc=$([datetime]::UtcNow.ToString('o'))"
    $childPidFile = Join-Path $root 'descendant.pid'
    $childScript = Join-Path $PSScriptRoot 'timeout-smoke-child.ps1'
    $shell = @('powershell.exe', 'pwsh.exe') | ForEach-Object { Join-Path $PSHOME $_ } | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if (-not $shell) { throw 'PowerShell executable was not found for the smoke descendant.' }
    $command = '/c start "" /b "' + $shell + '" -NoProfile -File "' + $childScript + '" -PidPath "' + $childPidFile + '" & timeout /t 30 /nobreak > nul'
    $child = Start-Process -FilePath $env:ComSpec -ArgumentList $command -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $root 'stdout.log') -RedirectStandardError (Join-Path $root 'stderr.log')
    Log "child_pid=$($child.Id)"
    Log "child_started_utc=$([datetime]::UtcNow.ToString('o'))"
    for ($i = 0; $i -lt 20 -and -not (Test-Path -LiteralPath $childPidFile); $i++) { Start-Sleep -Milliseconds 100 }
    $descendants = if (Test-Path -LiteralPath $childPidFile) { @([int](Get-Content -Raw -LiteralPath $childPidFile)) } else { @() }
    Log "known_descendant_pids=$($descendants -join ',')"
    if ($descendants.Count -ne 1) { throw 'Smoke descendant did not record its PID; taskkill was not run.' }
    $taskkill = 'taskkill.exe /pid ' + $child.Id + ' /t /f'
    Log "taskkill_command=$taskkill"
    & taskkill.exe /pid $child.Id /t /f *> (Join-Path $root 'taskkill.log')
    Log "taskkill_exit_code=$LASTEXITCODE"
    Start-Sleep -Milliseconds 500
    Log "child_alive_after_taskkill=$(-not $child.HasExited)"
    foreach ($descendantPid in $descendants) { Log "descendant_$descendantPid.alive_after_taskkill=$([bool](Get-Process -Id $descendantPid -ErrorAction SilentlyContinue))" }
    foreach ($file in @('stdout.log', 'stderr.log')) {
        try { $stream = [IO.File]::Open((Join-Path $root $file), 'Open', 'ReadWrite', 'None'); $stream.Dispose(); Log "$file.locked=false" } catch { Log "$file.locked=true" }
    }
} catch { Log "exception=$($_.Exception.ToString())"; Log "stack=$($_.ScriptStackTrace)" } finally { Log "finished_utc=$([datetime]::UtcNow.ToString('o'))"; Write-Output "Diagnostic directory: $root"; Get-Content -Raw -LiteralPath $report }
