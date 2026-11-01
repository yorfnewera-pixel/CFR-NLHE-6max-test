param([Parameter(Mandatory)] [string] $PidPath)
$PID | Set-Content -LiteralPath $PidPath -Encoding ascii
Start-Sleep -Seconds 30
