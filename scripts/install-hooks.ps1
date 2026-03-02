# Install git hooks from scripts/hooks/ into .git/hooks/
# Usage: .\scripts\install-hooks.ps1

$ErrorActionPreference = "Stop"

$RepoRoot = git rev-parse --show-toplevel
$HooksSrc = Join-Path (Join-Path $RepoRoot "scripts") "hooks"
$HooksDst = Join-Path (Join-Path $RepoRoot ".git") "hooks"

Get-ChildItem -Path $HooksSrc -File | ForEach-Object {
    Copy-Item $_.FullName -Destination (Join-Path $HooksDst $_.Name) -Force
    Write-Host "Installed $($_.Name)"
}

Write-Host "All hooks installed."