use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

pub const BV_ABI_VERSION: u32 = 1;

pub const BV_LOG_INFO: i32 = 0;
pub const BV_LOG_WARN: i32 = 1;
pub const BV_LOG_ERROR: i32 = 2;

pub const BV_EVENT_PLAYER_JOIN: u32 = 1;
pub const BV_EVENT_PLAYER_QUIT: u32 = 2;
pub const BV_EVENT_PLAYER_CHAT: u32 = 3;

#[repr(C)]
pub struct BvEvent {
    pub size: u32,
    pub type_: u32,
    pub player_id: u64,
    pub player_name: *const c_char,
    pub text: *const c_char,
    pub cancelled: i32,
}

#[repr(C)]
pub struct BvHostApi {
    pub size: u32,
    pub abi_version: u32,
    pub ctx: *mut c_void,
    pub log: Option<unsafe extern "C" fn(*mut c_void, i32, *const c_char)>,
    pub send_message: Option<unsafe extern "C" fn(*mut c_void, u64, *const c_char) -> i32>,
}

#[repr(C)]
pub struct BvPluginInfo {
    pub size: u32,
    pub abi_version: u32,
    pub name: *const c_char,
    pub version: *const c_char,
}

pub struct SyncPluginInfo(pub BvPluginInfo);
unsafe impl Sync for SyncPluginInfo {}

pub unsafe fn cstr_opt<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        None
    } else {
        CStr::from_ptr(p).to_str().ok()
    }
}

impl BvHostApi {
    pub unsafe fn write_log(&self, level: i32, msg: &str) {
        if let (Some(f), Ok(c)) = (self.log, CString::new(msg)) {
            f(self.ctx, level, c.as_ptr());
        }
    }

    pub unsafe fn send(&self, player_id: u64, msg: &str) -> i32 {
        if let (Some(f), Ok(c)) = (self.send_message, CString::new(msg)) {
            f(self.ctx, player_id, c.as_ptr())
        } else {
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn struct_sizes_match_c_header() {
        assert_eq!(size_of::<BvEvent>(), 40);
        assert_eq!(size_of::<BvHostApi>(), 32);
        assert_eq!(size_of::<BvPluginInfo>(), 24);
    }

    #[test]
    fn abi_version_is_one() {
        assert_eq!(BV_ABI_VERSION, 1);
    }
}
