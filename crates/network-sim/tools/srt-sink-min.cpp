// One-caller bench sink. Link the selected lineage's apps/statswriter.cpp, so
// CSV columns and formatting are native, not a parallel hand-maintained schema.
// Counters are INTERVAL-valued (srt_bstats clear=1), every 1000 payload messages.
#include "srt.h"
#include "statswriter.hpp"

#include <arpa/inet.h>
#include <charconv>
#include <csignal>
#include <ctime>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

static volatile std::sig_atomic_t stopping = 0;
static void stop(int) { stopping = 1; }
static int checked(int result) {
    if (result == SRT_ERROR) throw std::runtime_error(srt_getlasterror_str());
    return result;
}
static int integer(std::string_view text) {
    int value = 0;
    const auto parsed = std::from_chars(text.data(), text.data() + text.size(), value);
    if (parsed.ec != std::errc{} || parsed.ptr != text.data() + text.size())
        throw std::runtime_error("expected integer: " + std::string(text));
    return value;
}
struct Runtime {
    Runtime() { checked(srt_startup()); }
    ~Runtime() { srt_cleanup(); }
};
struct Socket {
    SRTSOCKET id;
    explicit Socket(SRTSOCKET socket) : id(checked(socket)) {}
    Socket(const Socket&) = delete;
    Socket& operator=(const Socket&) = delete;
    ~Socket() { srt_close(id); }
    void option(int key, int value) const {
        checked(srt_setsockflag(id, static_cast<SRT_SOCKOPT>(key), &value, sizeof(value)));
    }
};

int main(int argc, char** argv) try {
    int port = 0, latency = -1;
    std::string stats_path, out_path;
    std::vector<std::pair<int, int>> options;
    for (int i = 1; i < argc; ++i) {
        const std::string key = argv[i];
        if (++i == argc) throw std::runtime_error("missing value for " + key);
        const std::string_view value = argv[i];
        if (key == "--port") port = integer(value);
        else if (key == "--latency") latency = integer(value);
        else if (key == "--statsout") stats_path = value;
        else if (key == "--out") out_path = value;
        else if (key == "--sockopt") {
            const auto equals = value.find('=');
            if (equals == std::string_view::npos) throw std::runtime_error("expected id=value");
            options.emplace_back(integer(value.substr(0, equals)), integer(value.substr(equals + 1)));
        } else throw std::runtime_error("unknown argument: " + key);
    }
    if (port < 1 || port > 65535 || latency < 0 || stats_path.empty() || out_path.empty())
        throw std::runtime_error("usage: srt-sink-min --port N --latency MS "
                                 "[--sockopt id=int ...] --statsout CSV --out FILE");
    if (stats_path == out_path) throw std::runtime_error("CSV and payload paths must differ");
    std::ofstream stats, out;
    stats.exceptions(std::ios::badbit | std::ios::failbit);
    out.exceptions(std::ios::badbit | std::ios::failbit);
    stats.open(stats_path);
    out.open(out_path, std::ios::binary);
    std::signal(SIGINT, stop);
    std::signal(SIGTERM, stop);
    const Runtime runtime;
    const Socket listener(srt_create_socket());
    listener.option(SRTO_TRANSTYPE, SRTT_LIVE);
    listener.option(SRTO_LATENCY, latency);
    listener.option(SRTO_RCVTIMEO, 200); // bounded shutdown, including an idle caller
    for (const auto& option : options) listener.option(option.first, option.second);
    listener.option(SRTO_RCVSYN, 0); // accept is polled without an unbounded wait
    sockaddr_in address{};
    address.sin_family = AF_INET;
    address.sin_port = htons(static_cast<uint16_t>(port));
    address.sin_addr.s_addr = htonl(INADDR_ANY);
    checked(srt_bind(listener.id, reinterpret_cast<sockaddr*>(&address), sizeof(address)));
    checked(srt_listen(listener.id, 1));
    std::cerr << "listening port=" << port << '\n';
    SRTSOCKET accepted = SRT_INVALID_SOCK;
    while (!stopping) {
        accepted = srt_accept(listener.id, nullptr, nullptr);
        if (accepted != SRT_INVALID_SOCK) break;
        if (srt_getlasterror(nullptr) != SRT_EASYNCRCV) checked(SRT_ERROR);
        timespec pause{0, 20'000'000};
        nanosleep(&pause, nullptr);
    }
    if (stopping) return 0;
    const Socket caller(accepted);
    caller.option(SRTO_RCVSYN, 1);
    caller.option(SRTO_RCVTIMEO, 200);
    const auto writer = SrtStatsWriterFactory(SRTSTATS_PROFMAT_CSV);
    const auto sample = [&] {
        SRT_TRACEBSTATS mon{};
        checked(srt_bstats(caller.id, &mon, 1));
        stats << writer->WriteStats(caller.id, mon) << std::flush;
    };
    char buffer[65536];
    unsigned messages = 0;
    bool peer_closed = false;
    while (!stopping) {
        const int size = srt_recvmsg(caller.id, buffer, sizeof(buffer));
        if (size == SRT_ERROR) {
            const int error = srt_getlasterror(nullptr);
            if (error == SRT_ETIMEOUT) continue;
            if (error == SRT_ECONNLOST) {
                peer_closed = true;
                break;
            }
            checked(size);
        }
        if (size == 0) break;
        out.write(buffer, size);
        if (++messages % 1000 == 0) sample();
    }
    // libsrt rejects bstats after ECONNLOST (also its normal peer-close signal).
    // Never invent a final row: retain only the successfully captured intervals.
    if (!peer_closed) sample();
    out.close();
    stats.close();
    return 0;
} catch (const std::exception& error) {
    std::cerr << "srt-sink-min: " << error.what() << '\n';
    return 1;
}
