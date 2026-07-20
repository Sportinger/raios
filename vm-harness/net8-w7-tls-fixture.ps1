param(
    [ValidateSet("Start", "SetMode", "Stop")]
    [string]$Action = "Start",
    [string]$ArtifactPath,
    [string]$ReadyPath,
    [string]$ResultPath,
    [string]$ModePath,
    [ValidateSet("serve", "silent", "malformed")]
    [string]$Mode = "serve",
    [int]$ProcessId
)

$ErrorActionPreference = "Stop"
$Project = Join-Path $PSScriptRoot "net8-w7-tls-fixture\net8-w7-tls-fixture.csproj"

if ($Action -eq "SetMode") {
    if ([string]::IsNullOrWhiteSpace($ModePath)) { throw "SetMode requires ModePath" }
    Set-Content -LiteralPath $ModePath -Value $Mode -NoNewline -Encoding ascii
    return
}

if ($Action -eq "Stop") {
    if ($ProcessId -gt 0) { Stop-Process -Id $ProcessId -Force -ErrorAction SilentlyContinue }
    return
}

foreach ($path in @($ArtifactPath, $ReadyPath, $ResultPath, $ModePath)) {
    if ([string]::IsNullOrWhiteSpace($path)) { throw "Start requires artifact, ready, result, and mode paths" }
}
Set-Content -LiteralPath $ModePath -Value "serve" -NoNewline -Encoding ascii
$stdout = "$ReadyPath.stdout.log"
$stderr = "$ReadyPath.stderr.log"
$process = Start-Process dotnet -ArgumentList @(
    "run", "--project", $Project, "--", $ArtifactPath, $ReadyPath, $ResultPath, $ModePath
) -PassThru -WindowStyle Hidden -RedirectStandardOutput $stdout -RedirectStandardError $stderr
[pscustomobject]@{ process_id = $process.Id; stdout = $stdout; stderr = $stderr } | ConvertTo-Json -Compress
