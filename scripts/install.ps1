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
    $response = Invoke-WebRequest -Uri "https://github.com/$Repository/releases/latest" -Method Head -UseBasicParsing
    $baseResponse = if ($null -ne $response) { $response.PSObject.Properties['BaseResponse'] }
    $finalUri = $null
    if ($null -ne $baseResponse -and $null -ne $baseResponse.Value) {
        $responseUri = $baseResponse.Value.PSObject.Properties['ResponseUri']
        if ($null -ne $responseUri) {
            $finalUri = $responseUri.Value
        }
        else {
            $requestMessage = $baseResponse.Value.PSObject.Properties['RequestMessage']
            if ($null -ne $requestMessage -and $null -ne $requestMessage.Value) {
                $requestUri = $requestMessage.Value.PSObject.Properties['RequestUri']
                if ($null -ne $requestUri) { $finalUri = $requestUri.Value }
            }
        }
    }
    $prefix = "https://github.com/$Repository/releases/tag/"
    if ($finalUri -isnot [uri] -or -not $finalUri.IsAbsoluteUri -or -not $finalUri.AbsoluteUri.StartsWith($prefix, [System.StringComparison]::Ordinal)) {
        throw "Latest release did not resolve to a versioned tag in $Repository"
    }
    $Version = $finalUri.AbsoluteUri.Substring($prefix.Length)
    if ($Version -cnotmatch '^v[0-9][0-9A-Za-z.+-]*$') {
        throw "Latest release did not provide a valid versioned tag"
    }
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
    Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
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
