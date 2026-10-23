# Architecture

This describes what the source does today. Features that are not implemented are listed under Limits.

## Crates

| Crate | Role |
|---|---|
| `crates/bv-server` | The server, binary `blockverse`. Standalone cargo workspace with its own `Cargo.lock` because it pulls the protocol libraries from git |
| `crates/bv-plugin-abi` | Rust mirror of the plugin C ABI |
| `crates/bv-example-plugin` | Example native plugin |
| `crates/bv-skins` | Built-in default skins |
| `native/` | C ABI header, C++ plugin host prototype, C++ example plugins and tests |

External libraries (see [DEPENDENCIES.md](DEPENDENCIES.md)): `bedrock` (protocol, RakNet connection, compression) from BlockVerse-Protocol, `nbtx` from BlockVerse-NBT.

## Server modules (`crates/bv-server/src`)

| Module | Responsibility |
|---|---|
| `main.rs` | Start-up: CLI options, first-start setup, settings, logging, tokio runtime, listener, console, shutdown |
| `cli.rs`, `setup.rs` | `--version`, `--init`, `--help`; the first-start questions |
| `config.rs` | Reads and writes `server.properties`, validates every value and reports warnings |
| `logger.rs`, `lang.rs` | Log levels, colored console and file output; console messages in 14 languages (`lang/*.properties`) |
| `console.rs` | Console commands (`help`, `status`, `list`, `debug`, `stop`, ...) |
| `shared.rs` | State shared by all connections: players, settings, metrics, login slots |
| `server.rs` | One connection from RakNet accept to disconnect: login, session loop, world sending |
| `world.rs` | The flat test world: block hashes, chunk and biome bytes, test variants |
| `identity.rs` | Reads the player display name from the login data |
| `limits.rs`, `metrics.rs` | Packet rate limiter; counters and loop-lag monitor |
| `plugins.rs`, `manifest.rs` | Loading native plugins through the C ABI |
| `selftest.rs` | Internal checks run by the `selftest` command |

## Connection flow (`server.rs`)

1. RakNet connection accepted by the listener in `main.rs`; refused when the server is full or too many logins are in progress.
2. `RequestNetworkSettings`: the client protocol must equal 766, otherwise the connection is closed with a log line.
3. `NetworkSettings` (compression algorithm none, threshold 1).
4. `Login`: the display name is read from the login data (offline mode, nothing is verified).
5. `PlayStatus(LoginSuccess)`, `ResourcePacksInfo` and `ResourcePackStack` in one batch; two client replies are awaited.
6. `StartGame`, then `BiomeDefinitionList`, `AvailableActorIdentifiers` and `CreativeContent` (all three empty).
7. Session loop. On `RequestChunkRadius` the server sends `ChunkRadiusUpdated`, `NetworkChunkPublisherUpdate`, `LevelChunk` batches of 16 and `PlayStatus(PlayerSpawn)`, depending on `world-stage`. `SetLocalPlayerAsInitialized` logs "finished loading".
8. Every outbound packet goes through the journal in `server.rs`, which logs it and keeps the last entries for the disconnect report.

## Limits

One flat world in memory; no saving, entities, chat or movement handling; offline mode only; protocol 766 only. See the README for the user-facing list.
