$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$Version = "latest"
if ($args.Count -gt 0 -and $args[0]) { $Version = [string]$args[0] } elseif ($env:BV_VERSION) { $Version = $env:BV_VERSION }

if ($env:BV_REPO_SLUG) { $Slug = $env:BV_REPO_SLUG } else { $Slug = "BlockVerse-Mc/BlockVerse" }
if ($env:BV_REPO_URL) { $RepoUrl = $env:BV_REPO_URL } else { $RepoUrl = "https://github.com/$Slug.git" }
if ($env:BV_RELEASE_BASE) { $ReleaseBase = $env:BV_RELEASE_BASE } else { $ReleaseBase = "https://github.com/$Slug/releases/download" }
if ($env:BV_API_BASE) { $ApiBase = $env:BV_API_BASE } else { $ApiBase = "https://api.github.com" }
if ($env:BV_DIR) { $Dir = $env:BV_DIR } else { $Dir = (Get-Location).Path }
$Manifest = "crates\bv-server\Cargo.toml"
$Work = Join-Path ([System.IO.Path]::GetTempPath()) ("blockverse-" + [guid]::NewGuid().ToString("N"))
$Log = Join-Path $Work "install.log"
$BuildLog = Join-Path $Work "build.log"
$script:LastCode = 0
$script:StepOpen = $false
$script:Tag = ""
$script:Asset = ""
$script:SourceRef = ""
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

function Test-Folder {
    $trimmed = $Dir.TrimEnd("\")
    if ($trimmed -ieq $env:USERPROFILE.TrimEnd("\") -or $trimmed.Length -le 2) {
        throw "run this inside an empty folder you created for the server, not in $Dir"
    }
    if ($env:BV_FORCE -eq "1" -or (Test-Path (Join-Path $Dir "blockverse.exe"))) { return }
    $others = @(Get-ChildItem -Force -Path $Dir | Where-Object { $_.Name -ne "install.log" })
    if ($others.Count -gt 0) {
        throw "this folder is not empty. Create a new empty folder, go into it and run the command again"
    }
}

function Get-LatestTag {
    try {
        $release = Invoke-RestMethod -Uri "$ApiBase/repos/$Slug/releases/latest" -Headers @{ "User-Agent" = "blockverse-installer" }
        if ($release.tag_name) { return [string]$release.tag_name }
    } catch { }
    return ""
}

function Select-Channel {
    if ($Version -eq "latest") {
        $found = Get-LatestTag
        if ($found) {
            $script:Tag = $found
            $script:Asset = $found.TrimStart("v")
        } else {
            $script:Tag = "nightly"
            $script:Asset = "nightly"
        }
        $script:SourceRef = "main"
    } elseif ($Version -eq "nightly") {
        $script:Tag = "nightly"
        $script:Asset = "nightly"
        $script:SourceRef = "main"
    } elseif ($Version -match '^v[0-9]') {
        $script:Tag = $Version
        $script:Asset = $Version.TrimStart("v")
        $script:SourceRef = $Version
    } else {
        $script:SourceRef = $Version
    }
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
    $file = "blockverse-" + $script:Asset + "-windows-x86_64.tar.gz"
    $tmp = Join-Path $Work "download"
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    $archive = Join-Path $tmp "server.tar.gz"
    $sumFile = Join-Path $tmp "server.sha256"
    Start-Step ("Downloading BlockVerse " + $script:Tag)
    try {
        Invoke-WebRequest -Uri "$ReleaseBase/$($script:Tag)/$file" -OutFile $archive -UseBasicParsing
        Invoke-WebRequest -Uri "$ReleaseBase/$($script:Tag)/$file.sha256" -OutFile $sumFile -UseBasicParsing
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
}

function Test-BuildTools {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) { return $false }
    $found = & $vswhere -products "*" -requires "Microsoft.VisualStudio.Component.VC.Tools.x86.x64" -latest -property installationPath 2>$null
    return [bool]$found
}

function Build-FromSource {
    $src = Join-Path $Work "source"
    Start-Step "Installing prerequisites"
    Update-SessionPath
    if (-not (Test-Command "git")) {
        if (-not (Test-Command "winget")) { throw "git is required" }
        Invoke-Step "winget" @("install", "-e", "--id", "Git.Git", "--silent", "--accept-package-agreements", "--accept-source-agreements")
        Update-SessionPath
        if (-not (Test-Command "git")) { throw "git is required" }
    }
    if (-not (Test-BuildTools)) {
        if (-not (Test-Command "winget")) { throw "the Visual Studio C++ Build Tools are required" }
        Invoke-Step "winget" @("install", "-e", "--id", "Microsoft.VisualStudio.2022.BuildTools", "--silent", "--accept-package-agreements", "--accept-source-agreements", "--override", "--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended")
        if (-not (Test-BuildTools)) { throw "the Visual Studio C++ Build Tools could not be installed" }
    }
    if (-not (Test-Command "cargo")) {
        $init = Join-Path $Work "rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $init -UseBasicParsing
        Invoke-Step $init @("-y", "--profile", "minimal")
        Update-SessionPath
        if (-not (Test-Command "cargo")) { throw "Rust is required" }
    }
    Stop-Step

    Start-Step "Downloading source"
    New-Item -ItemType Directory -Force -Path $src | Out-Null
    Set-Location $src
    Invoke-Step "git" @("init", "-q")
    Invoke-Step "git" @("remote", "add", "origin", $RepoUrl)
    Invoke-Step "git" @("fetch", "-q", "--depth", "1", "origin", $script:SourceRef)
    Invoke-Step "git" @("checkout", "-q", "-f", "FETCH_HEAD")
    Stop-Step

    Start-Step "Downloading dependencies"
    Invoke-Step "cargo" @("fetch", "--manifest-path", $Manifest)
    Stop-Step

    Invoke-Compile

    Start-Step "Installing files"
    Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $Dir "blockverse.exe"), (Join-Path $Dir "start.bat")
    Copy-Item -Force (Join-Path $src "crates\bv-server\target\release\blockverse.exe") (Join-Path $Dir "blockverse.exe")
    Copy-Item -Force (Join-Path $src "start.bat") (Join-Path $Dir "start.bat")
    foreach ($extra in @("LICENSE", "LICENSE-APACHE-2.0", "NOTICE", "PLUGIN-EXCEPTION.md")) {
        $path = Join-Path $src $extra
        if (Test-Path $path) { Copy-Item -Force $path (Join-Path $Dir $extra) }
    }
    Stop-Step
}

Push-Location $Dir
try {
    if ($Version -notmatch '^[A-Za-z0-9][A-Za-z0-9._/-]*$') { throw "invalid version" }
    Test-Folder
    New-Item -ItemType Directory -Force -Path $Work | Out-Null
    Set-Content -Path $Log -Value ""

    Write-Host "Preparing BlockVerse" -ForegroundColor White

    Start-Step "Checking system"
    if (-not [Environment]::Is64BitOperatingSystem) { throw "a 64-bit Windows is required" }
    Select-Channel
    Stop-Step

    $installed = $false
    if ($script:Tag -and $env:BV_FROM_SOURCE -ne "1") {
        $installed = Install-Prebuilt
    }
    if (-not $installed) { Build-FromSource }

    Set-Location $Dir
    if ($env:BV_SERVER_NAME -or $env:BV_LANGUAGE) {
        Start-Step "Creating settings"
        $name = $env:BV_SERVER_NAME
        if (-not $name) { $name = "BlockVerse" }
        $language = $env:BV_LANGUAGE
        if (-not $language) { $language = "en" }
        Invoke-Step (Join-Path $Dir "blockverse.exe") @("--init", "--server-name", $name, "--language", $language)
        Stop-Step
    }
    if ($script:Tag) { Set-Content -Path (Join-Path $Dir "VERSION") -Value $script:Tag } else { Set-Content -Path (Join-Path $Dir "VERSION") -Value $script:SourceRef }

    Write-Host "Done" -ForegroundColor Green
    Write-Host "Start the server: .\start.bat"
}
catch {
    if ($script:StepOpen) { Write-Host "failed" -ForegroundColor Red }
    Write-Host ("BlockVerse install failed: " + $_.Exception.Message) -ForegroundColor Red
    if (Test-Path $Log) {
        Copy-Item -Force $Log (Join-Path $Dir "install.log") -ErrorAction SilentlyContinue
        Write-Host ("Log: " + (Join-Path $Dir "install.log"))
    }
}
finally {
    Pop-Location
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $Work
}
