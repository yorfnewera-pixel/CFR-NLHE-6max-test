function Add-ProcessDiagnosticLine {
    param(
        [Parameter(Mandatory)] [string] $ContextPath,
        [Parameter(Mandatory)] [string] $Key,
        [Parameter(Mandatory)] [AllowEmptyString()] [string] $Value
    )

    "$Key=$Value" | Add-Content -LiteralPath $ContextPath -Encoding utf8
}

function Write-ProcessDiagnostics {
    param(
        [Parameter(Mandatory)] [System.Diagnostics.Process] $Process,
        [Parameter(Mandatory)] [string] $Checkpoint,
        [Parameter(Mandatory)] [string] $ContextPath
    )

    foreach ($property in @('HasExited', 'ExitCode')) {
        try {
            $value = $Process.$property
            $key = if ($property -eq 'HasExited') { 'has_exited' } else { 'exit_code' }
            Add-ProcessDiagnosticLine -ContextPath $ContextPath -Key "$Checkpoint.$key" -Value "$value"
        } catch {
            Add-ProcessDiagnosticLine -ContextPath $ContextPath -Key "$Checkpoint.$property.exception" -Value $_.Exception.ToString()
            Add-ProcessDiagnosticLine -ContextPath $ContextPath -Key "$Checkpoint.$property.stack_trace" -Value $_.ScriptStackTrace
        }
    }
}
