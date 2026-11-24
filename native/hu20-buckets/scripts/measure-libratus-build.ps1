param(
    [Parameter(Mandatory)] [string] $OutputDirectory,
    [string] $Ks = '50,200',
    [string] $Centroid = 'mean'
)

$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$exe = (Resolve-Path 'native/hu20-buckets/target/release/hu20-buckets.exe').Path
$out = (Resolve-Path $OutputDirectory).Path
$arguments = @('build', '--abstraction', 'libratus-ochs', '--ranks', '7', '--ks', $Ks,
    '--seed', '2026112101', '--iterations', '50', '--cdf-centroid', $Centroid, '--out', ('"' + $out + '"'))
$watch = [Diagnostics.Stopwatch]::StartNew()
$process = Start-Process -FilePath $exe -ArgumentList $arguments -WindowStyle Hidden -PassThru `
    -RedirectStandardOutput (Join-Path $out 'stdout.log') -RedirectStandardError (Join-Path $out 'stderr.log')
$peak = 0L
while (-not $process.HasExited) {
    $process.Refresh()
    $peak = [Math]::Max($peak, $process.PeakWorkingSet64)
    Start-Sleep -Milliseconds 100
}
$watch.Stop()
$process.Refresh()
@{ elapsed_seconds = $watch.Elapsed.TotalSeconds; peak_working_set_bytes = $peak; exit_code = $process.ExitCode } |
    ConvertTo-Json | Set-Content -Encoding utf8 -LiteralPath (Join-Path $out 'measurement.json')
if ($process.ExitCode -ne 0) { throw "Сборщик завершился с кодом $($process.ExitCode)" }
