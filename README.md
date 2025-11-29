# **Truth Tunnel: The Groot Line Swarm**

> Beneath Memphis soil,  
> Groot's roots entwine code and dirt—  
> Truth tunnels ignite.

**Thesis:** Truth Tunnel is a Rust monorepo of glyph‑native daemons that simulate, observe, and ledger the first 18.4 km Memphis “Groot Line” — Colossus ↔ FedEx Hub ↔ downtown ↔ orbit — purely via CLI/gRPC/JSONL.  

**Tagline:** We are Groot. Backend-only daemons digging reality. No pixels. Just receipts that force physics to comply (now with ZK + quantum foresight at the edge).

[![CI](https://github.com/northstaraokeystone/truth-tunnel/actions/workflows/ci.yml/badge.svg)](../../actions)  
![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)
![Mode: Backend Only](https://img.shields.io/badge/mode-backend--only-black.svg)

---

## Strategy Snapshot – xAI Orbital Challenge Layer

This repo now carries the **Grok/xAI orbital challenge** as a first‑class backend pattern:

- **ZK anomaly shares:** `nebula-guard` calls into `glyph-lib` to generate and verify Groth16‑style proofs over orbital anomaly models. Peers see **proofs**, not private drift deltas. Receipts in `glyphs/receipts/receipts.jsonl` gain optional ZK fields.
- **Quantum foresight, not FTL:** `glyph-lib` exposes a small quantum sim helper used by `nebula-guard` and `digital-twin-groot` to score anomaly scenarios with a Bell‑style correlation and an **effective latency** prediction. Physics stays honest; the swarm just guesses early.
- **verify_orbital as lie detector:** Nebula’s `verify_orbital` flow hashes observations, checks ZK proofs, consults the quantum model, and only then mints an `OrbitalGlyph` or an `AnomalyReceipt`. If correlation drops below threshold, you get a fraud receipt, not a shrug.
- **Same file map, deeper behavior:** All of this lives inside the **existing** crates and paths (`glyphs/`, `config/`, `nebula-guard`, `digital-twin-groot`, `tests/`). No new docs; just more truth squeezed through the current glyph schemas.

---

## Quick Start

Bare minimum: Rust, NATS, SQLite/RocksDB, and a terminal you’re not afraid of.

```bash
# clone
git clone https://github.com/northstaraokeystone/truth-tunnel.git
cd truth-tunnel

# env: copy and edit as needed (tenant_id, Twilio, Grok, NATS, ledger paths)
cp config/env.example .env

# bootstrap: build all crates + run smoke swarm
chmod +x ship-all.sh
./ship-all.sh

# or explicitly run the orchestrator entrypoint
cargo build
cargo run -p groot-swarm -- ship

Key env knobs (see config/env.example):
	•	TENANT_ID, TENANT_CONFIG=./config/tenants/tenant_default.yaml
	•	NATS_URL, NATS_JWT, NATS_SEED
	•	TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_FROM_NUMBER
	•	GROK_API_KEY
	•	LEDGER_SQLITE_PATH / LEDGER_ROCKSDB_PATH

Everything should still run in “dry” mode with fake services if you leave secrets unset. ZK + quantum paths degrade gracefully to classic hashes and metrics.

⸻

Architecture Overview

This repo is a Rust workspace. One crate per daemon; one swarm entrypoint:
	•	src/crates/groot-swarm/ – Star‑Lord, global CLI entrypoint and daemon orchestrator (./groot-swarm ship).
	•	src/crates/glyph-lib/ – shared glyph types, AnchorGlyph schemas, Merkle helpers, plus ZK + quantum helper modules used by Nebula and the twin.
	•	src/crates/rocket-engine/ – Rocket, Prufrock/TBM simulation and tunnel physics.
	•	src/crates/spv-api/ – SPV/gRPC edge API for glyph submission + receipt retrieval (including orbital anomaly glyphs).
	•	src/crates/digital-twin-groot/ – Groot, Memphis tunnel digital twin + projections, now also ingesting orbital/entanglement glyphs for foresight.
	•	src/crates/portal-zero/ – CLI/gRPC ingress portal zero (first hole into the swarm).
	•	src/crates/ledger-explorer/ – SQLite/RocksDB‑backed ledger explorer and queries over every glyph type.
	•	src/crates/star-lord-orchestrator/ – orchestrator brain wired to config/orchestrator/*.
	•	src/crates/mantis-community/ – Mantis, Twilio/Grok-voice hooks and phone ops.
	•	src/crates/nebula-guard/ – Nebula, authN/Z and policy engine with verify_orbital ZK + quantum lie detection for anomalies.
	•	src/crates/drax-metrics/ – Drax, metrics, alerts, red‑loop compaction targets, including Bell‑style correlation and ZK health gauges.

Supporting structure (unchanged):
	•	daemons/phase*/ – 7‑phase 48h build daemons with phase.plan.yaml.
	•	glyphs/ – canonical glyph schemas, examples, and the receipts ledger.
	•	config/ – SLOs, agents, orchestrator routing, tenant configs (including Nebula’s ZK quorum + orbital roles).
	•	scripts/ – red‑loop compaction, NATS bootstrap, ledger migrations, deploy manifests.
	•	docs/Eng_Arch.md – high‑level architecture sketch (stub, but canonical source of truth).

⸻

Glyphs & Receipts

Everything in this swarm is a glyph.

Flow: ingest → normalize → anchor → ZK/quantum (optional) → ledger → emit receipt
	•	AnchorGlyph: minimal, tenant‑scoped statement about “what just happened.”
	•	Receipt: Merkle‑anchored confirmation that a glyph was accepted and persisted, optionally enriched with:
	•	a ZK proof stub (e.g., zk_proof, public_inputs_hash), and
	•	an entanglement stub (e.g., bell_correlation, negation_ms).

All receipts are JSONL (glyphs/receipts/receipts.jsonl), one glyph per line, append‑only:

{"tenant_id":"demo","glyph_type":"anchor","anchor_id":"ANCHOR_0001","phase":"phase1-seed","payload":{"event":"tunnel_segment_started","segment_km":0.0},"ts":"2025-11-28T00:00:00Z","merkle_root":"[TBD]"}

Orbital flows simply extend the payload shape (within the existing schemas) instead of inventing new file paths. Schemas live in glyphs/schemas/*.schema.json and are mirrored inside src/crates/glyph-lib/glyphs/.

⸻

Agents & Theme (Guardians Org Chart)

Character	Codename Crate	Responsibility
Star‑Lord	groot-swarm, star-lord-orchestrator	CLI entry, daemon graph, process orchestration
Gamora	nebula-guard	Safety, auth, ZK policy enforcement, verify_orbital contracts
Rocket	rocket-engine	TBM/Prufrock simulation and tunnel physics
Groot	digital-twin-groot	Memphis digital twin, tunnel + orbital state projections
Drax	drax-metrics	Metrics, alerts, red‑loop + ZK/entanglement health thresholds
Mantis	mantis-community	Twilio/Grok-voice community & ops channels
Yondu/Kraglin	config/agents/*.yaml	Swarm role maps, xAI consensus & Librarian agents

The swarm self‑heals via Librarian agents (defined in config/agents/swarm_roles.yaml) that reconcile daemons against the ledger and restart/rewind when glyph chains disagree — including mismatched ZK proofs or suspicious entanglement scores.

⸻

48h Ship Status

This repo is designed for the 48‑hour backend‑only sprint:
	•	Phase 0 – Skeleton: workspace, crates, configs stubbed (MERKLE_ROOT: [TBD])
	•	Phase 1 – Seed: first ANCHOR_0001 glyph written to glyphs/receipts/receipts.jsonl
	•	Phase 2 – Ledger: SPV API + ledger‑explorer round‑trip green
	•	Phase 3 – Twin: digital twin consumes ledger stream and projects tunnel state
	•	Phase 4 – Swarm: ./groot-swarm ship spins up a minimal but real daemon constellation
	•	Phase 5 – Orbital ZK: Nebula’s verify_orbital flow emitting ZK‑backed anomaly receipts
	•	Phase 6 – Quantum Foresight: entanglement metrics feeding Drax + Groot’s projections

Update this section ruthlessly as receipts land. Rhetoric without receipts is a bug.

⸻

Unfinished Manifesto (Tease)

The philosophy, open questions, and failure criteria live in:
	•	docs/UnfinishedManifesto.md￼
	•	docs/open_questions.md￼
	•	docs/death_criteria.md￼

When you’re tempted to believe the tunnel is “done,” go read those and then ship another glyph.

⸻

Contribute / Ship
	•	CI: GitHub Actions workflow (see .github/workflows/ci.yml) runs cargo fmt, cargo clippy, and cargo test.
	•	Style: One daemon per crate, keep modules small (<300 lines), one responsibility per glyph. ZK and quantum logic belong in glyph-lib and are consumed via clean APIs, not re‑implemented everywhere.
	•	Workflow:
	1.	Fork + branch (feat/nebula-verify-orbital, fix/entanglement-threshold).
	2.	Add tests under tests/ or daemon‑local tests/ folder.
	3.	Open a PR with a link to at least one new/updated receipt glyph showing the behavior.

To join the dig locally:

cargo build
cargo run -p groot-swarm -- ship

Run ./groot-swarm ship to join the dig. We are Groot.

