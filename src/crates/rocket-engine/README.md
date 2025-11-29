# rocket-engine

High-performance, immediate‑mode 2D renderer for `.glyph` documents, tuned for 120–240+ FPS with zero hot‑path allocations.

---

## Design Principles & Goals

| Principle              | Description                                                                                         |
|------------------------|-----------------------------------------------------------------------------------------------------|
| Deterministic speed    | Stable 120–240+ FPS on modern GPUs, even with 10k+ complex paths and nested clips.                 |
| Allocation-free hot path | No per-frame heap allocations in the render loop (arena + bump allocators, ring-buffered uploads). |
| GPU-first architecture | Aggressive GPU caching (SDF atlas, gradient textures, image cache, clip masks) for minimal CPU work. |
| Full Glyph fidelity    | Renders the entire Glyph feature set as defined by `glyph-lib`, no “fast path only” compromises.   |
| Composable & safe      | Fully `Send + Sync`; multiple renderers can share a single `wgpu::Device` without stepping on each other. |
| Simple API surface     | One primary `Renderer` struct with async initialization and a small set of core methods.           |

---

## Feature Flags

All features are additive; defaults are optimized for typical desktop + server use.

| Feature           | Default | Description                                                                                 |
|-------------------|---------|---------------------------------------------------------------------------------------------|
| `sdf-text`        | `true`  | Enables SDF text rendering with GPU-accelerated glyph atlas and subpixel positioning.      |
| `gradient-cache`  | `true`  | Caches linear/radial/conic gradient ramps as GPU textures to avoid per-frame uploads.      |
| `clip-mask`       | `true`  | Enables nested clip masks using stencil or alpha-mask passes.                              |
| `image-cache`     | `true`  | Deduplicates and caches images across frames and documents.                                |
| `hot-reload`      | `false` | Optional live-reload of `.glyph` documents for tooling workflows.                          |
| `tracing`         | `false` | Emits `tracing` spans for frame phases, GPU submissions, and cache hits/misses.           |

Example `Cargo.toml`:

```toml
[dependencies]
rocket-engine = { version = "0.1", features = ["sdf-text", "gradient-cache", "clip-mask", "image-cache"] }