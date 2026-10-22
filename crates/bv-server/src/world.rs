pub const SUB_CHUNK_COUNT: u32 = 1;
pub const SPAWN_FEET_Y: i32 = -60;
pub const SPAWN_EYE_Y: f32 = -58.38;

const SECTIONS: usize = 24;
const BOTTOM_SECTION_INDEX: i8 = -4;
const BIOME_PLAINS: i32 = 1;

#[allow(dead_code)]
pub enum StateValue {
    Bool(bool),
    Int(i32),
    Str(&'static str),
}

// Block hash
pub fn fnv1a32(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811C_9DC5;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn push_text(out: &mut Vec<u8>, text: &str) {
    out.extend_from_slice(&(text.len() as u16).to_le_bytes());
    out.extend_from_slice(text.as_bytes());
}

pub fn block_state_nbt(name: &str, states: &[(&str, StateValue)]) -> Vec<u8> {
    let mut sorted: Vec<&(&str, StateValue)> = states.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = vec![10u8];
    push_text(&mut out, "");
    out.push(8);
    push_text(&mut out, "name");
    push_text(&mut out, name);
    out.push(10);
    push_text(&mut out, "states");
    for (key, value) in sorted {
        match value {
            StateValue::Bool(v) => {
                out.push(1);
                push_text(&mut out, key);
                out.push(*v as u8);
            }
            StateValue::Int(v) => {
                out.push(3);
                push_text(&mut out, key);
                out.extend_from_slice(&v.to_le_bytes());
            }
            StateValue::Str(v) => {
                out.push(8);
                push_text(&mut out, key);
                push_text(&mut out, v);
            }
        }
    }
    out.push(0);
    out.push(0);
    out
}

pub fn block_hash(name: &str, states: &[(&str, StateValue)]) -> i32 {
    fnv1a32(&block_state_nbt(name, states)) as i32
}

// Varints
pub fn write_var_u32(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

pub fn write_var_i32(out: &mut Vec<u8>, value: i32) {
    write_var_u32(out, ((value << 1) ^ (value >> 31)) as u32);
}

// Flat chunk
pub fn flat_palette() -> [i32; 4] {
    [
        block_hash("minecraft:air", &[]),
        block_hash("minecraft:bedrock", &[("infiniburn_bit", StateValue::Bool(false))]),
        block_hash("minecraft:stone", &[]),
        block_hash("minecraft:grass_block", &[]),
    ]
}

fn flat_layer_value(y: usize) -> u32 {
    match y {
        0 => 1,
        1 | 2 => 2,
        3 => 3,
        _ => 0,
    }
}

fn sub_chunk_bytes() -> Vec<u8> {
    let palette = flat_palette();
    let mut out = Vec::new();
    out.push(9);
    out.push(1);
    out.push(BOTTOM_SECTION_INDEX as u8);
    out.push((2 << 1) | 1);
    let mut words = [0u32; 256];
    for x in 0..16usize {
        for z in 0..16usize {
            for y in 0..16usize {
                let index = (x << 8) | (z << 4) | y;
                words[index / 16] |= flat_layer_value(y) << ((index % 16) * 2);
            }
        }
    }
    for word in words {
        out.extend_from_slice(&word.to_le_bytes());
    }
    write_var_i32(&mut out, palette.len() as i32);
    for entry in palette {
        write_var_i32(&mut out, entry);
    }
    out
}

fn biome_bytes(explicit: bool) -> Vec<u8> {
    let mut out = Vec::new();
    if explicit {
        for _ in 0..SECTIONS {
            out.push(1);
            write_var_i32(&mut out, BIOME_PLAINS);
        }
    } else {
        out.push(1);
        write_var_i32(&mut out, BIOME_PLAINS);
        for _ in 1..SECTIONS {
            out.push(0xFF);
        }
    }
    out
}

// Variants: 0 full chunk, 1 no sub chunks, 2 no sub chunks and no biomes, 3 full chunk with explicit biomes
pub fn chunk_variant(variant: u8) -> (u32, Vec<u8>) {
    let mut out = Vec::new();
    let count = match variant {
        1 | 2 => 0,
        _ => {
            out.extend(sub_chunk_bytes());
            SUB_CHUNK_COUNT
        }
    };
    match variant {
        2 => {}
        3 => out.extend(biome_bytes(true)),
        _ => out.extend(biome_bytes(false)),
    }
    out.push(0);
    (count, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_var_i32(data: &[u8], pos: &mut usize) -> i32 {
        let mut value: u32 = 0;
        let mut shift = 0;
        loop {
            let byte = data[*pos];
            *pos += 1;
            value |= ((byte & 0x7f) as u32) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        ((value >> 1) as i32) ^ -((value & 1) as i32)
    }

    #[test]
    fn air_hash_matches_the_known_runtime_id() {
        assert_eq!(block_hash("minecraft:air", &[]), -604_749_536);
    }

    #[test]
    fn state_order_does_not_change_the_hash() {
        let a = block_hash("minecraft:test", &[("a", StateValue::Int(1)), ("b", StateValue::Bool(true))]);
        let b = block_hash("minecraft:test", &[("b", StateValue::Bool(true)), ("a", StateValue::Int(1))]);
        assert_eq!(a, b);
        let c = block_hash("minecraft:test", &[("a", StateValue::Int(2)), ("b", StateValue::Bool(true))]);
        assert_ne!(a, c);
    }

    #[test]
    fn varints_use_zigzag_for_signed_values() {
        let mut out = Vec::new();
        write_var_i32(&mut out, 4);
        write_var_i32(&mut out, -1);
        assert_eq!(out, vec![8, 1]);
        let mut pos = 0;
        let mut big = Vec::new();
        write_var_i32(&mut big, -604_749_536);
        assert_eq!(read_var_i32(&big, &mut pos), -604_749_536);
    }

    #[test]
    fn chunk_payload_has_the_expected_layout() {
        let data = chunk_variant(0).1;
        assert_eq!(&data[..4], &[9, 1, 0xFC, 5]);
        let words: Vec<u32> = data[4..4 + 1024]
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        assert_eq!(words.len(), 256);

        let mut pos = 4 + 1024;
        assert_eq!(read_var_i32(&data, &mut pos), 4);
        let mut palette = Vec::new();
        for _ in 0..4 {
            palette.push(read_var_i32(&data, &mut pos));
        }
        assert_eq!(palette, flat_palette().to_vec());

        let mut tail = vec![1u8, 2u8];
        tail.extend(std::iter::repeat_n(0xFFu8, 23));
        tail.push(0);
        assert_eq!(&data[pos..], tail.as_slice());
    }

    #[test]
    fn chunk_variants_differ_as_documented() {
        let (count0, full) = chunk_variant(0);
        assert_eq!(count0, SUB_CHUNK_COUNT);
        let (count1, no_sub) = chunk_variant(1);
        assert_eq!(count1, 0);
        assert_eq!(&no_sub[..2], &[1, 2]);
        let (count2, bare) = chunk_variant(2);
        assert_eq!(count2, 0);
        assert_eq!(bare, vec![0]);
        let (count3, explicit) = chunk_variant(3);
        assert_eq!(count3, SUB_CHUNK_COUNT);
        assert_eq!(explicit.len(), full.len() + 23);
    }

    #[test]
    fn chunk_layers_are_bedrock_stone_grass_then_air() {
        let data = chunk_variant(0).1;
        let word_at = |index: usize| {
            let o = 4 + (index / 16) * 4;
            let w = u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
            (w >> ((index % 16) * 2)) & 3
        };
        let block = |x: usize, y: usize, z: usize| word_at((x << 8) | (z << 4) | y);
        assert_eq!(block(0, 0, 0), 1);
        assert_eq!(block(5, 1, 7), 2);
        assert_eq!(block(15, 2, 15), 2);
        assert_eq!(block(3, 3, 9), 3);
        assert_eq!(block(8, 4, 8), 0);
        assert_eq!(block(15, 15, 15), 0);
    }
}
