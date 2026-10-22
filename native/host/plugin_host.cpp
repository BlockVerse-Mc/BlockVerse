#include "plugin_host.hpp"

#include <dlfcn.h>

namespace blockverse {

namespace {

const char* level_name(int32_t level) {
    switch (level) {
        case BV_LOG_WARN:  return "WARN";
        case BV_LOG_ERROR: return "ERROR";
        default:           return "INFO";
    }
}

void host_log(void* ctx, int32_t level, const char* message) {
    auto* p = static_cast<LoadedPlugin*>(ctx);
    if (!p || !p->host) return;
    p->host->log(std::string("[") + level_name(level) + "][" + p->name + "] " + (message ? message : ""));
}

int32_t host_send_message(void* ctx, uint64_t player_id, const char* message) {
    auto* p = static_cast<LoadedPlugin*>(ctx);
    if (!p || !p->host || !message) return -1;
    return p->host->send(player_id, message);
}

std::string dl_error() {
    const char* e = dlerror();
    return e ? e : "unknown error";
}

}

PluginHost::PluginHost(LogSink log, MessageSink send) : log_(std::move(log)), send_(std::move(send)) {}

PluginHost::~PluginHost() { unload_all(); }

bool PluginHost::load(const std::string& path, std::string& error) {
    void* handle = dlopen(path.c_str(), RTLD_NOW | RTLD_LOCAL);
    if (!handle) { error = "dlopen failed: " + dl_error(); return false; }

    auto info_fn   = reinterpret_cast<bv_plugin_info_fn>(dlsym(handle, "bv_plugin_info"));
    auto load_fn   = reinterpret_cast<bv_plugin_load_fn>(dlsym(handle, "bv_plugin_load"));
    auto event_fn  = reinterpret_cast<bv_plugin_event_fn>(dlsym(handle, "bv_plugin_event"));
    auto unload_fn = reinterpret_cast<bv_plugin_unload_fn>(dlsym(handle, "bv_plugin_unload"));
    if (!info_fn || !load_fn || !event_fn || !unload_fn) {
        error = "missing required exports (bv_plugin_info/load/event/unload)";
        dlclose(handle);
        return false;
    }

    const BV_PluginInfo* info = info_fn();
    if (!info || info->size < sizeof(BV_PluginInfo) || !info->name || !info->version) {
        error = "invalid plugin info";
        dlclose(handle);
        return false;
    }
    if (info->abi_version != BV_ABI_VERSION) {
        error = "ABI mismatch: plugin uses v" + std::to_string(info->abi_version) +
                ", host expects v" + std::to_string(BV_ABI_VERSION);
        dlclose(handle);
        return false;
    }

    auto plugin = std::make_unique<LoadedPlugin>();
    plugin->path = path;
    plugin->name = info->name;
    plugin->version = info->version;
    plugin->handle = handle;
    plugin->on_event = event_fn;
    plugin->on_unload = unload_fn;
    plugin->host = this;
    plugin->api.size = sizeof(BV_HostApi);
    plugin->api.abi_version = BV_ABI_VERSION;
    plugin->api.ctx = plugin.get();
    plugin->api.log = host_log;
    plugin->api.send_message = host_send_message;

    if (load_fn(&plugin->api) != 0) {
        error = "plugin '" + plugin->name + "' failed to initialize";
        dlclose(handle);
        return false;
    }

    log("Loaded plugin " + plugin->name + " v" + plugin->version);
    plugins_.push_back(std::move(plugin));
    return true;
}

bool PluginHost::dispatch(BV_Event& event) {
    event.size = sizeof(BV_Event);
    bool cancelled = false;
    for (auto& p : plugins_) {
        p->on_event(&event);
        if (event.cancelled) cancelled = true;
    }
    return cancelled;
}

void PluginHost::unload_all() {
    for (auto it = plugins_.rbegin(); it != plugins_.rend(); ++it) {
        (*it)->on_unload();
        dlclose((*it)->handle);
        log("Unloaded plugin " + (*it)->name);
    }
    plugins_.clear();
}

}
