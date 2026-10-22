# Plugin API (draft)

Status: design only. Nothing here except the native ABI (see PLUGIN-ABI.md) is implemented yet.

## Manifest `plugin.json`
Fields: `name`, `version`, `api` (ABI version), `main`, `language` (`php`, `rust`, `cpp`, `native`, optional).
- If `language` is set it must match the `main` file type; a mismatch is an error, never a guess.
- If `language` is missing, it is detected from the extension of `main`.
- `.rs` files are source code and are never loaded; compile them to a native library first.
- A `.so`, `.dll` or `.dylib` is a native library; the extension does not tell Rust from C++.
The rules are implemented and tested in `crates/bv-server/src/manifest.rs`.

## Lifecycle
`onLoad` -> `onEnable` -> `onDisable`. Errors in one plugin must not stop the server.

## Events (first set)
PlayerJoin, PlayerQuit, PlayerChat (cancellable), later PlayerMove, BlockBreak, BlockPlace, ServerStart, ServerStop.
An event is only available once the core really implements the behavior behind it.

## Threading
World and inventory changes happen on the main server thread only. Async work reports back to it.

## Rollout order
1. Plugin, Server, Player, Logger, Config, Event, Command
2. World, Block, Item, Inventory, Entity, Scheduler
3. Storage, Permission, Economy, Team, game frameworks
4. PHP, Rust and C++ SDKs, sample projects, docs, compatibility tests

Names and behavior should match across languages; implementations may differ.
