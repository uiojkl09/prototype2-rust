[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
$projectRoot = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$packageRoot = [IO.Path]::GetFullPath($OutputDirectory)
if ($packageRoot.Equals($projectRoot, [StringComparison]::OrdinalIgnoreCase) -or $packageRoot.StartsWith($projectRoot + [IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) { throw 'Package output must be outside the source repository' }
if (Test-Path -LiteralPath $packageRoot) { throw 'Output directory already exists; use a fresh destination' }
Push-Location $projectRoot
try {
    & python (Join-Path $PSScriptRoot 'audit-publication.py')
    if ($LASTEXITCODE -ne 0) { throw 'Publication audit failed' }
    $dirty = & git status --porcelain
    if ($dirty) { throw 'Commit the reviewed source before packaging; source.zip must match the executable source' }
    & cargo build --release --locked --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
    $commit = & git rev-parse HEAD
    $null = New-Item -ItemType Directory -Path $packageRoot
    Copy-Item -LiteralPath 'target\x86_64-pc-windows-msvc\release\prototype2-rust.exe' -Destination $packageRoot
    foreach ($file in @('README.md','STATUS.md','LICENSE','THIRD_PARTY.md')) { Copy-Item -LiteralPath $file -Destination $packageRoot }
    Copy-Item -LiteralPath 'docs' -Destination $packageRoot -Recurse
    $scriptOutput = Join-Path $packageRoot 'scripts'
    $null = New-Item -ItemType Directory -Path $scriptOutput
    Copy-Item -LiteralPath 'scripts\Run.ps1' -Destination $scriptOutput
    $sourceArchive = Join-Path $packageRoot 'source.zip'
    & git archive --format=zip -o $sourceArchive HEAD
    if ($LASTEXITCODE -ne 0) { throw 'Source archive failed' }
    & python (Join-Path $PSScriptRoot 'package-licenses.py') $packageRoot
    if ($LASTEXITCODE -ne 0) { throw 'Dependency notice collection failed' }
    $buildInfo = @("source_commit=$commit", (& rustc -V), (& cargo -V), 'target=x86_64-pc-windows-msvc')
    $buildInfo | Set-Content -LiteralPath (Join-Path $packageRoot 'BUILD-INFO.txt') -Encoding UTF8
    $exeHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $packageRoot 'prototype2-rust.exe')).Hash.ToLowerInvariant()
    "$exeHash  prototype2-rust.exe" | Set-Content -LiteralPath (Join-Path $packageRoot 'SHA256SUMS.txt') -Encoding ASCII
    $zipPath = $packageRoot + '.zip'
    if (Test-Path -LiteralPath $zipPath) { throw 'ZIP destination already exists' }
    Compress-Archive -LiteralPath (Get-ChildItem -LiteralPath $packageRoot | Select-Object -ExpandProperty FullName) -DestinationPath $zipPath -CompressionLevel Optimal
    Write-Host "Deliverable: $zipPath"
} finally { Pop-Location }
