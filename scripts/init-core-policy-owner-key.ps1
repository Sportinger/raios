param(
    [string]$PrivateKeyPath = (Join-Path $env:LOCALAPPDATA "raiOS\keys\core-policy-owner.p256.secret"),
    [string]$PublicKeyOutput = "$PSScriptRoot\..\target\core-policy\core-policy-owner.p256.pub"
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$PrivateKeyFullPath = [IO.Path]::GetFullPath($PrivateKeyPath)
$PublicKeyFullPath = [IO.Path]::GetFullPath($PublicKeyOutput)
$RepoPrefix = $RepoRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
if ($PrivateKeyFullPath.Equals($RepoRoot, [StringComparison]::OrdinalIgnoreCase) -or
    $PrivateKeyFullPath.StartsWith($RepoPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to create the Core Policy private key inside the repository."
}
if (Test-Path -LiteralPath $PrivateKeyFullPath) {
    throw "Private key already exists: $PrivateKeyFullPath"
}
if (Test-Path -LiteralPath $PublicKeyFullPath) {
    throw "Public-key output already exists: $PublicKeyFullPath"
}

$PrivateDirectory = Split-Path -Parent $PrivateKeyFullPath
$PublicDirectory = Split-Path -Parent $PublicKeyFullPath
New-Item -ItemType Directory -Force -Path $PrivateDirectory | Out-Null
New-Item -ItemType Directory -Force -Path $PublicDirectory | Out-Null

$CurrentSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$UserGrant = "*${CurrentSid}:(OI)(CI)F"
$SystemGrant = "*S-1-5-18:(OI)(CI)F"
& icacls $PrivateDirectory /inheritance:r /grant:r $UserGrant $SystemGrant | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Failed to restrict the private-key directory ACL."
}

$CreatedPrivateKey = $false
try {
    Push-Location -LiteralPath $RepoRoot
    try {
        & cargo run --locked --quiet -p core-policy-sign -- keygen $PrivateKeyFullPath $PublicKeyFullPath
        if ($LASTEXITCODE -ne 0) {
            throw "core-policy-sign keygen failed with exit code $LASTEXITCODE"
        }
        $CreatedPrivateKey = Test-Path -LiteralPath $PrivateKeyFullPath
    }
    finally {
        Pop-Location
    }

    $FileUserGrant = "*${CurrentSid}:F"
    $FileSystemGrant = "*S-1-5-18:F"
    & icacls $PrivateKeyFullPath /inheritance:r /grant:r $FileUserGrant $FileSystemGrant | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to restrict the private-key file ACL."
    }

    $Acl = Get-Acl -LiteralPath $PrivateKeyFullPath
    if (-not $Acl.AreAccessRulesProtected) {
        throw "Private-key ACL still inherits permissions."
    }
    foreach ($Rule in $Acl.Access) {
        $RuleSid = $Rule.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value
        if ($Rule.AccessControlType -eq [Security.AccessControl.AccessControlType]::Allow -and
            $RuleSid -notin @($CurrentSid, "S-1-5-18")) {
            throw "Private-key ACL grants an unexpected identity: $RuleSid"
        }
    }

    Write-Output "Core Policy owner private key created outside the repository."
    Write-Output "private: $PrivateKeyFullPath"
    Write-Output "public:  $PublicKeyFullPath"
}
catch {
    if ($CreatedPrivateKey -or (Test-Path -LiteralPath $PrivateKeyFullPath)) {
        Remove-Item -LiteralPath $PrivateKeyFullPath -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -LiteralPath $PublicKeyFullPath -Force -ErrorAction SilentlyContinue
    throw
}
