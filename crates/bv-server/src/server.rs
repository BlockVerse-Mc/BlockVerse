// Login flow adapted from bedrock-rs examples/server.rs (Apache-2.0), see NOTICE.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bedrock::network::compression::Compression;
use bedrock::network::connection::Connection;
use bedrock::protocol::v662::enums::{
    ChatRestrictionLevel, Difficulty, EditorWorldType, EducationEditionOffer, GamePublishSetting,
    GameType, GeneratorType, PacketCompressionAlgorithm, PlayStatus, PlayerPermissionLevel,
    ServerAuthMovementMode, SpawnBiomeType,
};
use bedrock::protocol::v662::packets::{
    AvailableActorIdentifiersPacket, BiomeDefinitionListPacket, ChunkRadiusUpdatedPacket, CreativeContentPacket,
    LevelChunkPacket, NetworkChunkPublisherUpdatePacket, NetworkSettingsPacket, PlayStatusPacket, StartGamePacket,
};
use bedrock::protocol::v662::types::{
    ActorRuntimeID, ActorUniqueID, BaseGameVersion, BlockPos, ChunkPos, EduSharedUriResource, Experiments,
    GameRulesChangedPacketData, NetworkBlockPosition, NetworkPermissions, SpawnSettings,
    SyncedPlayerMovementSettings,
};
use bedrock::protocol::v671::packets::ResourcePackStackPacket;
use bedrock::protocol::v685::types::LevelSettings;
use bedrock::protocol::v766::packets::ResourcePacksInfoPacket;
use bedrock::protocol::{ProtoVersion, Unknown, V766};
use uuid::Uuid;

use crate::config::{self, GameMode, PermissionLevel};
use crate::identity;
use crate::limits::RateLimiter;
use crate::logger::{self, Level};
use crate::shared::Shared;
use crate::world;

pub type Protocol = V766;

fn random_u64(salt: u64) -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u64(salt);
    hasher.finish()
}

fn random_uuid() -> Uuid {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let high = random_u64(n) as u128;
    let low = random_u64(n.wrapping_add(0x9E37_79B9_7F4A_7C15)) as u128;
    Uuid::from_u128((high << 64) | low)
}

fn trace(_shared: &Shared, text: &str) {
    logger::debug(text);
}

fn game_type(mode: GameMode) -> GameType {
    match mode {
        GameMode::Survival => GameType::Survival,
        GameMode::Creative => GameType::Creative,
        GameMode::Adventure => GameType::Adventure,
    }
}

fn difficulty(value: config::Difficulty) -> Difficulty {
    match value {
        config::Difficulty::Peaceful => Difficulty::Peaceful,
        config::Difficulty::Easy => Difficulty::Easy,
        config::Difficulty::Normal => Difficulty::Normal,
        config::Difficulty::Hard => Difficulty::Hard,
    }
}

fn permission(level: PermissionLevel) -> PlayerPermissionLevel {
    match level {
        PermissionLevel::Visitor => PlayerPermissionLevel::Visitor,
        PermissionLevel::Member => PlayerPermissionLevel::Member,
        PermissionLevel::Operator => PlayerPermissionLevel::Operator,
    }
}

// Connection
pub async fn handle_connection(unknown_conn: Connection<Unknown>, shared: Arc<Shared>) -> Result<(), String> {
    shared.metrics.connections.fetch_add(1, Ordering::Relaxed);
    if shared.is_full() {
        unknown_conn.close().await;
        return Err("server is full, connection refused".to_string());
    }

    let Ok(login_slot) = shared.login_slots.try_acquire() else {
        unknown_conn.close().await;
        return Err("too many logins in progress, connection refused".to_string());
    };
    let limit = Duration::from_secs(shared.config.login_timeout_secs);
    let (mut conn, name, mut journal) = match tokio::time::timeout(limit, login_phase(unknown_conn, &shared)).await {
        Ok(Ok(done)) => done,
        Ok(Err(e)) => {
            shared.metrics.login_failures.fetch_add(1, Ordering::Relaxed);
            return Err(e);
        }
        Err(_) => {
            shared.metrics.login_failures.fetch_add(1, Ordering::Relaxed);
            return Err(format!("login timed out after {} s", limit.as_secs()));
        }
    };

    drop(login_slot);

    // Session
    let id = shared.player_join(name);
    let joined_at = std::time::Instant::now();
    let mut limiter = RateLimiter::new(shared.config.max_packets_per_second, joined_at);
    let mut received: u64 = 0;
    let mut logged: u32 = 0;
    let mut close_reason = String::from("the connection ended");
    'session: loop {
        let packets = match conn.recv().await {
            Ok(packets) => packets,
            Err(e) => {
                close_reason = format!("receiving failed: {e}");
                break 'session;
            }
        };
        let count = packets.len() as u64;
        received += count;
        shared.metrics.add_packets(count);
        if !limiter.allow(count, std::time::Instant::now()) {
            logger::warn(&format!("Player {id} sent too many packets, disconnecting"));
            break 'session;
        }
        for packet in &packets {
            if logger::enabled(Level::Debug) && logged < 60 {
                logged += 1;
                let name = packet_name(packet);
                logger::debug(&format!("<- {name}"));
                if name.contains("Disconnect") {
                    logger::info(&format!("Player {id} sent {packet:?}"));
                }
            }
            match packet {
                Protocol::RequestChunkRadiusPacket(request) => {
                    let radius = request.chunk_radius.clamp(1, shared.config.view_distance.max(1));
                    if let Err(e) = send_world(&mut conn, radius, &shared, &mut journal).await {
                        logger::warn(&format!("Sending the world to player {id} failed: {e}"));
                        break 'session;
                    }
                    logger::success(&format!("Sent the world to player {id} (view distance {radius})"));
                }
                Protocol::SetLocalPlayerAsInitializedPacket(_) => {
                    logger::success(&format!("Player {id} finished loading"));
                }
                _ => {}
            }
        }
    }
    logger::info(&format!(
        "Connection of player {id} closed after {received} packets, {:.2} s ({close_reason})",
        joined_at.elapsed().as_secs_f64()
    ));
    logger::info(&format!("Last packets sent to player {id}: {}", journal.summary()));
    shared.player_quit(id);
    Ok(())
}

// Login
async fn login_phase(
    mut unknown_conn: Connection<Unknown>,
    shared: &Shared,
) -> Result<(Connection<Protocol>, Option<String>, Journal), String> {
    let mut journal = Journal::new();

    // RequestNetworkSettings
    let packets = unknown_conn
        .recv()
        .await
        .map_err(|e| format!("receiving RequestNetworkSettings: {e}"))?;
    let mut conn = match packets.first() {
        Some(Unknown::RequestNetworkSettingsPacket(request)) => {
            let client_version = request.client_network_version;
            if client_version == Protocol::PROTOCOL_VERSION as i32 {
                unknown_conn.into_ver::<Protocol>()
            } else {
                unknown_conn.close().await;
                return Err(format!(
                    "client protocol {client_version} is not supported (server speaks {} / Minecraft {})",
                    Protocol::PROTOCOL_VERSION,
                    Protocol::GAME_VERSION
                ));
            }
        }
        _ => {
            unknown_conn.close().await;
            return Err("first packet was not RequestNetworkSettings".to_string());
        }
    };

    // NetworkSettings
    journal.send(&mut conn, &[Protocol::NetworkSettingsPacket(Box::new(NetworkSettingsPacket {
        compression_threshold: 1,
        compression_algorithm: PacketCompressionAlgorithm::None,
        client_throttle_enabled: false,
        client_throttle_threshold: 0,
        client_throttle_scalar: 0.0,
    }))])
    .await
    .map_err(|e| format!("sending NetworkSettings: {e}"))?;
    conn.compression = Some(Compression::None);
    trace(shared, "sent NetworkSettings");

    // Login
    let login_packets = conn.recv().await.map_err(|e| format!("receiving Login: {e}"))?;
    let name = login_packets.iter().find_map(|packet| match packet {
        Protocol::LoginPacket(login) => identity::extract_display_name(&login.connection_request),
        _ => None,
    });
    trace(shared, &format!("login received, player name: {name:?}"));

    // Resource packs
    journal.send(&mut conn, &[
        Protocol::PlayStatusPacket(Box::new(PlayStatusPacket {
            status: PlayStatus::LoginSuccess,
        })),
        Protocol::ResourcePacksInfoPacket(Box::new(ResourcePacksInfoPacket {
            resource_pack_required: false,
            has_addon_packs: false,
            has_scripts: false,
            world_template_uuid: Uuid::nil(),
            world_template_version: String::new(),
            resource_packs: vec![],
        })),
        Protocol::ResourcePackStackPacket(Box::new(ResourcePackStackPacket {
            texture_pack_required: false,
            addon_list: vec![],
            texture_pack_list: vec![],
            base_game_version: BaseGameVersion(String::from("1.0")),
            experiments: Experiments {
                experiments: vec![],
                ever_toggled: false,
            },
            include_editor_packs: false,
        })),
    ])
    .await
    .map_err(|e| format!("sending resource packs: {e}"))?;
    trace(shared, "sent PlayStatus, resource packs info and stack");
    conn.recv().await.map_err(|e| format!("receiving client response 1: {e}"))?;
    conn.recv().await.map_err(|e| format!("receiving client response 2: {e}"))?;
    trace(shared, "client answered the resource pack exchange");

    // StartGame
    trace(shared, "building StartGame");
    let start_game = StartGamePacket {
        target_actor_id: ActorUniqueID(609),
        target_runtime_id: ActorRuntimeID(402),
        actor_game_type: game_type(shared.config.gamemode),
        position: (0.5, world::SPAWN_EYE_Y, 0.5),
        rotation: Default::default(),
        settings: LevelSettings {
            seed: shared.level_seed,
            spawn_settings: SpawnSettings {
                spawn_type: SpawnBiomeType::Default,
                user_defined_biome_name: String::from("RandomBiome"),
                dimension: 0,
            },
            generator_type: GeneratorType::Flat,
            game_type: game_type(shared.config.gamemode),
            is_hardcore_enabled: false,
            game_difficulty: difficulty(shared.config.difficulty),
            default_spawn_block_position: NetworkBlockPosition { x: 0, y: 0, z: 0 },
            achievements_disabled: true,
            editor_world_type: EditorWorldType::NonEditor,
            is_created_in_editor: false,
            is_exported_from_editor: false,
            day_cycle_stop_time: 2000,
            education_edition_offer: EducationEditionOffer::None,
            education_features_enabled: false,
            education_product_id: String::new(),
            rain_level: 0.0,
            lightning_level: 0.0,
            has_confirmed_platform_locked_content: false,
            multiplayer_enabled: true,
            lan_broadcasting_enabled: true,
            xbox_live_broadcast_setting: GamePublishSetting::FriendsOnly,
            platform_broadcast_setting: GamePublishSetting::FriendsOnly,
            commands_enabled: shared.config.allow_cheats,
            texture_packs_required: false,
            rule_data: GameRulesChangedPacketData { rules_list: vec![] },
            experiments: Experiments {
                experiments: vec![],
                ever_toggled: false,
            },
            bonus_chest_enabled: false,
            starting_map_enabled: false,
            player_permissions: permission(shared.config.default_permission),
            server_chunk_tick_range: 4,
            locked_behaviour_pack: false,
            locked_resource_pack: false,
            from_locked_template: false,
            use_msa_gamer_tags: false,
            from_template: false,
            has_locked_template_settings: false,
            only_spawn_v1_villagers: false,
            persona_disabled: false,
            custom_skins_disabled: false,
            emote_chat_muted: false,
            base_game_version: BaseGameVersion(Protocol::GAME_VERSION.to_string()),
            limited_world_width: 16,
            limited_world_depth: 16,
            nether_type: true,
            edu_shared_uri_resource: EduSharedUriResource {
                button_name: String::new(),
                link_uri: String::new(),
            },
            override_force_experimental_gameplay: None,
            chat_restriction_level: ChatRestrictionLevel::None,
            disable_player_interactions: false,
            server_id: String::new(),
            world_id: String::new(),
            scenario_id: String::new(),
        },
        level_id: String::from("QmxvY2tWZXJzZQ=="),
        level_name: shared.config.level_name.clone(),
        template_content_identity: String::new(),
        is_trial: false,
        movement_settings: SyncedPlayerMovementSettings {
            authority_mode: ServerAuthMovementMode::ClientAuthoritative,
            rewind_history_size: 0,
            server_authoritative_block_breaking: false,
        },
        current_level_time: 9000,
        enchantment_seed: 99000,
        block_properties: vec![],
        item_list: vec![],
        multiplayer_correlation_id: random_uuid().to_string(),
        enable_item_stack_net_manager: false,
        server_version: Protocol::GAME_VERSION.to_string(),
        player_property_data: HashMap::new(),
        server_block_type_registry_checksum: 0,
        world_template_id: Uuid::nil(),
        server_enabled_client_side_generation: false,
        block_network_ids_are_hashes: true,
        network_permissions: NetworkPermissions {
            server_auth_sound_enabled: false,
        },
    };
    trace(shared, "StartGame built");
    journal.send(&mut conn, &[Protocol::StartGamePacket(Box::new(start_game))])
        .await
        .map_err(|e| format!("sending StartGame: {e}"))?;
    trace(shared, "sent StartGame");

    // World data
    if std::env::var_os("BV_SKIP_EXTRA").is_none() {
    journal.send(&mut conn, &[
        Protocol::BiomeDefinitionListPacket(Box::new(BiomeDefinitionListPacket {
            biome_definition_data: HashMap::new(),
        })),
        Protocol::AvailableActorIdentifiersPacket(Box::new(AvailableActorIdentifiersPacket {
            actor_info_list: HashMap::new(),
        })),
        Protocol::CreativeContentPacket(Box::new(CreativeContentPacket { write_entries: vec![] })),
    ])
    .await
    .map_err(|e| format!("sending world data: {e}"))?;
    trace(shared, "sent biome, actor and creative content packets");
    }

    Ok((conn, name, journal))
}

// World
async fn send_world(
    conn: &mut Connection<Protocol>,
    radius: i32,
    shared: &Shared,
    journal: &mut Journal,
) -> Result<(), String> {
    let stage = shared.config.world_stage;
    let variant = shared.pick_chunk_variant();
    logger::info(&format!("World test: stage {stage}, chunk variant {variant}"));

    // Chunk radius
    journal.send(conn, &[Protocol::ChunkRadiusUpdatedPacket(Box::new(ChunkRadiusUpdatedPacket {
        chunk_radius: radius,
    }))])
    .await
    .map_err(|e| format!("sending chunk radius: {e}"))?;
    trace(shared, "sent chunk radius");
    if stage < 1 {
        return Ok(());
    }

    // Publisher update
    journal.send(conn, &[Protocol::NetworkChunkPublisherUpdatePacket(Box::new(NetworkChunkPublisherUpdatePacket {
        new_view_position: BlockPos { x: 0, y: world::SPAWN_FEET_Y, z: 0 },
        new_view_radius: radius as u32 * 16,
        server_built_chunks: vec![],
    }))])
    .await
    .map_err(|e| format!("sending publisher update: {e}"))?;
    trace(shared, "sent publisher update");
    if stage < 2 {
        return Ok(());
    }

    // Chunks
    let chunk_radius = if stage == 2 { 0 } else { radius };
    let (sub_chunk_count, payload) = world::chunk_variant(variant);
    logger::debug(&format!(
        "Chunk payload: variant {variant}, sub chunk count {sub_chunk_count}, {} bytes, {} chunks, dimension 0, cache off",
        payload.len(),
        (2 * chunk_radius + 1).pow(2)
    ));
    if logger::enabled(Level::Trace) {
        logger::trace(&format!("Chunk payload bytes: {}", world::hex(&payload)));
    }
    let mut chunks = Vec::new();
    for chunk_x in -chunk_radius..=chunk_radius {
        for chunk_z in -chunk_radius..=chunk_radius {
            chunks.push(Protocol::LevelChunkPacket(Box::new(LevelChunkPacket {
                chunk_position: ChunkPos { x: chunk_x, z: chunk_z },
                dimension_id: 0,
                sub_chunk_count,
                sub_chunk_limit: 0,
                cache_enabled: false,
                cache_blobs: vec![],
                serialized_chunk_data: payload.clone(),
            })));
        }
    }
    shared.metrics.add_chunks(chunks.len() as u64);
    for (index, batch) in chunks.chunks(16).enumerate() {
        journal.send(conn, batch)
            .await
            .map_err(|e| format!("sending chunk batch {index}: {e}"))?;
        trace(shared, &format!("sent chunk batch {index} ({} chunks)", batch.len()));
    }
    if stage < 3 {
        return Ok(());
    }

    // PlayerSpawn
    journal.send(conn, &[Protocol::PlayStatusPacket(Box::new(PlayStatusPacket {
        status: PlayStatus::PlayerSpawn,
    }))])
    .await
    .map_err(|e| format!("sending PlayerSpawn: {e}"))?;
    trace(shared, "sent PlayerSpawn");
    Ok(())
}

fn name_of(text: &str) -> String {
    text.split(['(', ' ', '{']).next().unwrap_or("?").to_string()
}

fn packet_name(packet: &Protocol) -> String {
    name_of(&format!("{packet:?}"))
}

fn clip(text: &str, limit: usize) -> String {
    let short: String = text.chars().take(limit).collect();
    if short.len() < text.len() {
        format!("{short}...")
    } else {
        short
    }
}

fn summarize(names: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut index = 0;
    while index < names.len() {
        let mut run = 1;
        while index + run < names.len() && names[index + run] == names[index] {
            run += 1;
        }
        if run > 1 {
            parts.push(format!("{} x{run}", names[index]));
        } else {
            parts.push(names[index].clone());
        }
        index += run;
    }
    parts.join(", ")
}

// Outbound journal
struct Journal {
    started: std::time::Instant,
    entries: Vec<(u128, String)>,
}

impl Journal {
    fn new() -> Self {
        Self { started: std::time::Instant::now(), entries: Vec::new() }
    }

    fn record(&mut self, names: &[String]) {
        let text = summarize(names);
        let millis = self.started.elapsed().as_millis();
        logger::debug(&format!("-> #{} {text} (+{millis} ms)", self.entries.len() + 1));
        self.entries.push((millis, text));
    }

    fn summary(&self) -> String {
        let first = self.entries.len().saturating_sub(4);
        self.entries
            .iter()
            .enumerate()
            .skip(first)
            .map(|(index, (millis, text))| format!("#{} {text} (+{millis} ms)", index + 1))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    async fn send(&mut self, conn: &mut Connection<Protocol>, packets: &[Protocol]) -> Result<(), String> {
        let mut names = Vec::with_capacity(packets.len());
        for packet in packets {
            let text = format!("{packet:?}");
            let name = name_of(&text);
            if logger::enabled(Level::Trace) && name != "LevelChunkPacket" {
                logger::trace(&format!("   {}", clip(&text, 1500)));
            }
            names.push(name);
        }
        conn.send(packets).await.map_err(|e| e.to_string())?;
        self.record(&names);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_map_to_the_protocol_values() {
        assert_eq!(game_type(GameMode::Survival) as i32, 0);
        assert_eq!(game_type(GameMode::Creative) as i32, 1);
        assert_eq!(game_type(GameMode::Adventure) as i32, 2);
        assert_eq!(difficulty(config::Difficulty::Peaceful) as i32, 0);
        assert_eq!(difficulty(config::Difficulty::Hard) as i32, 3);
        assert_eq!(permission(PermissionLevel::Visitor) as i32, 0);
        assert_eq!(permission(PermissionLevel::Operator) as i32, 2);
    }

    #[test]
    fn consecutive_packets_are_grouped_in_the_journal() {
        let names: Vec<String> = ["A", "B", "B", "B", "A"].iter().map(|s| s.to_string()).collect();
        assert_eq!(summarize(&names), "A, B x3, A");
        assert_eq!(summarize(&[]), "");
    }

    #[test]
    fn the_journal_keeps_the_last_four_entries() {
        let mut journal = Journal::new();
        for i in 0..6 {
            journal.record(&[format!("P{i}")]);
        }
        let text = journal.summary();
        assert!(text.contains("#6 P5") && text.contains("#3 P2"), "{text}");
        assert!(!text.contains("#2 P1"), "{text}");
    }

    #[test]
    fn packet_names_stop_at_the_first_separator() {
        assert_eq!(name_of("PlayStatusPacket(PlayStatusPacket { status: LoginSuccess })"), "PlayStatusPacket");
        assert_eq!(name_of("Unit"), "Unit");
        assert_eq!(clip("abcdef", 3), "abc...");
        assert_eq!(clip("abc", 3), "abc");
    }

    #[test]
    fn random_uuids_are_not_nil_and_differ() {
        let a = random_uuid();
        let b = random_uuid();
        assert_ne!(a, Uuid::nil());
        assert_ne!(a, b);
    }
}
