$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$Version = "main"
if ($args.Count -gt 0 -and $args[0]) { $Version = [string]$args[0] } elseif ($env:BV_VERSION) { $Version = $env:BV_VERSION }

if ($env:BV_REPO_SLUG) { $Slug = $env:BV_REPO_SLUG } else { $Slug = "BlockVerse-Mc/BlockVerse" }
if ($env:BV_REPO_URL) { $RepoUrl = $env:BV_REPO_URL } else { $RepoUrl = "https://github.com/$Slug.git" }
if ($env:BV_RELEASE_BASE) { $ReleaseBase = $env:BV_RELEASE_BASE } else { $ReleaseBase = "https://github.com/$Slug/releases/download" }
if ($env:BV_DIR) { $Dir = $env:BV_DIR } else { $Dir = Join-Path $env:USERPROFILE "blockverse-server" }
$Src = Join-Path $Dir "source"
$Log = Join-Path $Dir "install.log"
$script:LastCode = 0
$env:GIT_TERMINAL_PROMPT = "0"

function Update-SessionPath {
    $machine = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $user = [Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = "$machine;$user;$env:USERPROFILE\.cargo\bin"
}

function Test-Command {
    param([string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Invoke-Step {
    param([string]$File, [string[]]$StepArgs, [switch]$AllowFail)
    $previous = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & $File @StepArgs *>> $Log
    $script:LastCode = $LASTEXITCODE
    $ErrorActionPreference = $previous
    if ($script:LastCode -ne 0 -and -not $AllowFail) { throw "$File failed" }
}

function Read-Answer {
    param([string]$Prompt, [string]$Default)
    $answer = ""
    try { $answer = Read-Host "$Prompt [$Default]" } catch { $answer = "" }
    if ([string]::IsNullOrWhiteSpace($answer)) { return $Default }
    return $answer.Trim()
}

function Install-Prebuilt {
    $file = "blockverse-" + $Version.TrimStart("v") + "-windows-x86_64.tar.gz"
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("blockverse-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    try {
        $archive = Join-Path $tmp "server.tar.gz"
        $sumFile = Join-Path $tmp "server.sha256"
        try {
            Invoke-WebRequest -Uri "$ReleaseBase/$Version/$file" -OutFile $archive -UseBasicParsing
            Invoke-WebRequest -Uri "$ReleaseBase/$Version/$file.sha256" -OutFile $sumFile -UseBasicParsing
        } catch {
            return $false
        }
        $want = ((Get-Content -Path $sumFile -TotalCount 1) -split "\s+")[0].ToLower()
        $got = (Get-FileHash -Algorithm SHA256 -Path $archive).Hash.ToLower()
        if ($want -ne $got) { throw "checksum mismatch, the download is damaged or was modified" }
        if (-not (Test-Command "tar")) { return $false }
        $out = Join-Path $tmp "out"
        New-Item -ItemType Directory -Force -Path $out | Out-Null
        Invoke-Step "tar" @("-xzf", $archive, "-C", $out, "--strip-components=1") -AllowFail
        if ($script:LastCode -ne 0) { return $false }
        Invoke-Step (Join-Path $out "blockverse.exe") @("--version") -AllowFail
        if ($script:LastCode -ne 0) { return $false }
        Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $Dir "blockverse.exe"), (Join-Path $Dir "start.bat")
        Copy-Item -Recurse -Force (Join-Path $out "*") $Dir
        return $true
    } finally {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmp
    }
}

function Build-FromSource {
    Update-SessionPath
    if (-not (Test-Command "git")) {
        if (-not (Test-Command "winget")) { throw "git is required" }
        Invoke-Step "winget" @("install", "-e", "--id", "Git.Git", "--silent", "--accept-package-agreements", "--accept-source-agreements")
        Update-SessionPath
        if (-not (Test-Command "git")) { throw "git is required" }
    }
    if (-not (Test-Command "cargo")) {
        $init = Join-Path $env:TEMP "rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $init -UseBasicParsing
        Invoke-Step $init @("-y", "--profile", "minimal")
        Update-SessionPath
        if (-not (Test-Command "cargo")) { throw "Rust is required" }
    }
    New-Item -ItemType Directory -Force -Path $Src | Out-Null
    Set-Location $Src
    if (-not (Test-Path (Join-Path $Src ".git"))) { Invoke-Step "git" @("init", "-q") }
    Invoke-Step "git" @("remote", "remove", "origin") -AllowFail
    Invoke-Step "git" @("remote", "add", "origin", $RepoUrl)
    Invoke-Step "git" @("fetch", "-q", "--depth", "1", "origin", $Version)
    Invoke-Step "git" @("checkout", "-q", "-f", "FETCH_HEAD")
    Invoke-Step "cargo" @("build", "--release", "--manifest-path", "crates\bv-server\Cargo.toml")
    Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $Dir "blockverse.exe"), (Join-Path $Dir "start.bat")
    Copy-Item -Force (Join-Path $Src "crates\bv-server\target\release\blockverse.exe") (Join-Path $Dir "blockverse.exe")
    Copy-Item -Force (Join-Path $Src "start.bat") (Join-Path $Dir "start.bat")
    foreach ($extra in @("LICENSE", "LICENSE-APACHE-2.0", "NOTICE", "PLUGIN-EXCEPTION.md")) {
        $path = Join-Path $Src $extra
        if (Test-Path $path) { Copy-Item -Force $path (Join-Path $Dir $extra) }
    }
}

New-Item -ItemType Directory -Force -Path $Dir | Out-Null
Set-Content -Path $Log -Value ""
Push-Location $Dir

try {
    if ($Version -notmatch '^[A-Za-z0-9][A-Za-z0-9._/-]*$') { throw "invalid version" }

    Write-Host "Preparing BlockVerse"

    $Name = $env:BV_SERVER_NAME
    if (-not $Name) { $Name = Read-Answer "Server name" "BlockVerse" }
    $Language = $env:BV_LANGUAGE
    if (-not $Language) { $Language = Read-Answer "Language (en or fa)" "en" }
    if ($Language -ne "en" -and $Language -ne "fa") { $Language = "en" }

    $installed = $false
    if ($env:BV_FROM_SOURCE -ne "1" -and $Version -match '^v[0-9]') {
        $installed = Install-Prebuilt
    }
    if (-not $installed) { Build-FromSource }

    New-Item -ItemType Directory -Force -Path (Join-Path $Dir "bvmc") | Out-Null
    Set-Location (Join-Path $Dir "bvmc")
    Invoke-Step (Join-Path $Dir "blockverse.exe") @("--init", "--server-name", $Name, "--language", $Language)
    Set-Content -Path (Join-Path $Dir "VERSION") -Value $Version

    Write-Host ("BlockVerse ready: " + (Join-Path $Dir "start.bat"))
}
catch {
    Write-Host ("BlockVerse install failed: " + $_.Exception.Message)
    Write-Host ("Log: " + $Log)
}
finally {
    Pop-Location
}
