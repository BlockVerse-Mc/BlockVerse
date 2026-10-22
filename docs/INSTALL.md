# Installing BlockVerse

## One command

Linux, macOS and Termux (newest version, built from the `main` branch):

```
curl -fsSL https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.sh | bash
```

A specific version (a release tag):

```
curl -fsSL https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.sh | bash -s -- v1.0.0
```

Windows (PowerShell):

```
irm https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.ps1 | iex
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.ps1))) v1.0.0
```

The first line installs the newest version, the second a specific one. `BV_VERSION=v1.0.0` works as well.

Read a script before piping it into a shell.

## What the installer does

1. Prints `Preparing BlockVerse` and keeps all details in `install.log`.
2. Asks for the server name and the language (`en` or `fa`) when a terminal is available.
3. For a version tag (`v1.0.0`): downloads the prebuilt `blockverse-<version>-<target>.tar.gz` from the GitHub release and verifies its SHA-256 file. A damaged or modified file stops the install. Without a prebuilt file for the device it builds from source.
4. For `main`, or when no prebuilt file exists: fetches the source from `BlockVerse-Mc/BlockVerse` and builds it. Termux installs `git` and `rust` with `pkg`; Linux and macOS need `git` and a C compiler and install Rust with rustup when it is missing or older than 1.85; Windows needs Git, Rust and the Visual Studio C++ Build Tools. A build takes 10 to 30 minutes on a phone.
5. Installs into `~/blockverse-server` (Windows: `%USERPROFILE%\blockverse-server`) with `start.sh` (Windows: `start.bat`), creates `bvmc/server.properties` and writes `VERSION`.

Targets with prebuilt files: `android-aarch64` (Termux), `linux-x86_64`, `linux-aarch64`, `windows-x86_64`. Windows and Android files are built experimentally by the release workflow.

## Start

```
cd ~/blockverse-server && ./start.sh
```

Settings, logs and plugins are in `bvmc/`. Arguments of `start.sh` go to the server: `./start.sh --version`, `./start.sh --help`.

## Environment variables

`BV_DIR` (install folder), `BV_SERVER_NAME` and `BV_LANGUAGE` (skip the questions), `BV_FROM_SOURCE=1` (never use a prebuilt file), `BV_JOBS` (build jobs on Termux, default 2), `BV_REPO_SLUG`, `BV_REPO_URL`, `BV_RELEASE_BASE` (testing).

## Update

Run the same command again. Settings, logs, plugins and worlds in `bvmc/` are kept.

## Command-line options of the server

`./blockverse --version`, `./blockverse --init [--server-name NAME] [--language en|fa]` (never overwrites an existing `server.properties`), `./blockverse --help`.

## Maintainers

Publishing a release: [RELEASING.md](RELEASING.md).
