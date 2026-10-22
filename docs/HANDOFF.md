# BlockVerse - project handoff

Paste this file into a new chat with Claude to continue exactly where we stopped.

## Goal
Open-source Minecraft Bedrock server (BlockVerse, also called BlockWare) in Rust. Plugins: Rust and C++ through a stable C ABI (done, prototype), PHP through a bridge (planned). GitHub org: BlockVerse-Mc. Team: Stars. Telegram channel: https://t.me/BlockVerseMc. Branding in `assets/branding` (icon and wordmark in black and white on transparent, avatars, favicon). Not affiliated with Mojang.

## Decisions
- Licenses: core GPL-3.0-or-later + Plugin Exception; ABI, SDK, examples Apache-2.0; skins CC BY 4.0 (LICENSING.md).
- Built on the BlockVerse-Mc libraries BlockVerse-Protocol, BlockVerse-NBT and BlockVerse-RakNet (Apache-2.0 code, original license files kept): protocol, RakNet, compression, auth, level. Mojang's protocol JSON is not used and must never be committed.
- Dev target: Bedrock 1.21.50, protocol 766 (the user's client). Switching means changing the bedrock feature and the types in `server.rs`.
- Versions: Connection Test 0 (done) -> Beta 0.1 (in development) -> Beta 1.0. The user asked for a 1.0.0 release for client 1.21.50; it is gated by docs/RELEASING.md.

## Layout
`crates/bv-server` (binary `blockverse`; standalone cargo workspace), `crates/bv-plugin-abi`, `crates/bv-example-plugin`, `crates/bv-skins`, `native/`, `scripts/`, `docs/`, `tools/skins`. Runtime folder: `bvmc/`.

## Dependency pinning
BlockVerse-Protocol is pinned to commit `76c3c9a` (the revision that built and ran on the phone). Newer `main` moved `bedrock::network::connection::Connection` and `listener::Listener`, so never use `branch = "main"`. Keep `crates/bv-server/Cargo.lock` (binary crate): losing it makes cargo pick newer dependency versions. The `nbtx` git dependency is not pinned yet; find its revision with `ls ~/.cargo/git/checkouts/nbtx-*/` and pin it with `rev`.

## Environment
- User: phone with Termux (Rust installed), 4 GB RAM; project in `~/blockverse`. Builds take 2 to 20 minutes.
- Claude's sandbox has no internet, Rust or PHP (g++ and Python only); Rust code is verified only when the user builds it.
- No remote access to the phone: use `sh scripts/report.sh` (writes `blockverse-report.txt` to Downloads) and send the file.

## Verified on the phone with a real 1.21.50 client
- Build, tests, login, player name from the login data, StartGame accepted, join, chunk radius request.
- Android abort fixed by disabling heap tagging at startup (`mallopt`); cause was `nbtx` writing NBT (selftest: `BV_SELFTEST=1`).

## Open problem
The client answers "Server sent broken packet" when the chunk content arrives. Test with `bvmc/server.properties`: `debug=true`, `world-stage=2`, `chunk-variant=auto` (each connection uses the next variant 0..3; the log shows which). Suspects: empty BiomeDefinitionList (biome id 1 unresolved), biome section format, sub chunk format, block states of bedrock/stone/grass_block. Plan B: read a real flat world made by Minecraft with the `level` crate of BlockVerse-Protocol and take palette and states from it.

## Configuration and commands (step 25)
`bvmc/server.properties` (standard name, documented in docs/CONFIGURATION.md; the old `blockverse.properties` is still read if it is the only file). Log levels error/warn/info/debug/trace with `log-level` and `debug level <x>`. Console: help, version, status, list, plugins, worlds, skins, say, debug (level, network, chunks, plugins), save-all (honest "not implemented"), fakejoin/fakechat/fakequit, selftest, stop. Settings that are wired to the protocol: gamemode, difficulty, level-name, level-seed, allow-cheats, default-player-permission-level, max-players, view-distance. `docs/DEPENDENCIES.md` lists the three libraries hosted in BlockVerse-Mc.

## Installer
`install.sh [version]` (default `main`) and `install.ps1 [version]` print only `Preparing BlockVerse`, keep details in `install.log`, ask for server name and language when a terminal exists, install a verified prebuilt file for a tag (`v1.0.0`) or build from source, and leave `start.sh` or `start.bat` in `~/blockverse-server`; the data folder is `bvmc/` inside it. `scripts/test-installer.sh` tests the logic with a fake toolchain and a local git remote (passes). Unverified: the real GitHub flow, the Windows scripts, the tty prompts, the Windows and Android release jobs. Language (`language=en|fa`) translates only the main console messages.

## Chosen next batches (user selection)
1. Chunk test with `chunk-variant=auto` (still open). 2. `plugin.json` loader and enable/disable lifecycle. 3. `bv-api` and `bv-network` crates (clean architecture). 4. PHP and Python bridges (planned as separate processes talking to the server). 5. Own forks of the dependencies.

## Known limits
Offline mode (names are not verified), chat and commands in game not decoded, plugin `send_message` prints to the console, skins are chosen but not sent, no saving.

## Security measures in place
Login timeout, limit on simultaneous logins, per-player packet rate limit, max players, local-only bind by default, plugin folder is trusted code (native plugins can do anything).

## Next steps
1. Fix chunks (above). 2. PlayerSpawn and movement. 3. Chat packets and in-game commands. 4. Save and load. 5. Verified identity. 6. Plugin API v1 and PHP bridge.

## Progress estimate
About 48% toward a usable server.
