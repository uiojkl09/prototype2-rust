[CmdletBinding()]
param(
    [string]$GameDirectory = $env:PROTOTYPE2_GAME,
    [string]$Entry = 'art\locations\yellow_zone\Cell_29.p3d.rz',
    [string]$LogDirectory
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $GameDirectory) { $GameDirectory = Read-Host 'Installed Prototype 2 directory' }
if (-not (Test-Path -LiteralPath (Join-Path $GameDirectory 'cells.rcf') -PathType Leaf)) { throw 'No cells.rcf in game directory' }
$candidates = @(
    (Join-Path $projectRoot 'prototype2-rust.exe'),
    (Join-Path $projectRoot 'target\x86_64-pc-windows-msvc\release\prototype2-rust.exe'),
    (Join-Path $projectRoot 'target\release\prototype2-rust.exe')
)
$binary = $candidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
if (-not $binary) { throw 'Build first using scripts\Build.ps1 or extract the release package' }
if (-not $LogDirectory) { $LogDirectory = Join-Path $env:LOCALAPPDATA 'prototype2-rust\logs' }
$null = New-Item -ItemType Directory -Path $LogDirectory -Force
$logPath = Join-Path $LogDirectory ('viewer-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.log')
Write-Host "Diagnostic log: $logPath"
& $binary view --game $GameDirectory --entry $Entry 2>&1 | Tee-Object -FilePath $logPath
if ($LASTEXITCODE -ne 0) { throw "Viewer failed; see $logPath" }
