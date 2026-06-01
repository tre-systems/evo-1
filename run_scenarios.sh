#!/usr/bin/env bash
set -euo pipefail

# BattleO Simulation Scenarios Runner
# ===================================
#
# Curated collection of interesting evolution scenarios for reproducible experiments.
# Perfect for different evolution scenarios, performance benchmarking, and long-term studies.
#
# Usage: ./run_scenarios.sh [scenario_name]
#
# Examples:
#   ./run_scenarios.sh                    # List all available scenarios
#   ./run_scenarios.sh quick_test         # 30 seconds, fast evolution test
#   ./run_scenarios.sh stress_test        # 2 minutes, max CPU utilization
#   ./run_scenarios.sh evolution_test     # 15 minutes, focused evolution
#   ./run_scenarios.sh sustained_evolution # 30 minutes, long-term evolution
#
# Scenario Format: "name:duration:speed:agents:resources:max_agents:max_resources:seed"
# All scenarios automatically build, initialize rayon, run, and display results.

# Available scenarios
SCENARIOS=(
    "quick_test:0.5:10:100:100:500:500:101"
    "short_run:1.0:20:200:200:1000:800:202"
    "medium_run:5.0:15:500:500:2000:1500:303"
    "long_run:10.0:10:1000:1000:5000:3000:404"
    "stress_test:2.0:50:2000:2000:10000:5000:505"
    "evolution_test:15.0:5:300:300:1500:1000:606"
    "evolution_focused:10.0:8:400:600:1200:800:707"
    "balanced_evolution:20.0:3:200:400:800:600:808"
    "sustained_evolution:30.0:2:150:300:600:500:909"
)

# Function to run a scenario
run_scenario() {
    local scenario_name=$1
    local duration=$2
    local speed=$3
    local init_agents=$4
    local init_resources=$5
    local max_agents=$6
    local max_resources=$7
    local seed=$8
    
    echo "=== Running Scenario: $scenario_name ==="
    echo "Duration: ${duration} minutes"
    echo "Speed: ${speed}x"
    echo "Initial: ${init_agents} agents, ${init_resources} resources"
    echo "Max: ${max_agents} agents, ${max_resources} resources"
    echo "Seed: ${seed}"
    echo ""
    
    echo "Starting simulation..."
    cargo run --release --locked --bin headless -- \
        "$duration" \
        "$speed" \
        "$init_agents" \
        "$init_resources" \
        "$max_agents" \
        "$max_resources" \
        --seed "$seed"
    
    echo ""
    echo "✅ Scenario '$scenario_name' completed!"
    echo "=========================================="
    echo ""
}

# Main execution
if [ $# -eq 0 ]; then
    echo "=== BattleO Simulation Scenarios ==="
    echo "Available scenarios:"
    echo ""
    for scenario in "${SCENARIOS[@]}"; do
        IFS=':' read -r name duration speed agents resources max_agents max_resources seed <<< "$scenario"
        echo "  $name: ${duration}min, ${speed}x, ${agents}/${resources} initial, ${max_agents}/${max_resources} max, seed ${seed}"
    done
    echo ""
    echo "Usage: ./run_scenarios.sh [scenario_name]"
    echo "Example: ./run_scenarios.sh quick_test"
    exit 0
fi

# Find and run the requested scenario
REQUESTED_SCENARIO=$1
FOUND=false

for scenario in "${SCENARIOS[@]}"; do
    IFS=':' read -r name duration speed agents resources max_agents max_resources seed <<< "$scenario"
    if [ "$name" = "$REQUESTED_SCENARIO" ]; then
        run_scenario "$name" "$duration" "$speed" "$agents" "$resources" "$max_agents" "$max_resources" "$seed"
        FOUND=true
        break
    fi
done

if [ "$FOUND" = false ]; then
    echo "❌ Scenario '$REQUESTED_SCENARIO' not found!"
    echo "Available scenarios:"
    for scenario in "${SCENARIOS[@]}"; do
        IFS=':' read -r name _ _ _ _ _ _ <<< "$scenario"
        echo "  - $name"
    done
    exit 1
fi
