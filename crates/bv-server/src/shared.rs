use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use bv_plugin_abi::{BV_EVENT_PLAYER_CHAT, BV_EVENT_PLAYER_JOIN, BV_EVENT_PLAYER_QUIT};
use tokio::sync::{Notify, Semaphore};

use crate::config::Config;
use crate::logger;
use crate::metrics::Metrics;
use crate::plugins::PluginHost;

pub struct Player {
    pub id: u64,
    pub name: String,
    pub skin: &'static str,
}

pub struct Shared {
    pub config: Config,
    pub metrics: Metrics,
    pub level_seed: u64,
    pub shutdown: Notify,
    pub login_slots: Semaphore,
    players: Mutex<HashMap<u64, Player>>,
    plugins: Mutex<PluginHost>,
    next_id: AtomicU64,
    variant_counter: AtomicU64,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn random_seed() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u64(nanos);
    hasher.finish()
}

impl Shared {
    pub fn new(config: Config, plugins: PluginHost) -> Self {
        let config_slots = config.max_pending_logins;
        let level_seed = config.level_seed.unwrap_or_else(random_seed);
        Self {
            config,
            level_seed,
            metrics: Metrics::new(),
            shutdown: Notify::new(),
            login_slots: Semaphore::new(config_slots),
            players: Mutex::new(HashMap::new()),
            plugins: Mutex::new(plugins),
            next_id: AtomicU64::new(1),
            variant_counter: AtomicU64::new(0),
        }
    }

    pub fn pick_chunk_variant(&self) -> u8 {
        match self.config.chunk_variant {
            Some(v) => v,
            None => (self.variant_counter.fetch_add(1, Ordering::SeqCst) % 4) as u8,
        }
    }

    pub fn is_full(&self) -> bool {
        lock(&self.players).len() as i64 >= self.config.max_players as i64
    }

    pub fn player_join(&self, name: Option<String>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let name = name.unwrap_or_else(|| format!("Player{id}"));
        let skin = bv_skins::pick(id);
        lock(&self.players).insert(id, Player { id, name: name.clone(), skin: skin.id });
        logger::success(&crate::lang::player_joined(&name, id, skin.name));
        lock(&self.plugins).dispatch(BV_EVENT_PLAYER_JOIN, id, &name, None);
        id
    }

    pub fn player_quit(&self, id: u64) {
        let removed = lock(&self.players).remove(&id);
        if let Some(player) = removed {
            logger::info(&crate::lang::player_left(&player.name, id));
            lock(&self.plugins).dispatch(BV_EVENT_PLAYER_QUIT, id, &player.name, None);
        }
    }

    pub fn player_chat(&self, id: u64, text: &str) -> Option<bool> {
        let name = lock(&self.players).get(&id).map(|p| p.name.clone())?;
        Some(lock(&self.plugins).dispatch(BV_EVENT_PLAYER_CHAT, id, &name, Some(text)))
    }

    pub fn player_list(&self) -> Vec<(u64, String, &'static str)> {
        let mut list: Vec<_> = lock(&self.players)
            .values()
            .map(|p| (p.id, p.name.clone(), p.skin))
            .collect();
        list.sort();
        list
    }

    pub fn unload_plugins(&self) {
        lock(&self.plugins).unload_all();
    }

    pub fn plugin_summary(&self) -> Vec<String> {
        lock(&self.plugins).summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared(max_players: i32) -> Shared {
        let config = Config { max_players, ..Config::default() };
        Shared::new(config, PluginHost::new())
    }

    #[test]
    fn join_assigns_sequential_ids_and_names() {
        let s = shared(10);
        assert_eq!(s.player_join(Some("Alex".to_string())), 1);
        assert_eq!(s.player_join(None), 2);
        let list = s.player_list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].1, "Alex");
        assert_eq!(list[1].1, "Player2");
    }

    #[test]
    fn default_skin_is_deterministic_per_id() {
        let s = shared(10);
        let id = s.player_join(None);
        assert_eq!(s.player_list()[0].2, bv_skins::pick(id).id);
    }

    #[test]
    fn chunk_variant_cycles_in_auto_mode_and_stays_fixed_otherwise() {
        let s = shared(10);
        let picked: Vec<u8> = (0..5).map(|_| s.pick_chunk_variant()).collect();
        assert_eq!(picked, vec![0, 1, 2, 3, 0]);
        let fixed = Shared::new(Config { chunk_variant: Some(2), ..Config::default() }, PluginHost::new());
        assert_eq!(fixed.pick_chunk_variant(), 2);
        assert_eq!(fixed.pick_chunk_variant(), 2);
    }

    #[test]
    fn configured_seed_is_used_and_missing_seed_is_generated() {
        let fixed = Shared::new(Config { level_seed: Some(5), ..Config::default() }, PluginHost::new());
        assert_eq!(fixed.level_seed, 5);
        let generated = shared(10);
        let _ = generated.level_seed;
    }

    #[test]
    fn quit_removes_the_player() {
        let s = shared(10);
        let id = s.player_join(None);
        s.player_quit(id);
        assert!(s.player_list().is_empty());
        s.player_quit(id);
        assert!(s.player_list().is_empty());
    }

    #[test]
    fn full_server_is_detected() {
        let s = shared(2);
        assert!(!s.is_full());
        s.player_join(None);
        assert!(!s.is_full());
        s.player_join(None);
        assert!(s.is_full());
    }

    #[test]
    fn chat_without_plugins_is_never_cancelled() {
        let s = shared(10);
        let id = s.player_join(None);
        assert_eq!(s.player_chat(id, "hello"), Some(false));
        assert_eq!(s.player_chat(999, "hello"), None);
    }
}
