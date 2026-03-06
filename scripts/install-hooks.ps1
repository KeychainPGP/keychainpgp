# Configure git to use the shared hooks in .githooks/
# Usage: .\scripts\install-hooks.ps1

git config core.hooksPath .githooks
Write-Host "Git hooks configured (core.hooksPath -> .githooks)"
