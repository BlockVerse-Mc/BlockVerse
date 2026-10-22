#include "blockverse_plugin.h"

static const BV_PluginInfo g_info = { sizeof(BV_PluginInfo), 999u, "BadAbi", "0.0.1" };

BV_PLUGIN_API const BV_PluginInfo* bv_plugin_info(void) { return &g_info; }
BV_PLUGIN_API int32_t bv_plugin_load(const BV_HostApi*) { return 0; }
BV_PLUGIN_API void bv_plugin_event(BV_Event*) {}
BV_PLUGIN_API void bv_plugin_unload(void) {}
