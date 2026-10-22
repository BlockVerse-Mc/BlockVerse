# Dependencies hosted in BlockVerse-Mc

BlockVerse uses three libraries that are maintained in the `BlockVerse-Mc` GitHub organization, so builds do not depend on any other account.

| Repository | Cargo package | Used for |
|---|---|---|
| `BlockVerse-Protocol` | `bedrock` | Protocol packets, RakNet connection, compression |
| `BlockVerse-NBT` | `nbtx` | NBT writer |
| `BlockVerse-RakNet` | `raknet-tokio` | RakNet transport (used by BlockVerse-Protocol) |

`crates/bv-server/Cargo.toml` points `bedrock` and `nbtx` at these repositories. `BlockVerse-Protocol` points at the other two in its own manifests, so there is one source for each library in the build.

## Rules

- The repositories are public: the installer, CI and every contributor fetch them without a login.
- Keep every `LICENSE` and notice file of the original projects in these repositories (their licenses require it) and list changes in `NOTICE`.
- `bedrock` is pinned to the commit `76c3c9a`, the revision that built and ran on the test phone. Never use `branch = "main"` for it: newer revisions moved `bedrock::network::connection::Connection` and `listener::Listener`.
- Commit `crates/bv-server/Cargo.lock`. Losing it makes cargo pick newer dependency versions and the release workflow refuses a tag without it.
- After the first successful build, pin `nbtx` with `rev` to the commit recorded in `Cargo.lock`.

## Changing a dependency

1. Change the code in the dependency repository and push it to its `main` branch.
2. In `crates/bv-server/Cargo.toml` set `rev` to the new commit.
3. Run `cargo update -p bedrock`, then `cargo test` and the real client test in [TESTING.md](TESTING.md).
4. Commit `Cargo.toml` and `Cargo.lock` together.
