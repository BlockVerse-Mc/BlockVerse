# Engineering status

State of the project as verified so far. "Verified" means observed: a CI run, a test run or a real client. Everything else is listed as unverified.

## Goal
Open-source Minecraft Bedrock server (BlockVerse) in Rust. Plugins: Rust and C++ through a stable C ABI (prototype done), PHP through a bridge (planned). GitHub organization: BlockVerse-Mc. Telegram: https://t.me/BlockVerseMc. Branding in `assets/branding`. Not affiliated with Mojang.

## Decisions
- Licenses: core GPL-3.0-or-later with a plugin exception; ABI, SDK and examples Apache-2.0; skins CC BY 4.0 ([LICENSING.md](../LICENSING.md)).
- Protocol, NBT and RakNet libraries are hosted in BlockVerse-Mc ([DEPENDENCIES.md](DEPENDENCIES.md)). Original license files are kept. Mojang's protocol JSON is not used and must never be committed.
- Target client: Bedrock 1.21.50, protocol 766 only. Changing it means changing the `bedrock` features and the packet types in `server.rs`.
- Version 1.0.0 is gated by [RELEASING.md](RELEASING.md); the roadmap is in [ROADMAP.md](ROADMAP.md).

## Dependency pinning
`bedrock` is pinned to commit `fc06fe9` of the `blockverse-1.0` branch of BlockVerse-Protocol: the original revision `76c3c9a` plus two manifest changes that point its own dependencies at BlockVerse-NBT and BlockVerse-RakNet. Never use `branch = "main"`: newer revisions moved `bedrock::network::connection::Connection` and `listener::Listener`. `crates/bv-server/Cargo.lock` is committed and must stay; `nbtx` is locked there but not pinned with `rev` yet.

## Verified
- GitHub Actions on `main`: server unit tests, server build, smoke test, workspace tests, native C++ tests, installer tests and the Windows build passed (first run, commit `01bf971`). Format check, clippy and ShellCheck are non-blocking and reported failures.
- On the phone with a 1.21.50 client (earlier builds): build, tests, login, player name from the login data, StartGame accepted, join, chunk radius request.
- The Android abort was fixed by disabling heap tagging at start-up (`mallopt`); the cause was `nbtx` writing NBT (`BV_SELFTEST=1` checks it).
- Installer logic against a fake toolchain and a local git remote (`scripts/test-installer.sh`).

## Unverified
- The commits after `01bf971` (14 languages, first-start setup, installer progress, diagnostics): CI result pending.
- The real GitHub installer flow, the Windows scripts, the Windows and Android release jobs.
- The translations other than English and Persian have not been reviewed by native speakers.

## Open bug: "Server sent broken packet"
Observed (stage 3, chunk variant 0): the player joins, `Sent the world to player 1 (view distance 4)`, then the connection closes after 2 received packets and 3.20 s. Every send returned Ok, so the client rejected something it received. The client never sent `SetLocalPlayerAsInitialized`.

Not established: which packet is rejected. By reading the code, the chunk bytes follow the documented network layout (sub chunk version 9, 2-bit palette of block hashes, 24 biome sections, border byte) and the unit tests pin them; that does not prove client acceptance.

Evidence from stage 0 (nothing but `ChunkRadiusUpdated` sent after the client's request): the client still closes 24 ms later, after it asked for chunks 3.08 s after `StartGame` and sent `ServerBoundLoadingScreen`. So the chunk bytes, the `LevelChunk` header, the batch size and `PlayerSpawn` are not the cause of this disconnect.

Protocol documentation facts (Mojang's protocol reference, changelog of protocol 766): `ResourcePacksInfo` changed in 766 (world template id and version, pack ids as UUIDs; the server already uses the v766 type); the movement authority enum was renumbered (0 is now `LegacyClientAuthoritativeV1`; the server sends the old `ClientAuthoritative`, value 0); the client sends `PacketViolationWarning` (id 156) when it detects a malformed packet, and the server now logs it at info level when the debug log is on. Other sources describe the error as a packet the client could not parse; for custom servers that means a structure or value mismatch, not a network fault.

Open hypotheses (all packets come from the v662 definitions through the protocol 766 enum): `ChunkRadiusUpdated` (value or encoding); one of the three packets sent right after `StartGame` (empty biome definition list, empty actor identifier list, empty creative content), which the client may only handle after its loading phase; a `StartGame` field the client checks late. Method: the run table in [TESTING.md](TESTING.md) ("Finding the packet the client rejects"), the outbound journal and the receive-error log in `server.rs`. Needed evidence: the debug or trace log of runs A to D, and the definitions of `LevelChunkPacket` and `BiomeDefinitionListPacket` from the dependency checkout. Fallback plan: read a real flat world made by Minecraft with the `level` crate of BlockVerse-Protocol and take the palette and states from it.

## Configuration and console
`server.properties` in the server folder (documented in [CONFIGURATION.md](CONFIGURATION.md); the old `blockverse.properties` is read when it is the only file). Log levels error, warn, info, debug, trace. Console: help, version, status, list, plugins, worlds, skins, say, debug, save-all (reports that it is not implemented), fakejoin, fakechat, fakequit, selftest, stop. Settings wired to the protocol: gamemode, difficulty, level-name, level-seed, allow-cheats, default-player-permission-level, max-players, view-distance.

## Installer
`install.sh [version]` and `install.ps1 [version]` (default `latest`): installs into the current empty folder and creates no other folder; one colored line per stage; a verified prebuilt file of the newest release or of the rolling `nightly` build (made by `.github/workflows/nightly.yml`), a build from source as the fallback with the missing tools installed first and a live compile bar; `start.sh` or `start.bat` run the server in that folder. The server asks for language and name on its first start; `BV_SERVER_NAME` and `BV_LANGUAGE` create the settings during the install.

## Known limits
Offline mode (names are not verified); chat and in-game commands are not decoded; plugin `send_message` prints to the console; skins are chosen but not sent; nothing is saved.

## Security measures in place
Login timeout, limit on simultaneous logins, per-player packet rate limit, maximum players, local-only bind by default. The plugin folder is trusted code: native plugins can do anything.

## Next steps
1. Find and fix the chunk rejection. 2. PlayerSpawn and movement. 3. Chat and in-game commands. 4. Saving and loading. 5. Verified identity. 6. Plugin API v1 and the PHP bridge. 7. Split `server.rs` (login, session, world sending) once the chunk bug is closed.

## Progress estimate
About 55% toward a usable server (estimate).
