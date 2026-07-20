param(
    [switch]$IncludeUntracked,
    [switch]$IncludeIgnored,
    [switch]$IncludeTarget,
    [switch]$GitHistory
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$OpenAiKeyPattern = 'sk-(proj-[A-Za-z0-9_-]{40,}|svcacct-[A-Za-z0-9_-]{40,}|[A-Za-z0-9_-]{48,})'

if ($GitHistory) {
    $commits = @(git -C $RepoRoot rev-list --all)
    if ($LASTEXITCODE -ne 0) {
        throw "git rev-list --all failed."
    }

    $historyHits = @()
    foreach ($commit in $commits) {
        $matches = @(git -C $RepoRoot grep -a -l -E $OpenAiKeyPattern $commit 2>$null)
        if ($matches.Count -gt 0) {
            foreach ($match in $matches) {
                $historyHits += [pscustomobject]@{
                    Ref = $match
                    Pattern = "openai_key_like"
                }
            }
        }
    }

    if ($historyHits.Count -gt 0) {
        $historyHits | Sort-Object Ref -Unique | Format-Table -AutoSize
        throw "Secret-like values found in Git history. Values are intentionally not printed."
    }

    Write-Host "No OpenAI-key-like values found in Git history."
    exit 0
}

function Convert-ToRepoRelativePath {
    param([string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    $rootPath = [IO.Path]::GetFullPath($RepoRoot).TrimEnd('\')
    if ($fullPath.StartsWith($rootPath, [StringComparison]::OrdinalIgnoreCase)) {
        return $fullPath.Substring($rootPath.Length).TrimStart('\')
    }
    return $fullPath
}

function Get-TrackedPaths {
    git -C $RepoRoot ls-files
    if ($LASTEXITCODE -ne 0) {
        throw "git ls-files failed."
    }
}

function Get-UntrackedPaths {
    git -C $RepoRoot ls-files --others --exclude-standard
    if ($LASTEXITCODE -ne 0) {
        throw "git ls-files --others failed."
    }
}

function Get-WorktreePaths {
    Get-ChildItem -LiteralPath $RepoRoot -Recurse -Force -File | ForEach-Object {
        $relative = Convert-ToRepoRelativePath -Path $_.FullName
        if ($relative -like ".git\*") {
            return
        }
        if ((-not $IncludeTarget) -and $relative -like "target\*") {
            return
        }
        $relative
    }
}

$paths = [System.Collections.Generic.List[string]]::new()
if ($IncludeIgnored) {
    Get-WorktreePaths | ForEach-Object { $paths.Add($_) }
}
else {
    Get-TrackedPaths | ForEach-Object { $paths.Add($_) }
    if ($IncludeUntracked) {
        Get-UntrackedPaths | ForEach-Object { $paths.Add($_) }
    }
}

$hits = @()
foreach ($relative in ($paths | Sort-Object -Unique)) {
    if ([string]::IsNullOrWhiteSpace($relative)) {
        continue
    }
    if ((-not $IncludeTarget) -and $relative -like "target\*") {
        continue
    }

    $path = Join-Path $RepoRoot $relative
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        continue
    }

    try {
        $file = Get-Item -LiteralPath $path
        if ($file.Length -gt 150MB) {
            continue
        }
        $bytes = [IO.File]::ReadAllBytes($file.FullName)
        $text = [Text.Encoding]::ASCII.GetString($bytes)
        $count = [regex]::Matches($text, $OpenAiKeyPattern).Count
        if ($count -gt 0) {
            $hits += [pscustomobject]@{
                Path = $relative
                Pattern = "openai_key_like"
                Count = $count
                Bytes = $file.Length
            }
        }
    }
    catch {
        throw "Failed to scan ${relative}: $($_.Exception.Message)"
    }
}

if ($hits.Count -gt 0) {
    $hits | Sort-Object Path | Format-Table -AutoSize
    throw "Secret-like values found. Values are intentionally not printed."
}

Write-Host "No OpenAI-key-like values found."
