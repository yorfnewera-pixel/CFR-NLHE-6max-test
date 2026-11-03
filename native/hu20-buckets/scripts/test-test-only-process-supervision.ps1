$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

. (Join-Path $PSScriptRoot 'test-only-process-supervision.ps1')

$root = Join-Path ([IO.Path]::GetTempPath()) ('hu20-test-only-job-tree-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
try {
    $result = Invoke-TestOnlyJobProcess -FilePath $env:ComSpec -Arguments '/c start "" /b ping.exe -n 60 127.0.0.1 & ping.exe -n 60 127.0.0.1' -TimeoutSeconds 2 -StdoutPath (Join-Path $root 'stdout.log') -StderrPath (Join-Path $root 'stderr.log')
    if (-not $result.TimedOut) { throw 'The child tree did not reach the timeout.' }
    Start-Sleep -Milliseconds 200
    if (Get-Process -Name ping -ErrorAction SilentlyContinue) { throw 'A ping child survived the job termination.' }
    Write-Output 'PASS: test-only job supervision terminates the child tree.'
} finally {
    if (Test-Path -LiteralPath $root) { Remove-Item -LiteralPath $root -Recurse -Force }
}
