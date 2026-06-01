# Maintenance Notes

BattleO is a Rust evolutionary simulation with two supported execution paths:

- Native headless runs for experiments and smoke tests.
- WebAssembly browser runs with Canvas2D/WebGL rendering.

There is no configured live site for this repository at the moment. GitHub reports no Pages site and the repository homepage URL is empty, so browser smoke tests should use the local development server until a deployment target is added.

## Required Checks

Run these before pushing code changes:

```bash
./scripts/check.sh
```

`scripts/check.sh` runs formatting, clippy, tests, the stable WASM check, the pinned nightly threaded-WASM check, and the Graphviz diagram check.

For the threaded browser package build:

```bash
./scripts/build-wasm.sh
```

The threaded WASM path requires `nightly-2024-08-02` with `rust-src` and `wasm32-unknown-unknown` installed:

```bash
rustup toolchain install nightly-2024-08-02 --component rust-src --target wasm32-unknown-unknown
```

For a quick native behavior smoke test:

```bash
./run_simulation.sh 0.01 1 10 20 100 100
```

## Browser Smoke Test

After building `pkg/`, run:

```bash
python3 server.py
```

Then open `http://localhost:8000` and verify:

- The page reports that the simulation is ready.
- `Start Simulation` advances the stats.
- `Reset` returns the simulation to its configured initial population.
- The browser console does not show initialization errors.

If a live deployment is added later, document the production URL in the README and smoke test that URL after every pushed code change.

## Branch Cleanup

Current useful branch policy:

- `main` is the default branch.
- `feature/parallel-evolution-refactor` is the active cleanup branch.
- `pre-hecs` is fully contained in `main`; it is safe to delete once you no longer want it as a historical bookmark.
- Local `rayon` is also contained in `main` and its upstream branch is gone; it is safe to delete locally.

Suggested cleanup commands after the active branch is merged:

```bash
git branch -d rayon
git branch -d pre-hecs
git push origin --delete pre-hecs
```

Do not delete `feature/parallel-evolution-refactor` until the current cleanup work has been merged or otherwise preserved.
