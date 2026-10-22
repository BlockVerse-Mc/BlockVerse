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

Run on the same network as the server (`server-ip=0.0.0.0` in `bvmc/server.properties` when the client is another device) with `log-level=debug`.

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
