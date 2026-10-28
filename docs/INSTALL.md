# Installing BlockVerse

## Before you start

Create an empty folder for the server, go into it and run the command there. The installer puts the files in that folder, creates no other folder and refuses to run in your home folder or in a folder that already holds other files.

```
mkdir bvmc-sv
cd bvmc-sv
```

## One command

Linux, macOS and Termux (newest version):

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

`BV_VERSION=v1.0.0` works as well. Read a script before piping it into a shell.

## Versions

| Argument | What is installed |
|---|---|
| none or `latest` | The newest release; while there is no release, the rolling build of `main` |
| `nightly` | The rolling build of the newest commit on `main` |
| `v1.0.0` | That release |
| `main` or a branch | Built from source |

Rolling builds and releases are prebuilt by GitHub Actions, so nothing is compiled on your device and no compiler is needed. A build from source is the fallback when no prebuilt file exists for the device.

## What the installer does

1. Prints `Preparing BlockVerse`, then one colored line per stage.
2. Downloads the prebuilt file and checks its SHA-256 file. A damaged or modified file stops the install.
3. When it has to build from source it first installs what is missing: on Termux `git`, `clang` and `rust` through `pkg`; on Linux `git`, `curl` and a compiler through the system package manager (it asks for your password with `sudo` when needed) and Rust through rustup (into `~/.cargo`); on macOS it asks for Apple's Command Line Tools; on Windows Git, the Visual Studio C++ Build Tools and Rust through winget and rustup. Then it downloads the source and the dependencies, shows a live bar while compiling, for example `Compiling BlockVerse [###########.............] %46` (an estimate from the number of crates), and installs the result.
4. Works in a temporary folder that is deleted at the end. A failed install leaves only `install.log` in your folder.
5. Ends with `Done` and `./start.sh`.

Prebuilt targets: `android-aarch64` (Termux), `linux-x86_64`, `linux-aarch64`, `windows-x86_64`. The Android file needs Android 8 or newer (API 26) and, like the Windows file, is experimental. macOS builds from source.

## First start

```
./start.sh
```

The server runs inside the same folder. When `server.properties` does not exist and the server runs in a terminal, it asks for the language (a numbered list or a code) and the server name, writes the file and starts. Without a terminal (service, Docker) it uses the defaults. `BV_NO_SETUP=1` skips the questions. The server creates its own `logs/` and `plugins/` folders and, later, the world folders inside this folder.

Arguments of `start.sh` go to the server: `./start.sh --version`, `./start.sh --help`.

## Files in the folder

| File | What |
|---|---|
| `blockverse` (`blockverse.exe`) | The server |
| `start.sh` (`start.bat`) | Starts the server inside this folder |
| `VERSION` | The installed version |
| `LICENSE`, `LICENSE-APACHE-2.0`, `NOTICE`, `PLUGIN-EXCEPTION.md` | License texts |
| `server.properties` | Settings, created on the first start |

## Environment variables

`BV_SERVER_NAME` and `BV_LANGUAGE` (create `server.properties` during the install so the server does not ask), `BV_FROM_SOURCE=1` (never use a prebuilt file), `BV_FORCE=1` (install into a folder that holds other files), `BV_JOBS` (build jobs on Termux, default 2), `NO_COLOR=1` (no colors), `BV_REPO_SLUG`, `BV_REPO_URL`, `BV_RELEASE_BASE`, `BV_API_BASE` (testing).

## Update

Run the same command again inside the same folder. Settings, logs, plugins and worlds are kept.

## Command-line options of the server

`./blockverse --version`, `./blockverse --init [--server-name NAME] [--language CODE]` (never overwrites an existing `server.properties`; codes: see `--help`), `./blockverse --help`.

## Maintainers

Publishing a release: [RELEASING.md](RELEASING.md).
