use bv_plugin_abi::*;
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicPtr, Ordering};

static HOST: AtomicPtr<BvHostApi> = AtomicPtr::new(std::ptr::null_mut());

static INFO: SyncPluginInfo = SyncPluginInfo(BvPluginInfo {
    size: std::mem::size_of::<BvPluginInfo>() as u32,
    abi_version: BV_ABI_VERSION,
    name: b"RustHello\0".as_ptr() as *const c_char,
    version: b"0.1.0\0".as_ptr() as *const c_char,
});

#[no_mangle]
pub extern "C" fn bv_plugin_info() -> *const BvPluginInfo {
    &INFO.0 as *const BvPluginInfo
}

#[no_mangle]
pub unsafe extern "C" fn bv_plugin_load(host: *const BvHostApi) -> i32 {
    if host.is_null() || ((*host).size as usize) < std::mem::size_of::<BvHostApi>() {
        return 1;
    }
    HOST.store(host as *mut BvHostApi, Ordering::SeqCst);
    (*host).write_log(BV_LOG_INFO, "RustHello enabled");
    0
}

#[no_mangle]
pub unsafe extern "C" fn bv_plugin_event(ev: *mut BvEvent) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let host = HOST.load(Ordering::SeqCst);
        if host.is_null() || ev.is_null() {
            return;
        }
        let host = &*host;
        let ev = &mut *ev;
        match ev.type_ {
            BV_EVENT_PLAYER_JOIN => {
                let name = cstr_opt(ev.player_name).unwrap_or("player");
                host.send(ev.player_id, &format!("Welcome to the server, {name}!"));
            }
            BV_EVENT_PLAYER_CHAT => {
                if let Some(text) = cstr_opt(ev.text) {
                    if text.contains("badword") {
                        ev.cancelled = 1;
                        host.send(ev.player_id, "Please keep the chat clean.");
                        host.write_log(BV_LOG_WARN, "Blocked a chat message");
                    }
                }
            }
            _ => {}
        }
    }));
}

#[no_mangle]
pub extern "C" fn bv_plugin_unload() {
    let host = HOST.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !host.is_null() {
        unsafe { (*host).write_log(BV_LOG_INFO, "RustHello disabled") };
    }
}
