@echo off
setlocal
cd /d "%~dp0"
set "BIN="
set "DATA=%CD%"
if exist blockverse.exe set "BIN=%CD%\blockverse.exe"
if not defined BIN if exist crates\bv-server\target\release\blockverse.exe set "BIN=%CD%\crates\bv-server\target\release\blockverse.exe" & set "DATA=%CD%\bvmc"
if not defined BIN goto missing
if not exist "%DATA%" mkdir "%DATA%"
cd /d "%DATA%"
"%BIN%" %*
set "CODE=%ERRORLEVEL%"
if not "%CODE%"=="0" pause
exit /b %CODE%
:missing
echo BlockVerse is not built. Run: cd crates\bv-server ^&^& cargo build --release
pause
exit /b 1
