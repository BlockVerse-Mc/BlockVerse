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
$BuildLog = Join-Path $Dir "build.log"
$Manifest = "crates\bv-server\Cargo.toml"
$script:LastCode = 0
$script:StepOpen = $false
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

function Start-Step {
    param([string]$Label)
    $script:StepOpen = $true
    Write-Host "  - " -NoNewline -ForegroundColor Cyan
    Write-Host "$Label ... " -NoNewline
}

function Stop-Step {
    $script:StepOpen = $false
    Write-Host "done" -ForegroundColor Green
}

function Stop-StepNote {
    param([string]$Note)
    $script:StepOpen = $false
    Write-Host $Note -ForegroundColor Yellow
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

function Get-BarText {
    param([int]$Percent)
    $width = 24
    $filled = [math]::Floor($Percent * $width / 100)
    return ("#" * $filled) + ("." * ($width - $filled))
}

function Get-UnitTotal {
    $previous = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $lines = & cargo tree --manifest-path $Manifest -e "normal,build" --prefix none 2>$null
    $ErrorActionPreference = $previous
    if (-not $lines) { return 0 }
    $names = $lines | Where-Object { $_ -and -not $_.StartsWith("[") } | ForEach-Object { $_ -replace ' \(\*\)$', '' } | Sort-Object -Unique
    return @($names).Count
}

function Get-CompileCount {
    try {
        $stream = [System.IO.File]::Open($BuildLog, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
        $reader = New-Object System.IO.StreamReader($stream)
        $text = $reader.ReadToEnd()
        $reader.Close()
        return ([regex]::Matches($text, '(?m)^\s*Compiling ')).Count
    } catch {
        return 0
    }
}

function Invoke-Compile {
    $total = Get-UnitTotal
    Set-Content -Path $BuildLog -Value ""
    $outLog = "$BuildLog.out"
    $process = Start-Process -FilePath "cargo" -ArgumentList @("build", "--release", "--color", "never", "--manifest-path", $Manifest) -NoNewWindow -PassThru -RedirectStandardError $BuildLog -RedirectStandardOutput $outLog
    $last = -1
    while (-not $process.HasExited) {
        $count = Get-CompileCount
        if ($total -gt 0) {
            $percent = [math]::Min(99, [math]::Floor($count * 100 / $total))
            if ($percent -ne $last) {
                Write-Host ("`r  - ") -NoNewline -ForegroundColor Cyan
                Write-Host "Compiling BlockVerse " -NoNewline
                Write-Host ("[" + (Get-BarText $percent) + "] %" + $percent + "   ") -NoNewline -ForegroundColor Green
                $last = $percent
            }
        } else {
            Write-Host ("`r  - Compiling BlockVerse ... " + $count + " crates   ") -NoNewline
        }
        Start-Sleep -Seconds 1
    }
    $process.WaitForExit()
    Add-Content -Path $Log -Value (Get-Content -Path $BuildLog -ErrorAction SilentlyContinue)
    if ($process.ExitCode -ne 0) {
        Write-Host ""
        Write-Host "  - Compiling BlockVerse failed" -ForegroundColor Red
        Get-Content -Path $BuildLog -Tail 12 -ErrorAction SilentlyContinue | ForEach-Object { Write-Host $_ }
        throw "the build failed"
    }
    Write-Host ("`r  - ") -NoNewline -ForegroundColor Cyan
    Write-Host "Compiling BlockVerse " -NoNewline
    Write-Host ("[" + (Get-BarText 100) + "] %100 ") -NoNewline -ForegroundColor Green
    Write-Host "done" -ForegroundColor Green
}

function Install-Prebuilt {
    $file = "blockverse-" + $Version.TrimStart("v") + "-windows-x86_64.tar.gz"
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("blockverse-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    try {
        $archive = Join-Path $tmp "server.tar.gz"
        $sumFile = Join-Path $tmp "server.sha256"
        Start-Step "Downloading BlockVerse $Version"
        try {
            Invoke-WebRequest -Uri "$ReleaseBase/$Version/$file" -OutFile $archive -UseBasicParsing
            Invoke-WebRequest -Uri "$ReleaseBase/$Version/$file.sha256" -OutFile $sumFile -UseBasicParsing
        } catch {
            Stop-StepNote "no prebuilt file, building from source"
            return $false
        }
        Stop-Step
        Start-Step "Verifying download"
        $want = ((Get-Content -Path $sumFile -TotalCount 1) -split "\s+")[0].ToLower()
        $got = (Get-FileHash -Algorithm SHA256 -Path $archive).Hash.ToLower()
        if ($want -ne $got) { throw "checksum mismatch, the download is damaged or was modified" }
        Stop-Step
        Start-Step "Installing files"
        if (-not (Test-Command "tar")) {
            Stop-StepNote "tar is missing, building from source"
            return $false
        }
        $out = Join-Path $tmp "out"
        New-Item -ItemType Directory -Force -Path $out | Out-Null
        Invoke-Step "tar" @("-xzf", $archive, "-C", $out, "--strip-components=1") -AllowFail
        if ($script:LastCode -ne 0) {
            Stop-StepNote "unpack failed, building from source"
            return $false
        }
        Invoke-Step (Join-Path $out "blockverse.exe") @("--version") -AllowFail
        if ($script:LastCode -ne 0) {
            Stop-StepNote "the file does not run here, building from source"
            return $false
        }
        Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $Dir "blockverse.exe"), (Join-Path $Dir "start.bat")
        Copy-Item -Recurse -Force (Join-Path $out "*") $Dir
        Stop-Step
        return $true
    } finally {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmp
    }
}

function Build-FromSource {
    Start-Step "Installing prerequisites"
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
    Stop-Step

    Start-Step "Downloading source"
    New-Item -ItemType Directory -Force -Path $Src | Out-Null
    Set-Location $Src
    if (-not (Test-Path (Join-Path $Src ".git"))) { Invoke-Step "git" @("init", "-q") }
    Invoke-Step "git" @("remote", "remove", "origin") -AllowFail
    Invoke-Step "git" @("remote", "add", "origin", $RepoUrl)
    Invoke-Step "git" @("fetch", "-q", "--depth", "1", "origin", $Version)
    Invoke-Step "git" @("checkout", "-q", "-f", "FETCH_HEAD")
    Stop-Step

    Start-Step "Downloading dependencies"
    Invoke-Step "cargo" @("fetch", "--manifest-path", $Manifest)
    Stop-Step

    Invoke-Compile

    Start-Step "Installing files"
    Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $Dir "blockverse.exe"), (Join-Path $Dir "start.bat")
    Copy-Item -Force (Join-Path $Src "crates\bv-server\target\release\blockverse.exe") (Join-Path $Dir "blockverse.exe")
    Copy-Item -Force (Join-Path $Src "start.bat") (Join-Path $Dir "start.bat")
    foreach ($extra in @("LICENSE", "LICENSE-APACHE-2.0", "NOTICE", "PLUGIN-EXCEPTION.md")) {
        $path = Join-Path $Src $extra
        if (Test-Path $path) { Copy-Item -Force $path (Join-Path $Dir $extra) }
    }
    Stop-Step
}

New-Item -ItemType Directory -Force -Path $Dir | Out-Null
Set-Content -Path $Log -Value ""
Push-Location $Dir

try {
    if ($Version -notmatch '^[A-Za-z0-9][A-Za-z0-9._/-]*$') { throw "invalid version" }

    Write-Host "Preparing BlockVerse" -ForegroundColor White

    Start-Step "Checking system"
    if (-not [Environment]::Is64BitOperatingSystem) { throw "a 64-bit Windows is required" }
    Stop-Step

    $installed = $false
    if ($env:BV_FROM_SOURCE -ne "1" -and $Version -match '^v[0-9]') {
        $installed = Install-Prebuilt
    }
    if (-not $installed) { Build-FromSource }

    if ($env:BV_SERVER_NAME -or $env:BV_LANGUAGE) {
        Start-Step "Creating settings"
        $name = $env:BV_SERVER_NAME
        if (-not $name) { $name = "BlockVerse" }
        $language = $env:BV_LANGUAGE
        if (-not $language) { $language = "en" }
        New-Item -ItemType Directory -Force -Path (Join-Path $Dir "bvmc") | Out-Null
        Set-Location (Join-Path $Dir "bvmc")
        Invoke-Step (Join-Path $Dir "blockverse.exe") @("--init", "--server-name", $name, "--language", $language)
        Stop-Step
    }
    Set-Content -Path (Join-Path $Dir "VERSION") -Value $Version

    Write-Host "Done" -ForegroundColor Green
    Write-Host ("Start the server: " + (Join-Path $Dir "start.bat"))
}
catch {
    if ($script:StepOpen) { Write-Host "failed" -ForegroundColor Red }
    Write-Host ("BlockVerse install failed: " + $_.Exception.Message) -ForegroundColor Red
    Write-Host ("Log: " + $Log)
}
finally {
    Pop-Location
}
