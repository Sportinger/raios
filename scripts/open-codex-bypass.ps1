$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host "SeedOS / RaiOS2 Codex bypass session"
Write-Host "Workspace: $RepoRoot"
Write-Host "Project memory: $RepoRoot\AGENTS.md"
Write-Host ""
Write-Host "Warning: Codex will run with approvals and sandbox disabled in this window."
Write-Host ""

codex --dangerously-bypass-approvals-and-sandbox -C $RepoRoot
