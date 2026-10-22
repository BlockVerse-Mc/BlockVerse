/* SPDX-License-Identifier: Apache-2.0 */

#ifndef BLOCKVERSE_PLUGIN_H
#define BLOCKVERSE_PLUGIN_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define BV_ABI_VERSION 1u

#if defined(_WIN32)
#  define BV_EXPORT __declspec(dllexport)
#else
#  define BV_EXPORT __attribute__((visibility("default")))
#endif

#ifdef __cplusplus
#  define BV_PLUGIN_API extern "C" BV_EXPORT
#else
#  define BV_PLUGIN_API BV_EXPORT
#endif

typedef enum BV_LogLevel {
    BV_LOG_INFO  = 0,
    BV_LOG_WARN  = 1,
    BV_LOG_ERROR = 2
} BV_LogLevel;

typedef enum BV_EventType {
    BV_EVENT_PLAYER_JOIN = 1,
    BV_EVENT_PLAYER_QUIT = 2,
    BV_EVENT_PLAYER_CHAT = 3
} BV_EventType;

typedef struct BV_Event {
    uint32_t    size;
    uint32_t    type;
    uint64_t    player_id;
    const char* player_name;
    const char* text;
    int32_t     cancelled;
} BV_Event;

typedef struct BV_HostApi {
    uint32_t size;
    uint32_t abi_version;
    void*    ctx;
    void    (*log)(void* ctx, int32_t level, const char* message);
    int32_t (*send_message)(void* ctx, uint64_t player_id, const char* message);
} BV_HostApi;

typedef struct BV_PluginInfo {
    uint32_t    size;
    uint32_t    abi_version;
    const char* name;
    const char* version;
} BV_PluginInfo;

typedef const BV_PluginInfo* (*bv_plugin_info_fn)(void);
typedef int32_t              (*bv_plugin_load_fn)(const BV_HostApi*);
typedef void                 (*bv_plugin_event_fn)(BV_Event*);
typedef void                 (*bv_plugin_unload_fn)(void);

#ifdef __cplusplus
}
#endif
#endif
