#Requires -Version 5.1
<#
.SYNOPSIS
    Bump the KeychainPGP version across all manifests,
    commit, tag, push, and create a GitHub release.

.PARAMETER Version
    The new version string in semver format (e.g. 0.2.0)

.EXAMPLE
    .\scripts\bump-version.ps1 -Version 0.2.0
#>

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidatePattern('^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$')]
    [string]$Version
)

$ErrorActionPreference = 'Stop'

$RootDir = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $RootDir

Write-Host "==> Bumping version to $Version" -ForegroundColor Cyan

# 1. Cargo.toml (workspace)
$cargoToml = Get-Content Cargo.toml -Raw
$cargoToml = $cargoToml -replace '(?m)^(version = ")[^"]+(")', "`${1}$Version`${2}"
Set-Content Cargo.toml -Value $cargoToml -NoNewline
Write-Host "  Updated Cargo.toml"

# 2. tauri.conf.json
$tauriConf = Get-Content crates\keychainpgp-ui\tauri.conf.json -Raw
$tauriConf = $tauriConf -replace '("version":\s*")[^"]+(")', "`${1}$Version`${2}"
Set-Content crates\keychainpgp-ui\tauri.conf.json -Value $tauriConf -NoNewline
Write-Host "  Updated tauri.conf.json"

# 3. Frontend package.json
$frontendPkg = Get-Content crates\keychainpgp-ui\frontend\package.json -Raw
$frontendPkg = $frontendPkg -replace '("version":\s*")[^"]+(")', "`${1}$Version`${2}"
Set-Content crates\keychainpgp-ui\frontend\package.json -Value $frontendPkg -NoNewline
Write-Host "  Updated frontend package.json"

# 4. Web package.json
$webPkg = Get-Content web\package.json -Raw
$webPkg = $webPkg -replace '("version":\s*")[^"]+(")', "`${1}$Version`${2}"
Set-Content web\package.json -Value $webPkg -NoNewline
Write-Host "  Updated web package.json"

# 5. Update Cargo.lock
Write-Host "  Updating Cargo.lock..."
cargo generate-lockfile --quiet 2>$null
Write-Host "  Updated Cargo.lock"

Write-Host ""
Write-Host "==> Committing and tagging" -ForegroundColor Cyan
git add Cargo.toml Cargo.lock `
    crates/keychainpgp-ui/tauri.conf.json `
    crates/keychainpgp-ui/frontend/package.json `
    web/package.json
git commit -m "chore: bump version to v$Version"
git tag -a "v$Version" -m "Release v$Version"

Write-Host ""
Write-Host "==> Pushing to remote" -ForegroundColor Cyan
git push
git push --tags

Write-Host ""
Write-Host "==> Creating GitHub release" -ForegroundColor Cyan
gh release create "v$Version" `
    --title "v$Version" `
    --generate-notes

Write-Host ""
Write-Host "Done! Version bumped to v$Version and release created." -ForegroundColor Green
