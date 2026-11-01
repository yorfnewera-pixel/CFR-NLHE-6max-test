param(
    [Parameter(Mandatory)] [System.Diagnostics.Process] $Process,
    [Parameter(Mandatory)] [string] $Path,
    [int] $IntervalMilliseconds = 1000
)

"timestamp_utc,private_bytes,working_set_bytes" | Set-Content -LiteralPath $Path -Encoding utf8
while (-not $Process.HasExited) {
    $Process.Refresh()
    "{0:o},{1},{2}" -f [DateTime]::UtcNow, $Process.PrivateMemorySize64, $Process.WorkingSet64 |
        Add-Content -LiteralPath $Path -Encoding utf8
    Start-Sleep -Milliseconds $IntervalMilliseconds
}
$Process.Refresh()
"{0:o},{1},{2}" -f [DateTime]::UtcNow, $Process.PrivateMemorySize64, $Process.WorkingSet64 |
    Add-Content -LiteralPath $Path -Encoding utf8
