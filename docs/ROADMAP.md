# Roadmap

## Versions
| Version | Meaning |
|---|---|
| Connection Test 0 | Finished. Console server, login handshake, native plugin host, logging, config, tests |
| Beta 0.1 | In development. Playable core: chunks, movement, chat, in-game commands, saved player data |
| Beta 1.0 | Plugin API v1, official SDKs, storage, permissions, documentation |

## Connection Test 0 (done in code, verify on device)
- [x] Logging, settings file, graceful shutdown
- [x] Console commands, player registry, default skins
- [x] Native plugin host (Rust and C++ through the C ABI)
- [x] Login timeout, status and loop-lag metrics
- [x] Plugin manifest language rules (pure logic, tested)
- [ ] Real client connects and stays connected (needs a Minecraft 1.21.50 client)

## Beta 0.1
- [x] Player name from the login data (offline mode, unit-tested on synthetic tokens; verify with a real client)
- [x] `debug=true` logs the first 60 packets a client sends
- [ ] Verified identity (online mode)
- [x] Login timeout, simultaneous login limit, packet rate limit, log files (`logs/`)
- [x] Flat world chunks sent to the client (written, unit-tested against a python mirror; needs a real client test)
- [ ] Load a saved world
- [ ] Movement and rotation, block break and place
- [ ] Chat packets (receive, broadcast), in-game commands
- [ ] Inventory and items (item stacks, containers, creative inventory): needed before player items can be saved
- [ ] Save and load: world chunks and changes, player position and inventory (own format first)
- [x] `server.properties` with validated settings, five log levels, more console commands
- [ ] Rate limits and input validation, multi-player stress test

## Beta 1.0
- [ ] Manifest loader (`plugin.json`) and plugin lifecycle (load, enable, disable)
- [ ] Event system with cancellable events, scheduler on the main thread
- [ ] Storage API (key-value first), permissions, plugin config
- [ ] PHP bridge, Rust SDK, C++ SDK, sample plugins
- [ ] API specification, docs, installer, GitHub Releases

## Performance ideas
- Keep the main loop light; measure loop lag with `status` before and after each feature.
- Batch outgoing packets per tick; compress only above a size threshold on public servers.
- Run plugin callbacks on a dedicated thread or queue so a slow plugin cannot stall networking.
- Limit view distance and chunk sends per tick; load and save chunks off the main thread.
- Shipped files are built in CI with `cargo build --profile dist` (thin LTO, one codegen unit); installs from source use the faster `release` profile.
- Benchmark against other servers under the same hardware, players, plugins and world.
