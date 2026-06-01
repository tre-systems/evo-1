# Architecture Diagrams

Graphviz/DOT sources plus rendered PNGs. The `.dot` files are the source of truth; the PNGs are committed for in-browser viewing on GitHub. Mermaid is only for small inline diagrams inside Markdown.

## Files

| Diagram | Source | Rendered |
| --- | --- | --- |
| System overview | `system-overview.dot` | `system-overview.png` |
| Frame pipeline | `frame-pipeline.dot` | `frame-pipeline.png` |
| Runtime model | `runtime-model.dot` | `runtime-model.png` |

## Reading Order

1. **System overview** for the browser, native, simulation, ECS, rendering, and diagnostics boundaries.
2. **Frame pipeline** before changing per-frame behavior, parallel work, spatial queries, reproduction, death, or resource lifecycle code.
3. **Runtime model** before changing configuration, ECS components, legacy DTO conversion, stats, or headless diagnostics.

## Conventions

Color coding by domain:

- Green nodes and clusters: simulation core, ECS world, and write-side systems.
- Yellow/orange nodes: time-driven or parallel-compute stages.
- Purple nodes: compatibility DTOs and adapter-facing translation boundaries.
- Teal nodes: runtime data structures and generated outputs.
- Blue nodes: browser, CLI, rendering, and read-only surfaces.
- Red nodes: lifecycle terminal/removal states.
- Diamonds: decisions.
- Bold green outline: terminal success or committed state.

Fonts: Avenir. Rendered at 220 DPI.

## Render

```bash
node scripts/render-diagrams.mjs
node scripts/check-diagrams.mjs
```

Both scripts assume Graphviz is on PATH (`brew install graphviz`). CI installs Graphviz before running the diagram check. On a local machine without `dot`, `node scripts/check-diagrams.mjs` skips with a clear message; generated PNGs should still be refreshed before committing diagram changes.

To render one manually:

```bash
dot -Tpng:cairo docs/diagrams/<name>.dot -Gdpi=220 -o docs/diagrams/<name>.png
```
