# dirtree-rdm.ps1 — Windows dispatch wrapper for dirtree-rdm
# Resolves the pre-compiled Windows binary relative to this script and forwards all arguments.
#
# Usage: .\dirtree-rdm.ps1 [args...]

$arch = "x64"  # Only x64 Windows target supported in this release
$binary = Join-Path $PSScriptRoot "dirtree-rdm\bin\dirtree-rdm-windows-$arch.exe"

if (-not (Test-Path $binary)) {
    Write-Error "Binary not found: $binary`nBuild from source: cd skills/managing-roadmaps/scripts/dirtree-rdm && cargo build --release`nOr download a release asset from GitHub Releases."
    exit 1
}

& $binary @args
