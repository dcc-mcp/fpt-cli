[CmdletBinding()]
param(
    [string]$Version = $env:FPT_INSTALL_VERSION,
    [string]$InstallDir = $(if ($env:FPT_INSTALL_DIR) { $env:FPT_INSTALL_DIR } else { Join-Path $env:USERPROFILE ".fpt\bin" }),
    [string]$Repository = $(if ($env:FPT_INSTALL_REPOSITORY) { $env:FPT_INSTALL_REPOSITORY } else { "dcc-mcp/fpt-cli" })
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
switch ($architecture) {
    "x64" { $target = "x86_64-pc-windows-msvc" }
    default { throw "Unsupported Windows architecture: $architecture" }
}

if ([string]::IsNullOrWhiteSpace($Version) -or $Version -eq "latest") {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases/latest"
    $tag = if ($null -ne $release) { $release.PSObject.Properties['tag_name'] }
    if ($null -eq $tag -or $tag.Value -isnot [string] -or $tag.Value -cnotmatch '^v[0-9][0-9A-Za-z.+-]*$') {
        throw "Latest release did not provide a valid versioned tag"
    }
    $Version = $tag.Value
}
if (-not $Version.StartsWith("v")) {
    $Version = "v$Version"
}
$downloadAsset = "fpt-$Version-$target.zip"
$downloadUrl = "https://github.com/$Repository/releases/download/$Version/$downloadAsset"

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("fpt-install-" + [System.Guid]::NewGuid().ToString("N"))
$archivePath = Join-Path $tempRoot $downloadAsset
$extractDir = Join-Path $tempRoot "extract"
$binaryPath = Join-Path $InstallDir "fpt.exe"

try {
    New-Item -ItemType Directory -Force -Path $tempRoot, $extractDir, $InstallDir | Out-Null

    Write-Host "Downloading $downloadAsset from $downloadUrl"
    Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath
    Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force
    Copy-Item -Path (Join-Path $extractDir "fpt.exe") -Destination $binaryPath -Force

    Write-Host "Installed fpt to $binaryPath"
    if (-not (($env:PATH -split ';') -contains $InstallDir)) {
        Write-Host "Add $InstallDir to PATH to run 'fpt' directly."
    }
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item -Path $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
