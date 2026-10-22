// Adapted in part from bedrock-rs examples/server.rs (Apache-2.0), see NOTICE.

mod cli;
mod config;
mod console;
mod identity;
mod lang;
mod limits;
mod logger;
mod manifest;
mod metrics;
mod plugins;
mod selftest;
mod server;
mod setup;
mod shared;
mod version;
mod world;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bedrock::network::listener::Listener;
use bedrock::protocol::{ProtoVersion, Unknown};

use crate::config::Config;
use crate::plugins::PluginHost;
use crate::server::Protocol;
use crate::shared::Shared;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = cli::handle(&args) {
        std::process::exit(code);
    }
    setup::first_run();
    let tagging_disabled = disable_heap_tagging();
    let (cfg, warnings) = config::load_settings();
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    if cfg.max_threads > 0 {
        builder.worker_threads(cfg.max_threads);
    }
    let runtime = match builder.build() {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("Cannot start the async runtime: {e}");
            return;
        }
    };
    runtime.block_on(run(cfg, warnings, tagging_disabled));
}

// Android heap tagging
#[cfg(target_os = "android")]
unsafe extern "C" {
    fn mallopt(param: i32, value: i32) -> i32;
}

#[cfg(target_os = "android")]
fn disable_heap_tagging() -> bool {
    const M_BIONIC_SET_HEAP_TAGGING_LEVEL: i32 = -204;
    const M_HEAP_TAGGING_LEVEL_NONE: i32 = 0;
    unsafe { mallopt(M_BIONIC_SET_HEAP_TAGGING_LEVEL, M_HEAP_TAGGING_LEVEL_NONE) != 0 }
}

#[cfg(not(target_os = "android"))]
fn disable_heap_tagging() -> bool {
    false
}

// Server
async fn run(mut cfg: Config, warnings: Vec<String>, tagging_disabled: bool) {
    let started = std::time::Instant::now();
    if let Err(e) = logger::init_file("logs") {
        eprintln!("Log file disabled: {e}");
    }
    std::panic::set_hook(Box::new(|info| {
        logger::error(&format!("Internal error: {info}"));
    }));
    lang::set(cfg.language);
    logger::set_max_level(cfg.log_level);
    logger::set_utc_offset_secs(cfg.utc_offset_secs.unwrap_or_else(logger::auto_offset_secs));
    logger::info(&lang::starting(
        version::RELEASE_NAME,
        version::VERSION,
        Protocol::GAME_VERSION,
        Protocol::PROTOCOL_VERSION,
    ));
    if tagging_disabled {
        logger::info("Android heap pointer tagging disabled");
    }
    for w in warnings {
        logger::warn(&format!("server.properties: {w}"));
    }
    if let Ok(value) = std::env::var("BV_BIND") {
        match value.parse::<SocketAddr>() {
            Ok(addr) => {
                cfg.server_ip = addr.ip();
                cfg.server_port = addr.port();
            }
            Err(_) => logger::warn(&format!("Ignoring invalid BV_BIND '{value}'")),
        }
    }
    if cfg.online_mode {
        logger::error("online-mode=true is not supported yet: identity verification is not implemented. Set online-mode=false.");
        return;
    }
    logger::warn(&lang::offline_warning_1());
    logger::warn(&lang::offline_warning_2());

    if std::env::var_os("BV_SELFTEST").is_some() {
        selftest::run();
    }

    // Plugins
    let mut plugins = PluginHost::new();
    plugins.load_dir(&cfg.plugin_dir);
    logger::info(&format!("{} plugin(s) loaded", plugins.count()));

    // Listener
    let bind = cfg.bind();
    let mut listener = match Listener::new_raknet(
        bind,
        cfg.server_name.clone(),
        cfg.motd.clone(),
        Protocol::GAME_VERSION.to_string(),
        Protocol::PROTOCOL_VERSION,
        Protocol::RAKNET_VERSION,
        cfg.max_players,
        0,
        false,
    )
    .await
    {
        Ok(l) => l,
        Err(e) => {
            logger::error(&format!("Cannot bind {bind}: {e}"));
            return;
        }
    };
    if let Err(e) = listener.start().await {
        logger::error(&format!("Cannot start the listener: {e}"));
        return;
    }
    logger::info(&lang::network_running(&bind.to_string()));

    // Background tasks
    let shared = Arc::new(Shared::new(cfg, plugins));
    logger::info(&lang::level_info(&shared.config.level_name, shared.level_seed));
    if std::env::var_os("BV_NO_CONSOLE").is_none() {
        tokio::spawn(console::run(shared.clone()));
    }
    if std::env::var_os("BV_NO_MONITOR").is_none() {
        tokio::spawn(metrics::monitor_loop_lag(shared.clone()));
    }
    logger::success(&lang::done(started.elapsed().as_secs_f64()));

    // Accept loop
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                logger::info(&lang::signal_stop());
                break;
            }
            _ = shared.shutdown.notified() => {
                break;
            }
            accepted = listener.accept::<Unknown>() => match accepted {
                Ok(conn) => {
                    let shared = shared.clone();
                    let task = tokio::spawn(async move { server::handle_connection(conn, shared).await });
                    tokio::spawn(async move {
                        match task.await {
                            Ok(Ok(())) => {}
                            Ok(Err(e)) => logger::warn(&format!("Connection ended: {e}")),
                            Err(e) => logger::error(&format!("Connection task failed: {e}")),
                        }
                    });
                }
                Err(e) => {
                    logger::warn(&format!("Accept failed: {e}"));
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            },
        }
    }

    // Shutdown
    logger::info(&lang::stopping());
    shared.unload_plugins();
    logger::success(&lang::goodbye());
    std::process::exit(0);
}
