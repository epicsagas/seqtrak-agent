$ErrorActionPreference = "Stop"

$Repo = "epicsagas/seqtrak-agent"
$Target = "x86_64-pc-windows-msvc"
$Archive = "seqtrak-agent-$Target.zip"
$DataDir = if ($env:DATA_DIR) { $env:DATA_DIR } else { Join-Path $HOME ".local\share\seqtrak-agent" }
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $HOME ".local\bin" }

if ($env:SEQTRAK_VERSION) {
  $Url = "https://github.com/$Repo/releases/download/$($env:SEQTRAK_VERSION)/$Archive"
} else {
  $Url = "https://github.com/$Repo/releases/latest/download/$Archive"
}

$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("seqtrak-agent-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

try {
  $ZipPath = Join-Path $TmpDir $Archive
  Write-Host "Downloading $Url..."
  Invoke-WebRequest -Uri $Url -OutFile $ZipPath
  Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

  New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
  Copy-Item -Force (Join-Path $TmpDir "st.exe") (Join-Path $InstallDir "st.exe")
  Copy-Item -Force (Join-Path $TmpDir "st-mcp.exe") (Join-Path $InstallDir "st-mcp.exe")

  New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
  $GuiSrc = Join-Path $TmpDir "gui"
  if (Test-Path $GuiSrc) {
    $GuiDst = Join-Path $DataDir "gui"
    if (Test-Path $GuiDst) { Remove-Item -Recurse -Force $GuiDst }
    Copy-Item -Recurse -Force $GuiSrc $GuiDst
  }
  foreach ($f in @("LICENSE", "NOTICE", "THIRD_PARTY_LICENSES.md", "THIRD_PARTY_LICENSES_GUI.md")) {
    $src = Join-Path $TmpDir $f
    if (Test-Path $src) { Copy-Item -Force $src (Join-Path $DataDir $f) }
  }

  $SkillSrc = Join-Path $TmpDir ".claude\skills\seqtrak"
  if (Test-Path $SkillSrc) {
    $ClaudeSkills = Join-Path $HOME ".claude\skills"
    New-Item -ItemType Directory -Force -Path $ClaudeSkills | Out-Null
    Copy-Item -Recurse -Force $SkillSrc (Join-Path $ClaudeSkills "seqtrak")
  }

  $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if (-not $UserPath) { $UserPath = "" }
  if (($UserPath -split ";") -notcontains $InstallDir) {
    $NewPath = if ($UserPath) { "$UserPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "Added $InstallDir to User PATH."
  }

  Write-Host "Installed st.exe and st-mcp.exe to $InstallDir"
  Write-Host "Installed web GUI and license files to $DataDir"
} finally {
  Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}
