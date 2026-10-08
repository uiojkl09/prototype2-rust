[CmdletBinding()]
param(
    [string]$GameDirectory = $env:PROTOTYPE2_GAME,
    [string]$Clip = 'heller_loco_run_n',
    [string]$LogDirectory,
    [string]$DeadZone,
    [double]$LookSpeed = 2.0,
    [switch]$InvertY
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $GameDirectory) { $GameDirectory = Read-Host 'Installed Prototype 2 directory' }
if (-not (Test-Path -LiteralPath (Join-Path $GameDirectory 'boot.rcf') -PathType Leaf)) { throw 'No boot.rcf in game directory' }
$candidates = @(
    (Join-Path $projectRoot 'prototype2-rust.exe'),
    (Join-Path $projectRoot 'target\x86_64-pc-windows-msvc\release\prototype2-rust.exe'),
    (Join-Path $projectRoot 'target\release\prototype2-rust.exe')
)
$binary = $candidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
if (-not $binary) { throw 'Build first using scripts\Build.ps1 or extract the release package' }
if (-not $LogDirectory) { $LogDirectory = Join-Path $env:LOCALAPPDATA 'prototype2-rust\logs' }
$null = New-Item -ItemType Directory -Path $LogDirectory -Force
$logPath = Join-Path $LogDirectory ('animation-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.log')
Write-Host "Diagnostic log: $logPath"
$viewerArguments = @('animate', '--game', $GameDirectory, '--clip', $Clip,
    '--look-speed', $LookSpeed.ToString([Globalization.CultureInfo]::InvariantCulture),
    '--invert-y', $InvertY.IsPresent.ToString().ToLowerInvariant())
if ($DeadZone) { $viewerArguments += @('--dead-zone', $DeadZone) }
& $binary @viewerArguments 2>&1 | Tee-Object -FilePath $logPath
if ($LASTEXITCODE -ne 0) { throw "Animation viewer failed; see $logPath" }
