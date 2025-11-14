#!/bin/bash
# Start Multiple Fast Forth Agent Servers
#
# This script starts N Fast Forth verification servers on sequential ports.
# Each server acts as an independent agent worker.
#
# Usage:
#   ./start_agent_servers.sh [num_agents]
#
# Example:
#   ./start_agent_servers.sh 10

NUM_AGENTS=${1:-10}
BASE_PORT=8080

echo "Starting $NUM_AGENTS Fast Forth agent servers..."
echo "Base port: $BASE_PORT"
echo ""

# Store PIDs for cleanup
PIDS=()

# Cleanup function
cleanup() {
    echo ""
    echo "Shutting down agent servers..."
    for pid in "${PIDS[@]}"; do
        kill $pid 2>/dev/null
    done
    echo "All agents stopped."
    exit 0
}

# Trap SIGINT (Ctrl+C) and SIGTERM
trap cleanup SIGINT SIGTERM

# Start agent servers
for i in $(seq 0 $((NUM_AGENTS - 1))); do
    PORT=$((BASE_PORT + i))

    # Start Fast Forth server on port
    fastforth-server --port $PORT &
    PID=$!
    PIDS+=($PID)

    echo "  Agent $i: http://localhost:$PORT (PID: $PID)"

    # Small delay to avoid port conflicts
    sleep 0.1
done

echo ""
echo "All $NUM_AGENTS agents started!"
echo "Press Ctrl+C to stop all servers"
echo ""

# Wait for all background processes
wait
