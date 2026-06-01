# Product Vision and Research Notes

BattleO should become an evolutionary battle sandbox: a browser-visible artificial life world where agents compete, survive, reproduce, diverge into strategies, and leave an inspectable evolutionary history.

The project should not stop at rendering many moving dots. The core experience is discovery: watching a population become better adapted, seeing lineages rise and collapse, and understanding which traits or behaviors caused that change.

## Current Gap

The cleaned-up architecture is a good foundation, but the simulation needs stronger product direction:

- Evolution must be repeatable enough to test and tune.
- Standard scenarios should survive long enough to produce later generations.
- Agents need meaningful choices between food, mates, threats, prey, escape, and combat.
- The browser UI should explain what evolved, not only show counts.
- Research runs should report lineage, diversity, survival, and strategy metrics.

## Design Principles

- **Evolution first**: tune for sustained selection pressure before adding visual polish.
- **Deterministic by default for experiments**: named scenarios should carry seeds so behavior can be reproduced.
- **Diversity over single-score optimization**: preserve and expose different viable strategies instead of optimizing one global fitness number.
- **Inspectable causality**: stats, diagrams, and UI should make it clear why populations grow, crash, split, or stabilize.
- **Small verified steps**: each iteration should add an observable behavior and a regression check.

## Research Basis

- NEAT shows that evolved controllers become more powerful when structure can grow over time and innovations are protected by speciation. That is a later-stage fit for BattleO once simpler heritable policies are working. See [Stanley and Miikkulainen, 2002](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf).
- MAP-Elites argues for mapping high-performing individuals across behavioral niches. BattleO should eventually track elites by traits such as aggression, diet, speed, size, and survival style instead of reporting only one average fitness. See [Mouret and Clune, 2015](https://arxiv.org/abs/1504.04909).
- Open-ended artificial life research frames the goal as ongoing adaptive novelty and complexity growth, not just stable population counts. BattleO should measure novelty and diversity as first-class outputs. See [An Overview of Open-Ended Evolution](https://arxiv.org/abs/1909.04430).
- Lenia is a useful product reference because it turns artificial life into discovery, taxonomy, and interactive exploration. BattleO should similarly make emergent behavior legible. See [Lenia - Biology of Artificial Life](https://arxiv.org/abs/1812.05433).
- Browser thread support still depends on cross-origin isolation when `SharedArrayBuffer` is needed. BattleO should keep documenting and testing COOP/COEP deployment constraints. See [web.dev COOP/COEP guidance](https://web.dev/articles/coop-coep).

## Iteration Roadmap

### 1. Reproducible Ecology Baseline

Goal: every named scenario has a deterministic seed and a regression expectation.

Done in the first vision iteration:

- `SimulationConfig` and `HeadlessConfig` support optional seeds.
- Scenario scripts pass fixed seeds.
- Resources regenerate and depleted resources are replenished to a scenario floor.
- Agent lifespan, starting energy, sensing, and reproduction pressure are tuned so seeded runs can reach later generations without immediate extinction or unbounded growth.
- Tests assert seeded reproducibility and later-generation survival.

### 2. Meaningful Agent Decisions

Goal: make `Seeking`, `Hunting`, `Feeding`, `Fleeing`, `Fighting`, and `Reproducing` real strategic states.

Done in the second vision iteration:

- Agents build a spatial snapshot of nearby resources and agents before mutating their own state.
- Stronger nearby threats can trigger `Fleeing`.
- Predator drive and aggression influence prey targeting, combat risk, and `Fighting`.
- Eligible agents seek nearby mates before the reproduction system pairs and spawns offspring.
- `SimulationStats`, headless output, and the browser UI expose current state distribution, predator/prey mix, reproduction candidates, births, and kills.
- Seeded regression coverage now proves survival, reproduction, later generations, and nonzero combat pressure.

### 3. Heritable Strategy Genome

Goal: evolve behavior, not only physical traits.

Next work:

- Add a compact policy genome that weights food, mates, threats, prey, crowding, energy, and age.
- Use the policy to choose actions.
- Mutate policy weights with the existing gene inheritance flow.
- Report average policy traits and strategy clusters.

### 4. Lineage and Diversity

Goal: make evolution explainable.

Next work:

- Add agent IDs and parent IDs.
- Track lineage depth, living descendants, and dominant ancestors.
- Add species or cluster assignment from trait distance.
- Add diversity metrics and generation histograms.

### 5. Browser Discovery UI

Goal: make the browser app feel like an evolutionary observatory.

Next work:

- Add scenario picker and seed display.
- Add pause, step, and time-scale controls.
- Add inspect-agent details for genes, energy, generation, lineage, and state.
- Add trait overlays, population timeline, and lineage summaries.

## Technology Direction

Keep the current Rust, WASM, `hecs`, and custom rendering stack while the simulation model is still changing quickly. A Bevy migration could be valuable if BattleO becomes more game-like, but it would slow down the immediate goal: making the ecosystem genuinely interesting.
