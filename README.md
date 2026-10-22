<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/branding/logo-wordmark-white.png">
    <img src="assets/branding/logo-wordmark-black.png" alt="BlockVerse" width="420">
  </picture>
</p>

# BlockVerse

Minecraft Bedrock server software written in Rust, with native plugin support.

Community: https://t.me/BlockVerseMc

Version 1.0.0 targets **Minecraft Bedrock 1.21.50 (protocol 766)**. Release checks are listed in [docs/RELEASING.md](docs/RELEASING.md) and the test plan in [docs/TESTING.md](docs/TESTING.md).

## Install

Linux, macOS and Termux, newest version:

```
curl -fsSL https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.sh | bash
```

A specific version:

```
curl -fsSL https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.sh | bash -s -- v1.0.0
```

Windows (PowerShell), newest version:

```
irm https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.ps1 | iex
```

A specific version:

```
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.ps1))) v1.0.0
```

Start the server:

```
cd ~/blockverse-server && ./start.sh
```

On Windows run `start.bat` in `%USERPROFILE%\blockverse-server`. Details: [docs/INSTALL.md](docs/INSTALL.md).

## Languages

The console speaks 14 languages: en, fa, ar, tr, ru, es, pt, fr, de, it, id, zh, ja, ko. The server asks for the language on the first start, or set `language=` in `server.properties`. Translations are in `crates/bv-server/lang/` and are checked by tests; native speakers are welcome to improve them ([docs/TRANSLATING.md](docs/TRANSLATING.md)).

## Supported clients

| Minecraft Bedrock | Protocol | Status |
|---|---|---|
| 1.21.50 | 766 | Supported target of 1.0.0 |
| Newest stable release | n/a | Not supported yet: the server refuses the connection with a clear message |

## Known limitations

- Offline mode only: player names are not verified (`online-mode=true` is refused).
- One flat world generated in memory: no saving, no entities, no terrain generation.
- Chat and in-game commands are not decoded; skins are chosen but not sent to the client.
- Plugin `send_message` prints to the server console.

## Layout

| Path | What |
|---|---|
| `crates/bv-server` | The server (binary `blockverse`) |
| `crates/bv-plugin-abi` | Rust mirror of the plugin C ABI |
| `crates/bv-example-plugin` | Example plugin in Rust |
| `crates/bv-skins` | Built-in default skins |
| `native/` | C ABI header, C++ plugin host prototype, C++ example plugin, tests |
| `install.sh`, `install.ps1`, `start.sh`, `start.bat` | Installers and run scripts |
| `scripts/` | Release packager, smoke test, installer test, report helper |
| `docs/` | Install, configuration, testing, releasing, plugin ABI and API drafts, roadmap |

## Dependencies

The protocol, NBT and RakNet libraries live in the BlockVerse-Mc organization: BlockVerse-Protocol, BlockVerse-NBT and BlockVerse-RakNet. See [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md).

## Run from a source checkout

```
cd crates/bv-server && cargo build --release
cd ../.. && ./start.sh
```

Settings, logs and plugins live in the `bvmc/` folder (`server.properties`, `logs/latest.log`, `plugins/`). See [docs/CONFIGURATION.md](docs/CONFIGURATION.md) for every setting and the console commands (`help`).
Run `sh scripts/report.sh` to bundle settings and logs into one file for bug reports.

## Test

```
cargo test --workspace
cd crates/bv-server && cargo test
make -C native test
sh scripts/test-installer.sh
sh scripts/smoke-test.sh
```

## Plugins

Plugins talk to the server through a stable C ABI (`native/include/blockverse_plugin.h`), so they can be written in Rust or C++. PHP plugins are planned through a separate bridge.

## License

Free and open source. Server core: GPL-3.0-or-later with a plugin exception. Plugin ABI, SDKs and examples: Apache-2.0. Skins: CC BY 4.0. See [LICENSING.md](LICENSING.md).

*BlockVerse is not an official Minecraft product and is not approved by or associated with Mojang.*
