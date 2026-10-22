#ifndef BLOCKVERSE_PLUGIN_HOST_HPP
#define BLOCKVERSE_PLUGIN_HOST_HPP

#include <cstdint>
#include <functional>
#include <memory>
#include <string>
#include <vector>

#include "blockverse_plugin.h"

namespace blockverse {

class PluginHost;

struct LoadedPlugin {
    std::string path;
    std::string name;
    std::string version;
    void* handle = nullptr;
    bv_plugin_event_fn  on_event  = nullptr;
    bv_plugin_unload_fn on_unload = nullptr;
    BV_HostApi api{};
    PluginHost* host = nullptr;
};

class PluginHost {
public:
    using LogSink     = std::function<void(const std::string&)>;
    using MessageSink = std::function<int32_t(uint64_t, const std::string&)>;

    PluginHost(LogSink log, MessageSink send);
    ~PluginHost();
    PluginHost(const PluginHost&) = delete;
    PluginHost& operator=(const PluginHost&) = delete;

    bool load(const std::string& path, std::string& error);

    bool dispatch(BV_Event& event);

    void unload_all();
    size_t count() const { return plugins_.size(); }

    void log(const std::string& line) const { if (log_) log_(line); }
    int32_t send(uint64_t player, const std::string& msg) const { return send_ ? send_(player, msg) : -1; }

private:
    LogSink log_;
    MessageSink send_;
    std::vector<std::unique_ptr<LoadedPlugin>> plugins_;
};

}
#endif
