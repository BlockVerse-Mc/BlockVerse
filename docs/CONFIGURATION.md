# Configuration

The server reads `server.properties` from its data folder (the folder of the binary; `bvmc/` in a source checkout). The file is created with documented defaults on the first start and is never overwritten. If only the old `blockverse.properties` exists it is read instead (with a warning); copy its values to `server.properties`.

Invalid values are reported in the log and the default is used. Unknown keys are ignored with a warning.

| Key | Values | Default | Effect |
|---|---|---|---|
| `server-ip` | IP address | `127.0.0.1` | Address to listen on (`0.0.0.0` for your network) |
| `server-port` | 1 to 65535 | `19132` | UDP port |
| `server-name` | text | `BlockVerse` | Name in the server list |
| `motd` | text | `BlockVerse` | Second line in the server list (BlockVerse extension) |
| `language` | en, fa, ar, tr, ru, es, pt, fr, de, it, id, zh, ja, ko | `en` | Language of the main console messages (startup, join/leave, help, shutdown); other messages stay English. The server asks for it on the first start. See [TRANSLATING.md](TRANSLATING.md) |
| `gamemode` | survival, creative, adventure (or 0 to 2) | `creative` | Game mode sent at world entry |
| `difficulty` | peaceful, easy, normal, hard (or 0 to 3) | `peaceful` | Difficulty sent at world entry |
| `max-players` | number above 0 | `20` | Player limit |
| `level-name` | letters, digits, space, `-`, `_`, `.` (64 max) | `world` | World name (validated so it cannot escape its folder) |
| `level-seed` | number or text, empty = random | empty | World seed (text is hashed) |
| `level-type` | `flat` | `flat` | Only the flat development world exists |
| `view-distance` | 1 to 16 | `4` | Chunks sent around the player |
| `online-mode` | true, false | `false` | `true` is refused at startup: identity verification is not implemented |
| `allow-cheats` | true, false | `true` | Commands enabled flag sent to the client |
| `default-player-permission-level` | visitor, member, operator | `member` | Permission level sent to players |
| `max-threads` | 0 to 256 | `0` | Runtime worker threads (0 = automatic) |
| `log-level` | error, warn, info, debug, trace | `info` | Log filter (also `debug level <x>` in the console) |
| `utc-offset` | `auto` or `+03:30` | `auto` | Time zone of log timestamps |
| `plugin-dir` | folder | `plugins` | Native plugins folder |
| `login-timeout` | seconds above 0 | `15` | Time a client has to finish logging in |
| `max-pending-logins` | 1 to 1000 | `32` | Simultaneous logins |
| `max-packets-per-second` | 10 to 100000 | `2000` | Per-player packet limit before disconnecting |
| `world-stage` | 0 to 3 | `3` | Testing only: how much of the world is sent |
| `chunk-variant` | auto or 0 to 3 | `auto` | Testing only: chunk format variant |

## Not supported yet (deliberately absent)
`server-portv6` (no IPv6 listener), `tick-distance`, `allow-list`, `player-idle-timeout`, `autosave` (no world saving yet), `online-mode=true`.

## Environment variables
`BV_BIND=ip:port` overrides the address, `BV_SELFTEST=1` runs the internal checks at startup, `BV_NO_CONSOLE=1`, `BV_NO_MONITOR=1` and `BV_SKIP_EXTRA=1` switch parts off for diagnosing problems, `NO_COLOR=1` disables colors.
