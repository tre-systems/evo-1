#!/bin/bash

# BattleO Headless Simulation Runner
# =================================
# 
# Flexible script for custom experiments and parameter testing.
# Perfect for quick experiments, parameter testing, and iterative development.
#
# Usage: ./run_simulation.sh [duration_minutes] [speed_multiplier] [initial_agents] [initial_resources] [max_agents] [max_resources]
#
# Examples:
#   ./run_simulation.sh                    # Default configuration
#   ./run_simulation.sh 1.0 10 200 200     # 1min, 10x, 200 agents/resources
#   ./run_simulation.sh 0.5 15 100 100 500 500  # Full custom configuration
#
# Parameters:
#   duration_minutes: Simulation duration in minutes (default: 2.0)
#   speed_multiplier: Speed multiplier vs real-time (default: 20.0)
#   initial_agents: Starting number of agents (default: 500)
#   initial_resources: Starting number of resources (default: 500)
#   max_agents: Maximum allowed agents (default: 3000)
#   max_resources: Maximum allowed resources (default: 2000)

# Default values
DURATION_MINUTES=${1:-2.0}
SPEED_MULTIPLIER=${2:-20.0}
INITIAL_AGENTS=${3:-500}
INITIAL_RESOURCES=${4:-500}
MAX_AGENTS=${5:-3000}
MAX_RESOURCES=${6:-2000}

echo "=== BattleO Headless Simulation Runner ==="
echo "Configuration:"
echo "  Duration: ${DURATION_MINUTES} minutes"
echo "  Speed multiplier: ${SPEED_MULTIPLIER}x"
echo "  Initial agents: ${INITIAL_AGENTS}"
echo "  Initial resources: ${INITIAL_RESOURCES}"
echo "  Max agents: ${MAX_AGENTS}"
echo "  Max resources: ${MAX_RESOURCES}"
echo ""

# Build the simulation
echo "Building simulation..."
cargo build --bin headless --target x86_64-apple-darwin --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Run the simulation
echo "Starting simulation..."
./target/x86_64-apple-darwin/release/headless "$DURATION_MINUTES" "$SPEED_MULTIPLIER" "$INITIAL_AGENTS" "$INITIAL_RESOURCES" "$MAX_AGENTS" "$MAX_RESOURCES"

echo ""
echo "=== Simulation Complete ===" 