
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::{Path, PathBuf};

use bv_plugin_abi::*;
use libloading::{Library, Symbol};

use crate::logger;

type InfoFn = unsafe extern "C" fn() -> *const BvPluginInfo;
type LoadFn = unsafe extern "C" fn(*const BvHostApi) -> i32;
type EventFn = unsafe extern "C" fn(*mut BvEvent);
type UnloadFn = unsafe extern "C" fn();
struct PluginCtx {
    name: String,
}

pub struct Plugin {
    pub name: String,
    pub version: String,
    on_event: EventFn,
    on_unload: UnloadFn,
    _api: Box<BvHostApi>,
    _ctx: Box<PluginCtx>,
    _lib: Library,
}
unsafe impl Send for Plugin {}

impl Drop for Plugin {
    fn drop(&mut self) {
        unsafe { (self.on_unload)() };
        logger::info(&format!("Unloaded plugin {}", self.name));
    }
}

#[derive(Default)]
pub struct PluginHost {
    plugins: Vec<Plugin>,
}

unsafe extern "C" fn host_log(ctx: *mut c_void, level: i32, msg: *const c_char) {
    if ctx.is_null() || msg.is_null() {
        return;
    }
    let ctx = unsafe { &*(ctx as *const PluginCtx) };
    let text = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    let line = format!("[{}] {}", ctx.name, text);
    match level {
        BV_LOG_WARN => logger::warn(&line),
        BV_LOG_ERROR => logger::error(&line),
        _ => logger::info(&line),
    }
}

unsafe extern "C" fn host_send_message(ctx: *mut c_void, player_id: u64, msg: *const c_char) -> i32 {
    if ctx.is_null() || msg.is_null() {
        return -1;
    }
    let ctx = unsafe { &*(ctx as *const PluginCtx) };
    let text = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    logger::info(&format!("[{}] -> player {}: {}", ctx.name, player_id, text));
    0
}

impl PluginHost {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn unload_all(&mut self) {
        while let Some(plugin) = self.plugins.pop() {
            drop(plugin);
        }
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    pub fn summary(&self) -> Vec<String> {
        self.plugins
            .iter()
            .map(|p| format!("{} v{}", p.name, p.version))
            .collect()
    }
    pub fn load_dir(&mut self, dir: &str) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => {
                if let Err(e) = std::fs::create_dir_all(dir) {
                    logger::warn(&format!("Cannot create plugin folder '{dir}': {e}"));
                } else {
                    logger::info(&format!("Created plugin folder '{dir}' (put native plugins there)"));
                }
                return;
            }
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                matches!(
                    p.extension().and_then(|x| x.to_str()),
                    Some("so") | Some("dll") | Some("dylib")
                )
            })
            .collect();
        paths.sort();
        for path in paths {
            if let Err(e) = self.load(&path) {
                logger::warn(&format!("Plugin {}: {}", path.display(), e));
            }
        }
    }

    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        unsafe {
            let lib = Library::new(path).map_err(|e| format!("cannot open: {e}"))?;
            let (info_fn, load_fn, event_fn, unload_fn) = {
                let info: Symbol<InfoFn> = lib
                    .get(b"bv_plugin_info\0")
                    .map_err(|e| format!("missing bv_plugin_info: {e}"))?;
                let load: Symbol<LoadFn> = lib
                    .get(b"bv_plugin_load\0")
                    .map_err(|e| format!("missing bv_plugin_load: {e}"))?;
                let event: Symbol<EventFn> = lib
                    .get(b"bv_plugin_event\0")
                    .map_err(|e| format!("missing bv_plugin_event: {e}"))?;
                let unload: Symbol<UnloadFn> = lib
                    .get(b"bv_plugin_unload\0")
                    .map_err(|e| format!("missing bv_plugin_unload: {e}"))?;
                (*info, *load, *event, *unload)
            };

            let info_ptr = info_fn();
            if info_ptr.is_null() {
                return Err("plugin returned no info".to_string());
            }
            let info = &*info_ptr;
            if (info.size as usize) < std::mem::size_of::<BvPluginInfo>() {
                return Err("plugin info is too small".to_string());
            }
            if info.abi_version != BV_ABI_VERSION {
                return Err(format!(
                    "ABI mismatch: plugin uses v{}, server expects v{}",
                    info.abi_version, BV_ABI_VERSION
                ));
            }
            let (Some(name), Some(version)) = (cstr_opt(info.name), cstr_opt(info.version)) else {
                return Err("plugin name or version is missing".to_string());
            };
            let (name, version) = (name.to_string(), version.to_string());

            let ctx = Box::new(PluginCtx { name: name.clone() });
            let api = Box::new(BvHostApi {
                size: std::mem::size_of::<BvHostApi>() as u32,
                abi_version: BV_ABI_VERSION,
                ctx: &*ctx as *const PluginCtx as *mut c_void,
                log: Some(host_log),
                send_message: Some(host_send_message),
            });

            if load_fn(&*api) != 0 {
                return Err(format!("plugin '{name}' failed to initialize"));
            }

            logger::success(&format!("Loaded plugin {name} v{version}"));
            self.plugins.push(Plugin {
                name,
                version,
                on_event: event_fn,
                on_unload: unload_fn,
                _api: api,
                _ctx: ctx,
                _lib: lib,
            });
        }
        Ok(())
    }
    pub fn dispatch(&mut self, event_type: u32, player_id: u64, player_name: &str, text: Option<&str>) -> bool {
        let name_c = CString::new(player_name.replace('\0', "")).unwrap_or_default();
        let text_c = text.map(|t| CString::new(t.replace('\0', "")).unwrap_or_default());
        let mut ev = BvEvent {
            size: std::mem::size_of::<BvEvent>() as u32,
            type_: event_type,
            player_id,
            player_name: name_c.as_ptr(),
            text: text_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            cancelled: 0,
        };
        for plugin in &self.plugins {
            unsafe { (plugin.on_event)(&mut ev) };
        }
        ev.cancelled != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("bv-test-{}-{name}", std::process::id()))
    }

    #[test]
    fn loading_a_missing_file_fails() {
        let mut host = PluginHost::new();
        assert!(host.load(Path::new("/nonexistent/plugin.so")).is_err());
        assert_eq!(host.count(), 0);
    }

    #[test]
    fn loading_a_file_that_is_not_a_library_fails() {
        let dir = temp_dir("fake");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("fake.so");
        fs::write(&file, b"this is not a shared library").unwrap();
        let mut host = PluginHost::new();
        assert!(host.load(&file).is_err());
        assert_eq!(host.count(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_dir_creates_a_missing_folder() {
        let dir = temp_dir("missing");
        let _ = fs::remove_dir_all(&dir);
        let mut host = PluginHost::new();
        host.load_dir(dir.to_str().unwrap());
        assert!(dir.is_dir());
        assert_eq!(host.count(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_dir_skips_broken_plugins_without_failing() {
        let dir = temp_dir("broken");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("broken.so"), b"junk").unwrap();
        fs::write(dir.join("readme.txt"), b"ignored").unwrap();
        let mut host = PluginHost::new();
        host.load_dir(dir.to_str().unwrap());
        assert_eq!(host.count(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dispatch_without_plugins_is_not_cancelled() {
        let mut host = PluginHost::new();
        assert!(!host.dispatch(BV_EVENT_PLAYER_JOIN, 1, "Steve", None));
        assert!(!host.dispatch(BV_EVENT_PLAYER_CHAT, 1, "Steve", Some("hi")));
    }

    #[test]
    fn dispatch_tolerates_interior_nul_bytes() {
        let mut host = PluginHost::new();
        assert!(!host.dispatch(BV_EVENT_PLAYER_CHAT, 1, "Ste\0ve", Some("a\0b")));
    }
}
