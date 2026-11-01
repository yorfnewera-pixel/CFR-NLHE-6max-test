. (Join-Path $PSScriptRoot 'process-diagnostics.ps1')

function Invoke-SupervisedProcess {
    param(
        [Parameter(Mandatory)] [string] $FilePath,
        [Parameter(Mandatory)] [string] $Arguments,
        [Parameter(Mandatory)] [int] $TimeoutSeconds,
        [Parameter(Mandatory)] [string] $ContextPath,
        [Parameter(Mandatory)] [string] $StdoutPath,
        [Parameter(Mandatory)] [string] $StderrPath,
        [Parameter(Mandatory)] [string] $MemoryScriptPath,
        [Parameter(Mandatory)] [string] $MemoryPath,
        [switch] $DisableMemoryMonitor
    )

    $process = $null
    $memoryJob = $null
    $stdoutFile = $null
    $stderrFile = $null
    $stdoutCopy = $null
    $stderrCopy = $null
    try {
        $startInfo = New-Object System.Diagnostics.ProcessStartInfo
        $startInfo.FileName = $FilePath
        $startInfo.Arguments = $Arguments
        $startInfo.UseShellExecute = $false
        $startInfo.CreateNoWindow = $true
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true
        $process = New-Object System.Diagnostics.Process
        $process.StartInfo = $startInfo
        if (-not $process.Start()) { throw 'Child process did not start.' }
        $stdoutFile = New-Object System.IO.FileStream($StdoutPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write, [System.IO.FileShare]::Read)
        $stderrFile = New-Object System.IO.FileStream($StderrPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write, [System.IO.FileShare]::Read)
        $stdoutCopy = $process.StandardOutput.BaseStream.CopyToAsync($stdoutFile)
        $stderrCopy = $process.StandardError.BaseStream.CopyToAsync($stderrFile)
        "process_id=$($process.Id)`nprocess_started_utc=$([DateTime]::UtcNow.ToString('o'))" | Add-Content -LiteralPath $ContextPath -Encoding utf8
        if (-not $DisableMemoryMonitor) { $memoryJob = Start-Job -ScriptBlock $ExecutionContext.InvokeCommand.NewScriptBlock("& '$MemoryScriptPath' -Process (Get-Process -Id $($process.Id)) -Path '$MemoryPath'") }
        $completed = $process.WaitForExit($TimeoutSeconds * 1000)
        "wait_for_exit=$completed" | Add-Content -LiteralPath $ContextPath -Encoding utf8
        Write-ProcessDiagnostics -Process $process -Checkpoint 'after_wait_for_exit' -ContextPath $ContextPath
        if (-not $completed) {
            "timeout=true" | Add-Content -LiteralPath $ContextPath -Encoding utf8
            & taskkill.exe /pid $process.Id /t /f | Out-Null
            "taskkill_exit_code=$LASTEXITCODE" | Add-Content -LiteralPath $ContextPath -Encoding utf8
            $stopped = $process.WaitForExit(5000)
            "taskkill_process_stopped=$stopped" | Add-Content -LiteralPath $ContextPath -Encoding utf8
            throw "Timed out after $TimeoutSeconds seconds; stop result was recorded in $ContextPath."
        }
        if (-not [Threading.Tasks.Task]::WaitAll(@($stdoutCopy, $stderrCopy), 5000)) { throw 'Output transfer did not finish within 5 seconds.' }
        $memoryJobError = $null
        if ($memoryJob) { try {
            $memoryJobState = $memoryJob.State
            if (-not (Wait-Job -Job $memoryJob -Timeout 5)) { throw 'Memory monitor did not finish within 5 seconds.' }
            Receive-Job -Job $memoryJob -Wait -AutoRemoveJob -ErrorAction Stop | Out-Null
            "memory_job_state=$memoryJobState" | Add-Content -LiteralPath $ContextPath -Encoding utf8
        } catch {
            $memoryJobError = $_
            "memory_job_error=$($_.Exception.ToString())`nmemory_job_stack_trace=$($_.ScriptStackTrace)" | Add-Content -LiteralPath $ContextPath -Encoding utf8
        } }
        Write-ProcessDiagnostics -Process $process -Checkpoint 'after_receive_job' -ContextPath $ContextPath
        $process.Refresh()
        Write-ProcessDiagnostics -Process $process -Checkpoint 'after_refresh' -ContextPath $ContextPath
        $exitCode = $process.ExitCode
        "exit_code=$exitCode" | Add-Content -LiteralPath $ContextPath -Encoding utf8
        if ($null -eq $exitCode) { throw "Child process exited but its exit code is unavailable. See $StderrPath" }
        if ($exitCode -ne 0) { throw "Generator exited with code $exitCode. See $StderrPath" }
        if ($memoryJobError) { throw 'Memory monitor failed; child exit code was recorded separately.' }
        return $exitCode
    } finally {
        if ($memoryJob) {
            if ($memoryJob.State -eq 'Running') { Stop-Job -Job $memoryJob }
            Remove-Job -Job $memoryJob -Force -ErrorAction SilentlyContinue
        }
        if ($process -and -not $process.HasExited) { & taskkill.exe /pid $process.Id /t /f | Out-Null }
        try { foreach ($copy in @($stdoutCopy, $stderrCopy)) { if ($copy -and -not $copy.IsCompleted -and -not $copy.Wait(5000)) { throw 'Output transfer cleanup timed out.' } } } catch { "copy_cleanup_error=$($_.Exception.ToString())" | Add-Content -LiteralPath $ContextPath -Encoding utf8 } finally {
            if ($stdoutFile) { $stdoutFile.Dispose() }
            if ($stderrFile) { $stderrFile.Dispose() }
        }
        if ($process) { $process.Dispose() }
    }
}
