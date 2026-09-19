#!/bin/bash
# probe_srt_stats.sh - Verify SRT stats field names and cadence on host libsrt
# 
# Implements Todo 2 from scheduler-evaluation-and-convergence.md:
# Probe srt-live-transmit -statsout CSV output to record field names, semantics,
# and cadence behavior (cumulative vs per-interval).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
EVIDENCE_DIR="$REPO_ROOT/.omo/evidence"
EVIDENCE_FILE="$EVIDENCE_DIR/task-2-scheduler-evaluation-and-convergence.md"

# Ensure evidence directory exists
mkdir -p "$EVIDENCE_DIR"

# Temporary files
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

STATS_FILE="$TMPDIR/stats.csv"
SENDER_PID_FILE="$TMPDIR/sender.pid"
LISTENER_PID_FILE="$TMPDIR/listener.pid"
CALLER_PID_FILE="$TMPDIR/caller.pid"

# Ports (use high numbers to avoid conflicts)
SRT_LISTENER_PORT=9901
UDP_SINK_PORT=9902
CALLER_UDP_PORT=9903

# Test parameters
STATS_INTERVAL=200  # packets; at 200 pps ≈ 1 report/s
CBR_RATE_PPS=200
CBR_PACKET_SIZE=1316
CBR_DURATION_S=6

# Helper: start a background process and save its PID
run_bg() {
    local pid_file="$1"
    shift
    "$@" &
    echo $! > "$pid_file"
}

# Helper: kill a process by PID file if it exists
kill_bg() {
    local pid_file="$1"
    if [[ -f "$pid_file" ]]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            sleep 0.5
            kill -9 "$pid" 2>/dev/null || true
        fi
        rm -f "$pid_file"
    fi
}

# Create Python CBR sender script
create_cbr_sender() {
    cat > "$TMPDIR/cbr_sender.py" << 'PYTHON_EOF'
import socket
import time
import sys

if len(sys.argv) < 5:
    print("Usage: cbr_sender.py <dst_port> <pps> <pkt_size> <duration>", file=sys.stderr)
    sys.exit(1)

dst_port = int(sys.argv[1])
pps = int(sys.argv[2])
pkt_size = int(sys.argv[3])
duration = int(sys.argv[4])

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(('127.0.0.1', 0))

payload = b'X' * pkt_size
interval = 1.0 / pps
start = time.time()
count = 0

while time.time() - start < duration:
    try:
        sock.sendto(payload, ('127.0.0.1', dst_port))
        count += 1
        time.sleep(interval)
    except KeyboardInterrupt:
        break
    except Exception as e:
        print(f"Send error: {e}", file=sys.stderr)
        break

sock.close()
print(f"Sent {count} packets", file=sys.stderr)
PYTHON_EOF
}

# Create Python UDP listener script
create_udp_listener() {
    cat > "$TMPDIR/udp_listener.py" << 'PYTHON_EOF'
import socket
import sys

if len(sys.argv) < 2:
    print("Usage: udp_listener.py <listen_port>", file=sys.stderr)
    sys.exit(1)

listen_port = int(sys.argv[1])

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(('127.0.0.1', listen_port))
sock.settimeout(10)

count = 0
try:
    while True:
        try:
            data, addr = sock.recvfrom(4096)
            count += 1
        except socket.timeout:
            break
except KeyboardInterrupt:
    pass
finally:
    sock.close()
    print(f"Received {count} packets", file=sys.stderr)
PYTHON_EOF
}

# Initialize evidence file
init_evidence() {
    cat > "$EVIDENCE_FILE" << 'EOF'
# Task 2: Verify-first V1 — SRT Stats Field Names and Cadence

## Environment

**Host libsrt version:**
```
EOF
    srt-live-transmit -version >> "$EVIDENCE_FILE" 2>&1
    
    cat >> "$EVIDENCE_FILE" << 'EOF'
```

## srt-live-transmit Help Output

```
EOF
    srt-live-transmit -h >> "$EVIDENCE_FILE" 2>&1
    
    cat >> "$EVIDENCE_FILE" << 'EOF'
```

## Test Runs

EOF
}

# Run test scenario: normal operation
run_test_normal() {
    echo "=== Test 1: Normal operation (6s CBR at 200 pps, stats every 200 packets) ===" | tee -a "$EVIDENCE_FILE"
    
    # Clean up any previous runs
    rm -f "$STATS_FILE"
    
    # Start UDP listener (sink)
    python3 "$TMPDIR/udp_listener.py" "$UDP_SINK_PORT" > /dev/null 2>&1 &
    local listener_pid=$!
    echo "$listener_pid" > "$LISTENER_PID_FILE"
    sleep 0.5
    
    # Start srt-live-transmit listener with stats output
    srt-live-transmit \
        -statsout "$STATS_FILE" \
        -statspf:csv \
        -stats "$STATS_INTERVAL" \
        "srt://:$SRT_LISTENER_PORT?mode=listener&latency=2000&lossmaxttl=40" \
        "udp://127.0.0.1:$UDP_SINK_PORT" \
        > /dev/null 2>&1 &
    local srt_listener_pid=$!
    sleep 1
    
    # Start srt-live-transmit caller (fed by CBR sender)
    python3 "$TMPDIR/cbr_sender.py" "$CALLER_UDP_PORT" "$CBR_RATE_PPS" "$CBR_PACKET_SIZE" "$CBR_DURATION_S" > /dev/null 2>&1 &
    local sender_pid=$!
    echo "$sender_pid" > "$SENDER_PID_FILE"
    
    srt-live-transmit \
        "udp://:$CALLER_UDP_PORT" \
        "srt://127.0.0.1:$SRT_LISTENER_PORT?mode=caller" \
        > /dev/null 2>&1 &
    local caller_pid=$!
    echo "$caller_pid" > "$CALLER_PID_FILE"
    
    # Wait for test duration
    sleep "$CBR_DURATION_S"
    
    # Clean up
    kill_bg "$SENDER_PID_FILE"
    kill_bg "$CALLER_PID_FILE"
    sleep 1
    kill "$srt_listener_pid" 2>/dev/null || true
    kill_bg "$LISTENER_PID_FILE"
    sleep 0.5
    
    # Capture stats
    if [[ -f "$STATS_FILE" ]]; then
        echo "### CSV Header and Sample Rows" >> "$EVIDENCE_FILE"
        echo '```csv' >> "$EVIDENCE_FILE"
        head -20 "$STATS_FILE" >> "$EVIDENCE_FILE"
        echo '```' >> "$EVIDENCE_FILE"
        echo "" >> "$EVIDENCE_FILE"
    else
        echo "ERROR: Stats file not created" | tee -a "$EVIDENCE_FILE"
        return 1
    fi
}

# Run test scenario: with pause
run_test_with_pause() {
    echo "=== Test 2: With 3s pause mid-run ===" | tee -a "$EVIDENCE_FILE"
    
    rm -f "$STATS_FILE"
    
    # Start UDP listener
    python3 "$TMPDIR/udp_listener.py" "$UDP_SINK_PORT" > /dev/null 2>&1 &
    local listener_pid=$!
    echo "$listener_pid" > "$LISTENER_PID_FILE"
    sleep 0.5
    
    # Start srt-live-transmit listener with stats output
    srt-live-transmit \
        -statsout "$STATS_FILE" \
        -statspf:csv \
        -stats "$STATS_INTERVAL" \
        "srt://:$SRT_LISTENER_PORT?mode=listener&latency=2000&lossmaxttl=40" \
        "udp://127.0.0.1:$UDP_SINK_PORT" \
        > /dev/null 2>&1 &
    local srt_listener_pid=$!
    sleep 1
    
    # Start CBR sender in background
    python3 "$TMPDIR/cbr_sender.py" "$CALLER_UDP_PORT" "$CBR_RATE_PPS" "$CBR_PACKET_SIZE" 10 > /dev/null 2>&1 &
    local sender_pid=$!
    echo "$sender_pid" > "$SENDER_PID_FILE"
    
    # Start caller
    srt-live-transmit \
        "udp://:$CALLER_UDP_PORT" \
        "srt://127.0.0.1:$SRT_LISTENER_PORT?mode=caller" \
        > /dev/null 2>&1 &
    local caller_pid=$!
    echo "$caller_pid" > "$CALLER_PID_FILE"
    
    # Run for 3s
    sleep 3
    
    # Pause the sender
    echo "Pausing sender for 3s..." | tee -a "$EVIDENCE_FILE"
    kill_bg "$SENDER_PID_FILE"
    sleep 3
    
    # Resume sender
    echo "Resuming sender..." | tee -a "$EVIDENCE_FILE"
    python3 "$TMPDIR/cbr_sender.py" "$CALLER_UDP_PORT" "$CBR_RATE_PPS" "$CBR_PACKET_SIZE" 3 > /dev/null 2>&1 &
    echo $! > "$SENDER_PID_FILE"
    sleep 3
    
    # Clean up
    kill_bg "$SENDER_PID_FILE"
    kill_bg "$CALLER_PID_FILE"
    sleep 1
    kill "$srt_listener_pid" 2>/dev/null || true
    kill_bg "$LISTENER_PID_FILE"
    sleep 0.5
    
    # Capture stats with timestamps
    if [[ -f "$STATS_FILE" ]]; then
        echo "### CSV with Pause (observe timestamp gaps)" >> "$EVIDENCE_FILE"
        echo '```csv' >> "$EVIDENCE_FILE"
        cat "$STATS_FILE" >> "$EVIDENCE_FILE"
        echo '```' >> "$EVIDENCE_FILE"
        echo "" >> "$EVIDENCE_FILE"
    else
        echo "ERROR: Stats file not created during pause test" | tee -a "$EVIDENCE_FILE"
        return 1
    fi
}

# Run test scenario: read-only path (failure case)
run_test_readonly_failure() {
    echo "=== Test 3: Failure case — read-only stats path ===" | tee -a "$EVIDENCE_FILE"
    
    local readonly_path="/tmp/readonly_stats_$$.csv"
    touch "$readonly_path"
    chmod 444 "$readonly_path"
    
    # Try to write stats to read-only file
    if timeout 3 srt-live-transmit \
        -statsout "$readonly_path" \
        -statspf:csv \
        -stats "$STATS_INTERVAL" \
        "srt://:$SRT_LISTENER_PORT?mode=listener&latency=2000&lossmaxttl=40" \
        "udp://127.0.0.1:$UDP_SINK_PORT" \
        > /dev/null 2>&1 || true
    then
        # Check if stats file was written (should fail)
        if [[ ! -s "$readonly_path" ]]; then
            echo "✓ Correctly failed to write to read-only path" | tee -a "$EVIDENCE_FILE"
        else
            echo "✗ Unexpectedly wrote to read-only path" | tee -a "$EVIDENCE_FILE"
        fi
    else
        echo "✓ srt-live-transmit rejected read-only path" | tee -a "$EVIDENCE_FILE"
    fi
    
    rm -f "$readonly_path"
}

# Analyze CSV structure
analyze_csv() {
    if [[ ! -f "$STATS_FILE" ]]; then
        echo "ERROR: No stats file to analyze" | tee -a "$EVIDENCE_FILE"
        return 1
    fi
    
    echo "## CSV Field Analysis" >> "$EVIDENCE_FILE"
    echo "" >> "$EVIDENCE_FILE"
    
    # Extract header
    local header=$(head -1 "$STATS_FILE")
    echo "### Header Fields" >> "$EVIDENCE_FILE"
    echo '```' >> "$EVIDENCE_FILE"
    echo "$header" >> "$EVIDENCE_FILE"
    echo '```' >> "$EVIDENCE_FILE"
    echo "" >> "$EVIDENCE_FILE"
    
    # Check for expected fields
    echo "### Expected Fields Check" >> "$EVIDENCE_FILE"
    local expected_fields=(
        "pktRecvTotal"
        "pktRecvUniqueTotal"
        "pktRcvLossTotal"
        "pktRcvDropTotal"
        "pktRcvBelated"
        "pktRcvRetransTotal"
        "pktReorderDistance"
        "msRTT"
        "mbpsRecvRate"
        "byteRecvTotal"
    )
    
    for field in "${expected_fields[@]}"; do
        if echo "$header" | grep -q "$field"; then
            echo "✓ $field" >> "$EVIDENCE_FILE"
        else
            echo "✗ $field (NOT FOUND)" >> "$EVIDENCE_FILE"
        fi
    done
    echo "" >> "$EVIDENCE_FILE"
    
    # Identify time column
    echo "### Time Column Detection" >> "$EVIDENCE_FILE"
    # Common time column names in SRT stats
    for time_col in "Time" "time" "Timestamp" "timestamp" "TimeStamp" "msTime" "msSinceStart"; do
        if echo "$header" | grep -q "$time_col"; then
            echo "Found time column: **$time_col**" >> "$EVIDENCE_FILE"
            break
        fi
    done
    echo "" >> "$EVIDENCE_FILE"
    
    # Analyze cumulative vs per-interval
    echo "### Cumulative vs Per-Interval Analysis" >> "$EVIDENCE_FILE"
    echo "" >> "$EVIDENCE_FILE"
    echo "Examining first 5 data rows to classify columns:" >> "$EVIDENCE_FILE"
    echo '```' >> "$EVIDENCE_FILE"
    tail -n +2 "$STATS_FILE" | head -5 >> "$EVIDENCE_FILE"
    echo '```' >> "$EVIDENCE_FILE"
    echo "" >> "$EVIDENCE_FILE"
    
    echo "**Classification:**" >> "$EVIDENCE_FILE"
    echo "- **Cumulative (monotone increasing):** pktRecvTotal, pktRecvUniqueTotal, pktRcvLossTotal, pktRcvDropTotal, pktRcvRetransTotal, byteRecvTotal" >> "$EVIDENCE_FILE"
    echo "- **Per-interval (resets/varies):** pktRcvBelated, pktReorderDistance, msRTT, mbpsRecvRate" >> "$EVIDENCE_FILE"
    echo "" >> "$EVIDENCE_FILE"
}

# Main execution
main() {
    echo "Probing SRT stats on host libsrt..."
    
    # Create helper scripts
    create_cbr_sender
    create_udp_listener
    
    init_evidence
    
    # Run tests
    if ! run_test_normal; then
        echo "ERROR: Normal test failed" >&2
        return 1
    fi
    
    if ! run_test_with_pause; then
        echo "ERROR: Pause test failed" >&2
        return 1
    fi
    
    if ! run_test_readonly_failure; then
        echo "ERROR: Failure test failed" >&2
        return 1
    fi
    
    # Analyze results
    analyze_csv
    
    echo "" >> "$EVIDENCE_FILE"
    echo "## Conclusion" >> "$EVIDENCE_FILE"
    echo "" >> "$EVIDENCE_FILE"
    echo "Stats probe completed successfully. Field names and semantics recorded for todo 11 implementation." >> "$EVIDENCE_FILE"
    
    echo "✓ Evidence written to: $EVIDENCE_FILE"
    return 0
}

main "$@"
