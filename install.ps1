$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = if ($env:FORGE_REPO) { $env:FORGE_REPO } else { 'Cliedd/Lama_Facher' }
$installDir = if ($env:FORGE_INSTALL_DIR) { $env:FORGE_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Forge' }
$binDir = if ($env:FORGE_BIN_DIR) { $env:FORGE_BIN_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\Forge' }
$version = if ($env:FORGE_VERSION) { $env:FORGE_VERSION } else { 'latest' }
$tempDir = Join-Path ([IO.Path]::GetTempPath()) ('forge-install-' + [guid]::NewGuid().ToString('N'))
$packageDir = Join-Path $tempDir 'package'

try {
    New-Item -ItemType Directory -Force -Path $packageDir | Out-Null
    if ($version -eq 'latest') {
        $url = "https://github.com/$repo/releases/latest/download/forge-windows-x86_64.zip"
    } else {
        $url = "https://github.com/$repo/releases/download/$version/forge-windows-x86_64.zip"
    }
    $archivePath = Join-Path $tempDir 'forge.zip'
    $checksumsPath = Join-Path $tempDir 'SHA256SUMS'
    $checksumsUrl = $url -replace 'forge-windows-x86_64\.zip$', 'SHA256SUMS'
    Write-Host "Downloading Forge ($version)..."
    Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $archivePath
    Invoke-WebRequest -UseBasicParsing -Uri $checksumsUrl -OutFile $checksumsPath
    $checksums = Get-Content -LiteralPath $checksumsPath -Raw
    $checksumMatch = [regex]::Match($checksums, '(?m)^([A-Fa-f0-9]{64})\s+forge-windows-x86_64\.zip\s*$')
    if (-not $checksumMatch.Success) { throw 'Release checksum is missing or invalid.' }
    $actualChecksum = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash
    if ($actualChecksum -ine $checksumMatch.Groups[1].Value) { throw 'Archive checksum mismatch; installation stopped.' }
    Expand-Archive -LiteralPath $archivePath -DestinationPath $packageDir

    if (-not (Test-Path (Join-Path $packageDir 'forge.exe'))) { throw 'Package is missing forge.exe.' }
    if (-not (Test-Path (Join-Path $packageDir 'exercises\java')) -or -not (Test-Path (Join-Path $packageDir 'exercises\rust'))) {
        throw 'Package is missing Java or Rust exercises.'
    }
    $releasesDir = Join-Path $installDir 'releases'
    $releaseDir = Join-Path $releasesDir ([guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Force -Path $releaseDir, $binDir | Out-Null
    Copy-Item (Join-Path $packageDir 'forge.exe') $releaseDir
    Copy-Item (Join-Path $packageDir 'exercises') $releaseDir -Recurse
    $launcher = "@echo off`r`nset `"FORGE_HOME=$releaseDir`"`r`n`"%FORGE_HOME%\forge.exe`" %*`r`n"
    $launcherPath = Join-Path $binDir 'forge.cmd'
    [IO.File]::WriteAllText($launcherPath, $launcher, [Text.Encoding]::ASCII)

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $parts = @($userPath -split ';' | Where-Object { $_ })
    if ($parts -notcontains $binDir) {
        [Environment]::SetEnvironmentVariable('Path', (($parts + $binDir) -join ';'), 'User')
    }
    if (($env:Path -split ';') -notcontains $binDir) { $env:Path += ";$binDir" }
    Write-Host "Forge installed. Open a new terminal and run: forge doctor; forge start"
    Write-Host "Executable: $launcherPath"
} finally {
    if (Test-Path $tempDir) { Remove-Item -LiteralPath $tempDir -Recurse -Force }
}
