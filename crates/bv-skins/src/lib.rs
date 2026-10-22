pub const SKIN_WIDTH: u32 = 64;
pub const SKIN_HEIGHT: u32 = 64;
pub const SKIN_BYTES: usize = 64 * 64 * 4;

pub const MAX_SKIN_DIMENSION: u32 = 512;

pub const DEFAULT_RESOURCE_PATCH: &str = r#"{"geometry":{"default":"geometry.humanoid.custom"}}"#;

#[derive(Debug, Clone, Copy)]
pub struct DefaultSkin {
    pub id: &'static str,
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub rgba: &'static [u8],
}

macro_rules! skin {
    ($id:literal, $name:literal, $file:literal) => {
        DefaultSkin {
            id: $id,
            name: $name,
            width: SKIN_WIDTH,
            height: SKIN_HEIGHT,
            rgba: include_bytes!(concat!("../assets/rgba/blockverse_", $file, ".rgba")),
        }
    };
}

static SKINS: [DefaultSkin; 12] = [
    skin!("blockverse.default.black", "Black", "black"),
    skin!("blockverse.default.charcoal", "Charcoal", "charcoal"),
    skin!("blockverse.default.silver", "Silver", "silver"),
    skin!("blockverse.default.blue", "BlockVerse Blue", "blockverse_blue"),
    skin!("blockverse.default.purple", "Deep Purple", "deep_purple"),
    skin!("blockverse.default.pink", "Pink", "pink"),
    skin!("blockverse.default.crimson", "Crimson", "crimson"),
    skin!("blockverse.default.amber", "Amber", "amber"),
    skin!("blockverse.default.yellow", "Yellow", "yellow"),
    skin!("blockverse.default.green", "Forest Green", "forest_green"),
    skin!("blockverse.default.teal", "Teal", "teal"),
    skin!("blockverse.default.cyan", "Cyan", "cyan"),
];

pub fn all() -> &'static [DefaultSkin] {
    &SKINS
}

pub fn by_id(id: &str) -> Option<&'static DefaultSkin> {
    SKINS.iter().find(|s| s.id == id)
}

pub fn pick(seed: u64) -> &'static DefaultSkin {
    &SKINS[(seed % SKINS.len() as u64) as usize]
}

pub fn is_plausible_skin(width: u32, height: u32, data_len: usize) -> bool {
    if width == 0 || height == 0 || width > MAX_SKIN_DIMENSION || height > MAX_SKIN_DIMENSION {
        return false;
    }
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        == Some(data_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(skin: &DefaultSkin, x: usize, y: usize) -> [u8; 4] {
        let i = (y * SKIN_WIDTH as usize + x) * 4;
        [skin.rgba[i], skin.rgba[i + 1], skin.rgba[i + 2], skin.rgba[i + 3]]
    }

    #[test]
    fn there_are_twelve_skins_with_unique_ids() {
        assert_eq!(all().len(), 12);
        let mut ids: Vec<_> = all().iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 12);
    }

    #[test]
    fn every_skin_has_the_right_size() {
        for s in all() {
            assert_eq!(s.rgba.len(), SKIN_BYTES, "{}", s.id);
            assert_eq!((s.width, s.height), (64, 64));
        }
    }

    #[test]
    fn base_colors_and_accent() {
        let expected: [(&str, [u8; 3], [u8; 3]); 12] = [
            ("blockverse.default.black", [0x00, 0x00, 0x00], [0x3b, 0x82, 0xff]),
            ("blockverse.default.charcoal", [0x23, 0x25, 0x2b], [0x22, 0xd3, 0xee]),
            ("blockverse.default.silver", [0xd5, 0xd8, 0xde], [0x2a, 0x5b, 0xd7]),
            ("blockverse.default.blue", [0x2a, 0x5b, 0xd7], [0x00, 0x48, 0xff]),
            ("blockverse.default.purple", [0x5b, 0x3f, 0xc9], [0x34, 0x00, 0xff]),
            ("blockverse.default.pink", [0xd6, 0x47, 0x9b], [0xff, 0x00, 0x96]),
            ("blockverse.default.crimson", [0xa6, 0x2b, 0x35], [0xff, 0x00, 0x15]),
            ("blockverse.default.amber", [0xc9, 0x70, 0x2a], [0xff, 0x70, 0x00]),
            ("blockverse.default.yellow", [0xd9, 0xb2, 0x1c], [0xff, 0xca, 0x00]),
            ("blockverse.default.green", [0x2f, 0x7d, 0x4f], [0x00, 0xff, 0x69]),
            ("blockverse.default.teal", [0x1f, 0x8a, 0x8a], [0x00, 0xff, 0xff]),
            ("blockverse.default.cyan", [0x1f, 0xa9, 0xd6], [0x00, 0xc0, 0xff]),
        ];
        for (id, base, accent) in expected {
            let s = by_id(id).unwrap();
            assert_eq!(pixel(s, 10, 10), [base[0], base[1], base[2], 255], "{id}");
            let want = [accent[0], accent[1], accent[2], 255];
            assert!(s.rgba.chunks_exact(4).any(|p| p == want), "{id}");
            assert_eq!(pixel(s, 0, 0)[3], 0, "{id}");
        }
    }

    #[test]
    fn pick_is_deterministic_and_spread() {
        assert_eq!(pick(0).id, pick(all().len() as u64).id);
        assert_ne!(pick(0).id, pick(1).id);
        assert_eq!(pick(u64::MAX).id, all()[(u64::MAX % all().len() as u64) as usize].id);
    }

    #[test]
    fn plausibility_check() {
        assert!(is_plausible_skin(64, 64, 16384));
        assert!(is_plausible_skin(64, 32, 8192));
        assert!(is_plausible_skin(128, 128, 65536));
        assert!(!is_plausible_skin(64, 64, 100));
        assert!(!is_plausible_skin(0, 64, 0));
        assert!(!is_plausible_skin(100_000, 100_000, 0));
    }
}
