use std::sync::Arc;
use std::sync::atomic::Ordering;

use bedrock::protocol::ProtoVersion;
use bv_plugin_abi::BV_ABI_VERSION;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::logger;
use crate::metrics::format_uptime;
use crate::server::Protocol;
use crate::shared::Shared;
use crate::version;

pub async fn run(shared: Arc<Shared>) {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if !handle(&shared, line.trim()) {
            break;
        }
    }
}

pub fn version_line() -> String {
    format!(
        "BlockVerse {} ({}) | Minecraft {} | protocol {} | plugin ABI {}",
        version::RELEASE_NAME,
        version::VERSION,
        Protocol::GAME_VERSION,
        Protocol::PROTOCOL_VERSION,
        BV_ABI_VERSION
    )
}

pub fn world_line(shared: &Shared) -> String {
    format!(
        "World '{}': flat, seed {}, generated on demand, not saved (world saving is not implemented yet)",
        shared.config.level_name, shared.level_seed
    )
}

pub fn status_line(shared: &Shared) -> String {
    let m = &shared.metrics;
    format!(
        "Uptime {} | players {}/{} | connections {} | login failures {} | packets in {} | loop lag {:.1} ms (max {:.1} ms)",
        format_uptime(m.uptime_secs()),
        shared.player_list().len(),
        shared.config.max_players,
        m.connections.load(Ordering::Relaxed),
        m.login_failures.load(Ordering::Relaxed),
        m.packets_in.load(Ordering::Relaxed),
        m.lag_last_ms(),
        m.lag_max_ms()
    )
}

fn debug_command(shared: &Shared, args: &str) {
    let (sub, rest) = args.split_once(' ').unwrap_or((args, ""));
    let rest = rest.trim();
    match sub.to_ascii_lowercase().as_str() {
        "" => logger::info(&format!(
            "Log level: {} (usage: debug level|network|chunks|plugins)",
            logger::max_level().name()
        )),
        "level" => match logger::parse_level(rest) {
            Some(level) => {
                logger::info(&format!("Log level set to {}", level.name()));
                logger::set_max_level(level);
            }
            None => logger::warn("Usage: debug level <error|warn|info|debug|trace>"),
        },
        "network" => {
            let m = &shared.metrics;
            logger::info(&format!(
                "Connections {} | login failures {} | packets in {} | players {}/{}",
                m.connections.load(Ordering::Relaxed),
                m.login_failures.load(Ordering::Relaxed),
                m.packets_in.load(Ordering::Relaxed),
                shared.player_list().len(),
                shared.config.max_players
            ));
        }
        "chunks" => {
            let variant = match shared.config.chunk_variant {
                Some(v) => v.to_string(),
                None => "auto".to_string(),
            };
            logger::info(&format!(
                "Chunks sent {} | view distance {} | world stage {} | chunk variant {}",
                shared.metrics.chunks_sent.load(Ordering::Relaxed),
                shared.config.view_distance,
                shared.config.world_stage,
                variant
            ));
        }
        "plugins" => {
            let plugins = shared.plugin_summary();
            logger::info(&format!("Plugins loaded: {}", plugins.len()));
            for p in plugins {
                logger::info(&format!("  {p}"));
            }
        }
        other => logger::warn(&format!("Unknown debug option '{other}'. Type 'debug' for usage.")),
    }
}

pub fn handle(shared: &Shared, line: &str) -> bool {
    if line.is_empty() {
        return true;
    }
    let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
    let rest = rest.trim();
    match cmd.to_ascii_lowercase().as_str() {
        "help" | "?" => logger::info(&crate::lang::help_text()),
        "version" => logger::info(&version_line()),
        "status" => logger::info(&status_line(shared)),
        "worlds" => logger::info(&world_line(shared)),
        "list" => {
            let players = shared.player_list();
            logger::info(&format!("Players online: {}/{}", players.len(), shared.config.max_players));
            for (id, name, skin) in players {
                logger::info(&format!("  #{id} {name} (skin {skin})"));
            }
        }
        "plugins" => {
            let plugins = shared.plugin_summary();
            logger::info(&format!("Plugins loaded: {}", plugins.len()));
            for p in plugins {
                logger::info(&format!("  {p}"));
            }
        }
        "skins" => {
            for s in bv_skins::all() {
                logger::info(&format!("  {} - {}", s.id, s.name));
            }
        }
        "say" => {
            if rest.is_empty() {
                logger::warn("Usage: say <text>");
            } else {
                logger::info(&format!("[Server] {rest}"));
            }
        }
        "debug" => debug_command(shared, rest),
        "save-all" => logger::warn("save-all: nothing to save, world and player saving are not implemented yet"),
        "fakejoin" => {
            if rest.is_empty() {
                logger::warn("Usage: fakejoin <name>");
            } else {
                let id = shared.player_join(Some(rest.to_string()));
                logger::info(&format!("Simulated join, player id {id}"));
            }
        }
        "fakechat" => {
            let (id_text, text) = rest.split_once(' ').unwrap_or((rest, ""));
            match (id_text.parse::<u64>(), text.trim()) {
                (Ok(id), text) if !text.is_empty() => match shared.player_chat(id, text) {
                    None => logger::warn(&format!("No player with id {id}")),
                    Some(true) => logger::info("Chat message was cancelled by a plugin"),
                    Some(false) => logger::info(&format!("<#{id}> {text}")),
                },
                _ => logger::warn("Usage: fakechat <id> <text>"),
            }
        }
        "fakequit" => match rest.parse::<u64>() {
            Ok(id) => shared.player_quit(id),
            Err(_) => logger::warn("Usage: fakequit <id>"),
        },
        "selftest" => crate::selftest::run(),
        "stop" | "exit" | "quit" => {
            logger::info(&crate::lang::stop_requested());
            shared.shutdown.notify_one();
            return false;
        }
        other => logger::warn(&format!("Unknown command '{other}'. Type 'help'.")),
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::plugins::PluginHost;

    fn shared() -> Shared {
        Shared::new(Config::default(), PluginHost::new())
    }

    #[test]
    fn empty_and_unknown_commands_keep_the_console_running() {
        let s = shared();
        assert!(handle(&s, ""));
        assert!(handle(&s, "nonsense"));
        assert!(handle(&s, "HELP"));
    }

    #[test]
    fn fake_players_can_join_chat_and_quit() {
        let s = shared();
        assert!(handle(&s, "fakejoin Steve"));
        assert_eq!(s.player_list().len(), 1);
        assert!(handle(&s, "fakechat 1 hello there"));
        assert!(handle(&s, "fakechat 99 hello"));
        assert!(handle(&s, "fakequit 1"));
        assert!(s.player_list().is_empty());
    }

    #[test]
    fn bad_arguments_do_not_crash() {
        let s = shared();
        assert!(handle(&s, "fakejoin"));
        assert!(handle(&s, "fakechat abc text"));
        assert!(handle(&s, "fakechat 1"));
        assert!(handle(&s, "fakequit x"));
        assert!(handle(&s, "say"));
        assert!(handle(&s, "debug level loud"));
        assert!(handle(&s, "debug nothing"));
    }

    #[test]
    fn information_commands_run() {
        let s = shared();
        for command in ["version", "status", "worlds", "list", "plugins", "skins", "save-all", "debug", "debug network", "debug chunks", "debug plugins"] {
            assert!(handle(&s, command), "{command}");
        }
    }

    #[test]
    fn stop_ends_the_console() {
        let s = shared();
        assert!(!handle(&s, "stop"));
    }

    #[test]
    fn version_line_names_every_version() {
        let line = version_line();
        assert!(line.contains("protocol 766"));
        assert!(line.contains("Minecraft"));
        assert!(line.contains("plugin ABI 1"));
    }

    #[test]
    fn status_line_mentions_players_and_uptime() {
        let s = shared();
        let line = status_line(&s);
        assert!(line.contains("players 0/20"));
        assert!(line.contains("Uptime 00:00:0"));
    }

    #[test]
    fn world_line_shows_the_seed() {
        let s = Shared::new(Config { level_seed: Some(99), ..Config::default() }, PluginHost::new());
        assert!(world_line(&s).contains("seed 99"));
    }
}
