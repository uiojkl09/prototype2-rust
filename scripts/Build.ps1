[CmdletBinding()]
param([switch]$CoreOnly)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    foreach ($command in @('rustc','cargo')) {
        if (-not (Get-Command $command -ErrorAction SilentlyContinue)) { throw "Missing $command; install Rust through rustup and the x64 MSVC C++ tools/Windows SDK." }
    }
    & rustc -Vv
    if ($LASTEXITCODE -ne 0) { throw 'Toolchain check failed' }
    & cargo fmt --all --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed' }
    & cargo test --locked --no-default-features
    if ($LASTEXITCODE -ne 0) { throw 'Reader/collision tests failed' }
    $featureArguments = @()
    if ($CoreOnly) { $featureArguments = @('--no-default-features') }
    & cargo clippy --locked --all-targets @featureArguments -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    & cargo build --release --locked --target x86_64-pc-windows-msvc @featureArguments
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
    Write-Host 'Built target\x86_64-pc-windows-msvc\release\prototype2-rust.exe'
} finally { Pop-Location }
