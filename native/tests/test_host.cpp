#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>
#include <utility>

#include "plugin_host.hpp"

static int g_failed = 0;
#define CHECK(cond) do { if (!(cond)) { std::printf("FAIL line %d: %s\n", __LINE__, #cond); ++g_failed; } } while (0)

int main(int argc, char** argv) {
    if (argc < 4) { std::printf("usage: test_host <hello.so> <bad_abi.so> <no_exports.so>\n"); return 2; }
    const std::string hello = argv[1], bad = argv[2], no_exports = argv[3];

    std::vector<std::string> logs;
    std::vector<std::pair<uint64_t, std::string>> sent;
    {
        blockverse::PluginHost host(
            [&](const std::string& l) { logs.push_back(l); },
            [&](uint64_t id, const std::string& m) { sent.emplace_back(id, m); return 0; });

        std::string err;
        CHECK(host.load(hello, err));
        CHECK(host.count() == 1);

        BV_Event join{};
        join.type = BV_EVENT_PLAYER_JOIN; join.player_id = 7; join.player_name = "Steve";
        CHECK(!host.dispatch(join));
        CHECK(sent.size() == 1 && sent[0].first == 7 && sent[0].second == "Welcome to the server, Steve!");

        BV_Event ok{};
        ok.type = BV_EVENT_PLAYER_CHAT; ok.player_id = 7; ok.player_name = "Steve"; ok.text = "hello all";
        CHECK(!host.dispatch(ok));

        BV_Event bad_chat{};
        bad_chat.type = BV_EVENT_PLAYER_CHAT; bad_chat.player_id = 7; bad_chat.player_name = "Steve"; bad_chat.text = "this is a badword";
        CHECK(host.dispatch(bad_chat));
        CHECK(bad_chat.cancelled == 1);
        CHECK(sent.size() == 2 && sent[1].second == "Please keep the chat clean.");

        std::string err2;
        CHECK(!host.load(bad, err2));
        CHECK(err2.find("ABI mismatch") != std::string::npos);
        CHECK(host.count() == 1);

        std::string err3;
        CHECK(!host.load(no_exports, err3));
        CHECK(err3.find("missing required exports") != std::string::npos);

        std::string err4;
        CHECK(!host.load("/nonexistent/plugin.so", err4));
    }

    bool saw_load = false, saw_unload = false, saw_warn = false;
    for (auto& l : logs) {
        if (l.find("Loaded plugin HelloPlugin") != std::string::npos) saw_load = true;
        if (l.find("Unloaded plugin HelloPlugin") != std::string::npos) saw_unload = true;
        if (l.find("[WARN][HelloPlugin]") != std::string::npos) saw_warn = true;
    }
    CHECK(saw_load); CHECK(saw_unload); CHECK(saw_warn);

    for (auto& l : logs) std::printf("  log: %s\n", l.c_str());
    if (g_failed == 0) std::printf("ALL TESTS PASSED\n");
    return g_failed == 0 ? 0 : 1;
}
