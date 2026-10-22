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

1. Prints `Preparing BlockVerse`, then one line per stage with colors. Details go to `install.log`.
2. Asks nothing: the server asks for the language and the name the first time it starts.
3. For a version tag (`v1.0.0`): downloads the prebuilt `blockverse-<version>-<target>.tar.gz` from the GitHub release and verifies its SHA-256 file. A damaged or modified file stops the install. Without a prebuilt file for the device it builds from source.
4. For `main`, or when no prebuilt file exists: fetches the source from `BlockVerse-Mc/BlockVerse` and builds it. Termux installs `git` and `rust` with `pkg`; Linux and macOS need `git` and a C compiler and install Rust with rustup when it is missing or older than 1.85; Windows needs Git, Rust and the Visual Studio C++ Build Tools. A build takes 10 to 30 minutes on a phone.
5. Shows a live bar while compiling, for example `Compiling BlockVerse [###########.............] %46` (an estimate from the number of crates), and ends with `Done` and the path of `start.sh`.
6. Installs into `~/blockverse-server` (Windows: `%USERPROFILE%\blockverse-server`) with `start.sh` (Windows: `start.bat`), creates `bvmc/server.properties` and writes `VERSION`.

Targets with prebuilt files: `android-aarch64` (Termux), `linux-x86_64`, `linux-aarch64`, `windows-x86_64`. Windows and Android files are built experimentally by the release workflow.

## First start

When `bvmc/server.properties` does not exist and the server runs in a terminal, it asks for the language (a numbered list or a code) and the server name, writes the file and starts. Without a terminal (service, Docker) it uses the defaults. `BV_NO_SETUP=1` skips the questions.

## Start

```
cd ~/blockverse-server && ./start.sh
```

Settings, logs and plugins are in `bvmc/`. Arguments of `start.sh` go to the server: `./start.sh --version`, `./start.sh --help`.

## Environment variables

`BV_DIR` (install folder), `BV_SERVER_NAME` and `BV_LANGUAGE` (create `server.properties` during the install so the server does not ask), `NO_COLOR=1` (no colors), `BV_FROM_SOURCE=1` (never use a prebuilt file), `BV_JOBS` (build jobs on Termux, default 2), `BV_REPO_SLUG`, `BV_REPO_URL`, `BV_RELEASE_BASE` (testing).

## Update

Run the same command again. Settings, logs, plugins and worlds in `bvmc/` are kept.

## Command-line options of the server

`./blockverse --version`, `./blockverse --init [--server-name NAME] [--language CODE]` (never overwrites an existing `server.properties`; codes: see `--help`), `./blockverse --help`.

## Maintainers

Publishing a release: [RELEASING.md](RELEASING.md).
