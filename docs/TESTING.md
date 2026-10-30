# Testing

## Automated (no game client needed)

| Check | Command | Where it runs |
|---|---|---|
| Unit tests of the server | `cd crates/bv-server && cargo test` | CI, phone |
| Tests of the library crates | `cargo test --workspace` | CI |
| Native plugin host (C++) | `make -C native test` | CI |
| Installer logic (fake toolchain, local git, checksum refusal) | `sh scripts/test-installer.sh` | CI, release workflow |
| Smoke test of the console and a Rust plugin | `sh scripts/smoke-test.sh` | CI |
| Shell syntax and ShellCheck | `.github/workflows/ci.yml` | CI |

Every push runs these on GitHub Actions. Windows is built as an extra job that may fail without blocking.

## Real client: Minecraft Bedrock 1.21.50 (protocol 766)

Run on the same network as the server (`server-ip=0.0.0.0` in `server.properties` when the client is another device) with `log-level=debug`.

| # | Step | Pass |
|---|---|---|
| 1 | Add the server and join | The log shows the player name and `Player N finished loading` |
| 2 | Chunk test: `world-stage=2`, `chunk-variant=auto`, connect 4 times | Note the result of each variant (0 to 3) and the client message |
| 3 | `world-stage=3` with the best variant | The flat world is visible, the client stays connected for 60 seconds |
| 4 | Disconnect and join again | The second login works |
| 5 | Two devices at once | Both stay connected |
| 6 | `stop` in the console, or Ctrl+C | The log ends with the goodbye line and the process exits |
| 7 | Join with 1.21.50 while the server is full (`max-players=1`) | The second client is refused, the first stays |

After a test run `sh scripts/report.sh` bundles settings and logs into one file.

Record the result of each run (date, step, variant, pass or fail, client message) in the release issue.

## Newest stable client (not a beta)

BlockVerse 1.0.0 speaks protocol 766 only. With a newer client:

1. The client cannot join; the server log must show `client protocol N is not supported` and keep running.
2. Supporting the newer client needs its protocol number (from the official protocol documentation of that exact version), a matching protocol module in `BlockVerse-Protocol`, and a second `Protocol` type in `crates/bv-server/src/server.rs` with the packet differences of that version.
3. After that, run the whole table above again with the newer client.

Both client versions stay in the table. Beta and preview clients are not part of the release checks.

## Finding the packet the client rejects ("Server sent broken packet")

The server sends everything and the client disconnects itself, so the server log cannot name the bad packet. Narrow it down by changing only one thing per run. Use `log-level=trace`.

| Run | Settings | Packets sent after StartGame | If the client still shows the error |
|---|---|---|---|
| A | `world-stage=0` | biome list, actor list, creative content, chunk radius | the cause is in the login packets, StartGame or the three extra packets |
| B | `world-stage=1` | A plus the chunk publisher update | the cause is the publisher update or earlier |
| C | `world-stage=2`, `chunk-variant=auto`, connect 4 times | B plus one chunk, payload variants 0 to 3 | compare the 4 variants (below) |
| D | like C with `BV_SKIP_EXTRA=1` in the environment | like C without the biome, actor and creative packets | tells whether those three packets matter |

For stages 0 and 1 the client waits on the loading screen. Wait 20 seconds and note whether the error appears.

Chunk payload variants: 0 full chunk, 1 no sub chunks, 2 no sub chunks and no biomes (only the border byte), 3 full chunk with explicit biomes. Reading the four results:

- Only variant 2 survives: the biome data or the biome definitions are suspect.
- Variants 1 and 2 survive but 0 and 3 do not: the sub chunk bytes are suspect.
- All four fail: the chunk payload is not the cause; look at the packets before it.
- All four survive: the earlier failures came from stage 3 (many chunks per batch or PlayerSpawn); run stage 3 with the surviving variant.

Start a run like this (Termux), then connect and write down the client message and the time until it appears:

```
sed -i 's/^log-level=.*/log-level=trace/; s/^world-stage=.*/world-stage=2/; s/^chunk-variant=.*/chunk-variant=auto/' server.properties && ./start.sh
```

### Automatic probe (`bvdev probe`)

`BV_PROBE=1 ./start.sh` runs the bisect by itself: every new connection drops different packets (stage 0 is forced) and the server prints `PROBE RESULT step N/6 dropping [...]: REJECTED by the client` or `the client stayed`. Join six times, wait 10 seconds each time, leave, type `stop`. Steps: 1 nothing dropped (control), 2 all three packets after StartGame, 3 BiomeDefinitionList, 4 AvailableActorIdentifiers, 5 CreativeContent, 6 ChunkRadiusUpdated. The step that turns REJECTED into "stayed" names the packet. `bvdev probe` does the install and the run, and saves the result lines in Downloads (`bv-probe.txt`).

### Dropping one packet type (`BV_DROP`)

`BV_DROP=Name1,Name2 ./start.sh` makes the server skip every outbound packet with one of those names (the log shows `BV_DROP: not sending Name`). With the variable set, packets are sent one by one instead of in batches. Use it to test whether the client needs a packet or rejects it, for example:

```
BV_DROP=BiomeDefinitionListPacket ./start.sh
BV_DROP=AvailableActorIdentifiersPacket ./start.sh
BV_DROP=CreativeContentPacket ./start.sh
BV_DROP=ChunkRadiusUpdatedPacket ./start.sh
```

Names are the ones shown in the `->` log lines. Do not drop `NetworkSettingsPacket`, `PlayStatusPacket` or `StartGamePacket`: the login cannot continue without them.

### Results so far

- Stage 3, variant 0: the client closes about 90 ms after the last chunk batch and `PlayStatus(PlayerSpawn)`.
- Stage 0 (no chunks, no publisher update, no spawn status): the client still closes 24 ms after `ChunkRadiusUpdated`. The chunk bytes, the chunk header, the batch size and `PlayerSpawn` were never sent in that run, so they are not the cause of this disconnect. The cause is in `StartGame`, one of the three packets after it, or `ChunkRadiusUpdated`.

### What the log shows

- `<- Name` lines: packets received from the client (first 60).
- `-> #N Name (+ms)` lines: packets sent, numbered, with the time since login started. Consecutive equal packets are grouped (`LevelChunkPacket x9`).
- `Chunk payload: ...` and, at trace level, `Chunk payload bytes: ...`: the exact bytes of the chunk data.
- At trace level the text form of every sent packet except chunks, to check each field value.
- `Player N sent PacketViolationWarningPacket ...`: the client's own report of the packet it found malformed (packet id and a context text). Look for it first.
- When the connection ends: `Connection of player N closed after X packets, T s (reason)` and `Last packets sent to player N: ...` with the last four sent packets. The reason is the transport error, or `the connection ended` when the client closed it. A `Player N sent DisconnectPacket ...` line appears if the client sent a disconnect packet.
