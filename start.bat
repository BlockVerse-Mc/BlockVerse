@echo off
setlocal
cd /d "%~dp0"
if not exist bvmc mkdir bvmc
set "BIN="
if exist blockverse.exe set "BIN=%CD%\blockverse.exe"
if not defined BIN if exist crates\bv-server\target\release\blockverse.exe set "BIN=%CD%\crates\bv-server\target\release\blockverse.exe"
if not defined BIN if exist crates\bv-server\target\debug\blockverse.exe set "BIN=%CD%\crates\bv-server\target\debug\blockverse.exe"
if not defined BIN goto missing
cd bvmc
"%BIN%" %*
set "CODE=%ERRORLEVEL%"
if not "%CODE%"=="0" pause
exit /b %CODE%
:missing
echo BlockVerse is not built. Run: cd crates\bv-server ^&^& cargo build --release
pause
exit /b 1
