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
    try {
        Write-Host "Downloading Forge ($version)..."
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile (Join-Path $tempDir 'forge.zip')
        Expand-Archive -LiteralPath (Join-Path $tempDir 'forge.zip') -DestinationPath $packageDir
    } catch {
        Write-Host 'No release archive available; building from source.'
    }

    if (-not (Test-Path (Join-Path $packageDir 'forge.exe')) -or -not (Test-Path (Join-Path $packageDir 'exercises'))) {
        if (-not (Get-Command git -ErrorAction SilentlyContinue)) { throw 'Git is required to build Forge from source.' }
        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Rust/Cargo is required to build Forge from source. Install it from https://rustup.rs.' }
        if ((Test-Path 'Cargo.toml') -and (Test-Path 'exercises')) {
            $sourceDir = (Get-Location).Path
        } else {
            $sourceDir = Join-Path $tempDir 'source'
            git clone --depth 1 "https://github.com/$repo.git" $sourceDir
            if ($LASTEXITCODE -ne 0) { throw 'Could not download Forge source.' }
        }
        Push-Location $sourceDir
        try {
            cargo build --release --locked
            if ($LASTEXITCODE -ne 0) { throw 'Forge build failed.' }
        } finally { Pop-Location }
        Copy-Item (Join-Path $sourceDir 'target\release\forge.exe') $packageDir
        Copy-Item (Join-Path $sourceDir 'exercises') $packageDir -Recurse
    }

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
