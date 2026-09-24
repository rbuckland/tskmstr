# tskmstr installer for Windows.
#
#   irm https://raw.githubusercontent.com/rbuckland/tskmstr/main/install.ps1 | iex
#
# Downloads the latest per-user MSI (CLI + tray widget), installs it silently
# (no administrator rights: %LOCALAPPDATA%\Programs\tskmstr is added to your
# PATH and the tray widget is registered to start at login), adds a `t`
# alias for PowerShell and writes a config template if you have none.
#
# Environment overrides:
#   TSKMSTR_VERSION        e.g. v0.6.3 (default: latest release)
#   TSKMSTR_DOWNLOAD_BASE  alternative download base URL (testing / mirrors)
$ErrorActionPreference = 'Stop'

$Repo = 'rbuckland/tskmstr'
$Headers = @{ 'User-Agent' = 'tskmstr-install' }

function Say($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

$Tag = $env:TSKMSTR_VERSION
if (-not $Tag) {
    Say 'Looking up the latest release'
    $Tag = (Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest" -Headers $Headers).tag_name
}
$Version = $Tag.TrimStart('v')
$Base = if ($env:TSKMSTR_DOWNLOAD_BASE) { $env:TSKMSTR_DOWNLOAD_BASE } else { "https://github.com/$Repo/releases/download/$Tag" }

$Msi = "tskmstr-$Version-windows-x64.msi"
$MsiPath = Join-Path $env:TEMP $Msi
Say "Downloading $Msi"
Invoke-WebRequest "$Base/$Msi" -OutFile $MsiPath -Headers $Headers -UseBasicParsing

Say 'Installing (per-user, silent)'
$Log = Join-Path $env:TEMP 'tskmstr-install.log'
$p = Start-Process msiexec.exe -ArgumentList "/i `"$MsiPath`" /qn /norestart /L*v `"$Log`"" -Wait -PassThru
if ($p.ExitCode -ne 0) { throw "msiexec failed with exit code $($p.ExitCode); see $Log" }
Remove-Item $MsiPath -ErrorAction SilentlyContinue

$InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\tskmstr'
# make the tools available in this session too (the MSI updated the persistent user PATH)
if (-not (($env:Path -split ';') -contains $InstallDir)) { $env:Path = "$InstallDir;$env:Path" }

# ------------------------------------------------------------ alias ----
# `t` is also installed as t.exe (works in cmd.exe); add a PowerShell alias as well.
$ProfileDir = Split-Path -Parent $PROFILE
if (-not (Test-Path $ProfileDir)) { New-Item -ItemType Directory -Path $ProfileDir -Force | Out-Null }
if (-not (Test-Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE -Force | Out-Null }
if (-not (Select-String -Path $PROFILE -Pattern 'Set-Alias -Name t -Value tskmstr' -Quiet)) {
    Add-Content -Path $PROFILE -Value "`n# tskmstr`nSet-Alias -Name t -Value tskmstr"
    Say "Added 'Set-Alias -Name t -Value tskmstr' to $PROFILE"
}
Set-Alias -Name t -Value tskmstr -Scope Global -ErrorAction SilentlyContinue

# ----------------------------------------------------------- config ----
$Config = Join-Path $HOME '.config\tskmstr\tskmstr.config.yml'
if (-not (Test-Path $Config)) {
    Say 'Writing a config template'
    & (Join-Path $InstallDir 'tskmstr.exe') init | Out-Null
}

Write-Host ''
Say "tskmstr $Version installed"
Write-Host ''
Write-Host "  CLI:     $InstallDir\tskmstr.exe   (alias: t)"
Write-Host "  Tray:    tskmstr-tray   (Start Menu 'tskmstr tray'; starts automatically at your next login)"
Write-Host "  Config:  $Config"
Write-Host ''
Write-Host 'Next steps:'
Write-Host '  1. Edit the config: add your repositories/projects and store API tokens in the'
Write-Host '     Windows Credential Manager (the template explains how).'
Write-Host '  2. Open a new terminal, then:  t              # list your tasks'
Write-Host '  3. tskmstr-tray                               # start the tray widget now'
Write-Host '     tskmstr-tray autostart disable             # if you do not want it at login'
