use std::collections::HashMap;

use crate::logger;

pub fn run() {
    // Control
    logger::info("[selftest] control: allocating and freeing memory");
    let mut items: Vec<String> = Vec::new();
    for i in 0..1000 {
        items.push(format!("item {i}"));
    }
    drop(items);
    logger::success("[selftest] control passed");

    // NBT
    logger::info("[selftest] nbt: writing an empty compound (network format)");
    let mut buffer: Vec<u8> = Vec::new();
    let empty: HashMap<String, nbtx::Value> = HashMap::new();
    match nbtx::to_varint_bytes_in(&mut buffer, &empty) {
        Ok(()) => logger::success(&format!("[selftest] nbt passed ({} bytes: {:?})", buffer.len(), buffer)),
        Err(e) => logger::error(&format!("[selftest] nbt returned an error: {e}")),
    }

    logger::success("[selftest] finished");
}
