#include <cstring>
#include <string>

#include "blockverse_plugin.h"

static const BV_HostApi* g_host = nullptr;

static const BV_PluginInfo g_info = { sizeof(BV_PluginInfo), BV_ABI_VERSION, "HelloPlugin", "0.1.0" };

BV_PLUGIN_API const BV_PluginInfo* bv_plugin_info(void) { return &g_info; }

BV_PLUGIN_API int32_t bv_plugin_load(const BV_HostApi* host) {
    if (!host || host->size < sizeof(BV_HostApi)) return 1;
    g_host = host;
    g_host->log(g_host->ctx, BV_LOG_INFO, "HelloPlugin enabled");
    return 0;
}

BV_PLUGIN_API void bv_plugin_event(BV_Event* ev) {
    if (!ev || !g_host) return;
    try {
        if (ev->type == BV_EVENT_PLAYER_JOIN) {
            std::string msg = std::string("Welcome to the server, ") + (ev->player_name ? ev->player_name : "player") + "!";
            g_host->send_message(g_host->ctx, ev->player_id, msg.c_str());
        } else if (ev->type == BV_EVENT_PLAYER_CHAT && ev->text && std::strstr(ev->text, "badword")) {
            ev->cancelled = 1;
            g_host->send_message(g_host->ctx, ev->player_id, "Please keep the chat clean.");
            g_host->log(g_host->ctx, BV_LOG_WARN, "Blocked a chat message");
        }
    } catch (...) {
    }
}

BV_PLUGIN_API void bv_plugin_unload(void) {
    if (g_host) g_host->log(g_host->ctx, BV_LOG_INFO, "HelloPlugin disabled");
    g_host = nullptr;
}
