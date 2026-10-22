# Releasing

## Gate (all must be true before the tag)

- [ ] CI is green on `main` (workspace, server, native, scripts).
- [ ] `crates/bv-server/Cargo.lock` is committed.
- [ ] The real client checklist in [TESTING.md](TESTING.md) passed on Minecraft 1.21.50: steps 1 to 4 and 6 at least.
- [ ] `chunk-variant` in the default settings is a fixed variant (the one that passed), not `auto`, and `world-stage` is `3`.
- [ ] The version in `crates/bv-server/Cargo.toml` is the version of the tag without the `v`.
- [ ] README and [CONFIGURATION.md](CONFIGURATION.md) match the behaviour.

## Tag

```
git switch main
git pull
git tag -a v1.0.0 -m "BlockVerse 1.0.0"
git push origin v1.0.0
```

The release workflow checks that the tag equals the version, builds and tests Linux (x86_64 and aarch64), Windows and Android files, writes a `.sha256` for each and attaches them to a GitHub release. A tag with a hyphen (`v1.1.0-rc.1`) is marked as a pre-release.

## After the tag

1. Open the release page and check that the `linux-x86_64`, `linux-aarch64` files and their `.sha256` files are attached. Windows and Android are experimental: if a job failed, the installer builds from source on those systems.
2. Test the installer on a clean device: `curl -fsSL https://raw.githubusercontent.com/BlockVerse-Mc/BlockVerse/main/install.sh | bash -s -- v1.0.0`.
3. Announce only what the checklist proved.

## Fixing a bad release

Delete the release and the tag, fix `main`, tag again:

```
git push origin --delete v1.0.0
git tag -d v1.0.0
```

## Versions

Change the number in `crates/bv-server/Cargo.toml` (`version = "..."`). The server prints it from there; `RELEASE_NAME` in `crates/bv-server/src/version.rs` is the label shown next to it.
