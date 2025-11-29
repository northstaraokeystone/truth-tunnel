# **Truth Tunnel: The Groot Line Swarm**

> Beneath Memphis soil,  
> Groot's roots entwine code and dirt—  
> Truth tunnels ignite.

**Thesis:** Truth Tunnel is a Rust monorepo of glyph‑native daemons that simulate, observe, and ledger the first 18.4 km Memphis “Groot Line” — Colossus ↔ FedEx Hub ↔ downtown — purely via CLI/gRPC/JSONL.  
**Tagline:** We are Groot. Backend-only daemons digging reality. No pixels. Just receipts that force physics to comply.

[![CI](https://github.com/northstaraokeystone/truth-tunnel/actions/workflows/ci.yml/badge.svg)](../../actions)  
![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)
![Mode: Backend Only](https://img.shields.io/badge/mode-backend--only-black.svg)

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
TENANT_ID, TENANT_CONFIG=./config/tenants/tenant_default.yaml
NATS_URL, NATS_JWT, NATS_SEED
TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_FROM_NUMBER
GROK_API_KEY
LEDGER_SQLITE_PATH / LEDGER_ROCKSDB_PATH
Everything should still run in “dry” mode with fake services if you leave secrets unset.
Architecture Overview
This repo is a Rust workspace. One crate per daemon; one swarm entrypoint:
src/crates/groot-swarm/ – Star‑Lord, global CLI entrypoint and daemon orchestrator (./groot-swarm ship).
src/crates/glyph-lib/ – shared glyph types, AnchorGlyph schemas, Merkle helpers.
src/crates/rocket-engine/ – Rocket, Prufrock/TBM simulation.
src/crates/spv-api/ – SPV/gRPC edge API for glyph submission + receipt retrieval.
src/crates/digital-twin-groot/ – Groot, Memphis tunnel digital twin + projections.
src/crates/portal-zero/ – CLI/gRPC ingress portal zero (first hole into the swarm).
src/crates/ledger-explorer/ – SQLite/RocksDB‑backed ledger explorer and queries.
src/crates/star-lord-orchestrator/ – orchestrator brain wired to config/orchestrator/*.
src/crates/mantis-community/ – Mantis, Twilio/Grok-voice hooks and phone ops.
src/crates/nebula-guard/ – Nebula, authN/Z and policy engine.
src/crates/drax-metrics/ – Drax, metrics, red‑loop, and health export.
Supporting structure:
daemons/phase*/ – 7‑phase 48h build daemons with phase.plan.yaml.
glyphs/ – canonical glyph schemas, examples, and the receipts ledger.
config/ – SLOs, agents, orchestrator routing, tenant configs.
scripts/ – red‑loop compaction, NATS bootstrap, ledger migrations, deploy manifests.
docs/Eng_Arch.md – high‑level architecture sketch (stub, but canonical source of truth).
Glyphs & Receipts
Everything in this swarm is a glyph. Flow: ingest → normalize → anchor → ledger → emit receipt
AnchorGlyph: minimal, tenant‑scoped statement about “what just happened.”
Receipt: Merkle‑anchored confirmation that a glyph was accepted and persisted.
All receipts are JSONL (glyphs/receipts/receipts.jsonl), one glyph per line, append‑only:
jsonl
Copy code
{"tenant_id":"demo","glyph_type":"anchor","anchor_id":"ANCHOR_0001","phase":"phase1-seed","payload":{"event":"tunnel_segment_started","segment_km":0.0},"ts":"2025-11-28T00:00:00Z","merkle_root":"[TBD]"}
Schemas live in glyphs/schemas/*.schema.json and mirrored inside src/crates/glyph-lib/glyphs/.
Agents & Theme (Guardians Org Chart)
Character	Codename Crate	Responsibility
Star‑Lord	groot-swarm, star-lord-orchestrator	CLI entry, daemon graph, process orchestration
Gamora	nebula-guard	Safety, auth, policy enforcement
Rocket	rocket-engine	TBM/Prufrock simulation and tunnel physics
Groot	digital-twin-groot	Memphis digital twin, tunnel state projections
Drax	drax-metrics	Metrics, alerts, red‑loop compaction targets
Mantis	mantis-community	Twilio/Grok-voice community & ops channels
Yondu/Kraglin	config/agents/*.yaml	Swarm role maps, xAI consensus & Librarian agents

The swarm self‑heals via Librarian agents (defined in config/agents/swarm_roles.yaml) that reconcile daemons against the ledger and restart/rewind when glyph chains disagree.
48h Ship Status
This repo is designed for the 48‑hour backend‑only sprint:
 Phase 0 – Skeleton: workspace, crates, configs stubbed (MERKLE_ROOT: [TBD])
 Phase 1 – Seed: first ANCHOR_0001 glyph written to glyphs/receipts/receipts.jsonl
 Phase 2 – Ledger: SPV API + ledger‑explorer round‑trip green
 Phase 3 – Twin: digital twin consumes ledger stream and projects tunnel state
 Phase 4 – Swarm: ./groot-swarm ship spins up a minimal but real daemon constellation
Update this section ruthlessly as receipts land. Rhetoric without receipts is a bug.
Unfinished Manifesto (Tease)
The philosophy, open questions, and failure criteria live in:
docs/UnfinishedManifesto.md
docs/open_questions.md
docs/death_criteria.md
Read those when you start to believe the tunnel is “done.” It isn’t.
Contribute / Ship
CI: GitHub Actions workflow (see .github/workflows/ci.yml) runs cargo fmt, cargo clippy, and cargo test.
Style: One daemon per crate, keep modules small (<300 lines), one responsibility per glyph.
Workflow:
Fork + branch (feat/rocket-noise, fix/anchor-merkle).
Add tests under tests/ or daemon‑local tests/ folder.
Open a PR with a link to at least one new/updated receipt glyph.
To join the dig locally:
bash
Copy code
cargo build
cargo run -p groot-swarm -- ship
Run ./groot-swarm ship to join the dig. We are Groot.