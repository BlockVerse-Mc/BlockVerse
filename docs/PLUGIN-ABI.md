# Native plugin ABI (v1)

Header: `native/include/blockverse_plugin.h`. License: Apache-2.0.

Rules
- Everything crossing the boundary is plain C. No C++ classes, Rust types, exceptions or panics may cross it.
- Strings are UTF-8, NUL-terminated, owned by the caller and valid only during the call.
- Every struct starts with `size` so fields can be appended without breaking old plugins.
- All entry points run on the server main thread.
- `BV_ABI_VERSION` changes only for breaking changes; the host rejects plugins with a different version.

Entry points exported by every plugin
- `const BV_PluginInfo* bv_plugin_info(void)`
- `int32_t bv_plugin_load(const BV_HostApi* host)` returns 0 on success
- `void bv_plugin_event(BV_Event* event)`
- `void bv_plugin_unload(void)`

A plugin crash takes the whole server down. Process isolation is a possible later step.
