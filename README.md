# Truth Tunnel: The Groot Line Swarm

Truth Tunnel is a Rust monorepo of backend-only daemons that simulate, verify, and ledger the first 18.4 km Memphis “Groot Line” (Colossus ↔ FedEx Hub ↔ downtown ↔ orbit) via CLI, gRPC, and JSONL glyphs.

[![CI](https://github.com/northstaraokeystone/truth-tunnel/actions/workflows/ci.yml/badge.svg)](../../actions)
![Progress: 44/55](https://img.shields.io/badge/progress-44%2F55-brightgreen.svg)
![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)
![Mode: Backend Only](https://img.shields.io/badge/mode-backend--only-black.svg)

---

## KPI Dashboard (Nov 2025)

| Area            | KPI                                      | Current State (Files #1–#44)                                        | Target                                  | Evidence                                    |
|-----------------|-------------------------------------------|---------------------------------------------------------------------|-----------------------------------------|---------------------------------------------|
| Phases          | Binding phase plans defined (1–7)         | 3 / 7 plans committed (seed, glyph-chain, ledger)                  | 7 / 7 phases with executable plans      | `daemons/phase*/phase.plan.yaml`           |
| Glyphs          | Core schemas + golden examples            | 4 schemas + status + 8 examples + receipts JSONL fixture           | Full glyph set driving live receipts    | `glyphs/schemas/`, `glyphs/examples/`      |
| SLOs            | Service-level objectives + test coverage  | Global SLO config + 3 integration suites (glyph, SPV, twin)        | All daemons gated by SLOs in runtime    | `config/slo.toml`, `tests/test_*.rs`       |
| Ledger          | Hot/cold ledger + integrity scripts       | SQLite + RocksDB configs + red-loop + migrate + check-receipts     | Continuous, monitored append-only chain | `config/ledger.*.toml`, `scripts/*.sh,rs`  |
| Orbital/Quantum | ZK + entanglement paths                   | Schemas + examples + tests for ZK anomaly + twin divergence        | Online Groth16 + entanglement in swarm  | `glyphs/schemas/`, `tests/test_digital_twin.rs` |

All KPIs are verifiable directly from the repository; no runtime metrics are implied until the swarm is deployed.

---

## System Overview

Truth Tunnel is a backend-only daemon swarm:

- **Glyph-native:** Every operation is a glyph: intent, receipt, anchor, daemon status.
- **Deterministic:** All state transitions are BLAKE3- and Merkle-anchored, Kyber-1024 signed.
- **Quantum- and ZK-aware:** Schemas and tests support Groth16 ZK anomaly proofs and entanglement-based latency prediction.
- **Multi-tenant:** `tenant_id` is mandatory in all glyphs and configs.

Core design is specified in:

- [docs/SDD.md](docs/SDD.md) — System Design Document (invariants, glyph taxonomy, halt rules)  
- [docs/Charter.md](docs/Charter.md) — Governance contract between humans, daemons, and physics  
- [docs/Eng_Arch.md](docs/Eng_Arch.md) — Canonical architecture and message flow

---

## Repository Layout (Backend Only)

Workspace and daemons:

- `src/crates/groot-swarm/` — single entrypoint binary (`groot-swarm ship`), phase control, final anchoring
- `src/crates/glyph-lib/` — shared glyph types, schema bindings, Merkle + Kyber helpers, ZK/quantum helpers
- `src/crates/rocket-engine/` — Prufrock/TBM simulation and tunnel physics
- `src/crates/spv-api/` — SPV HTTP/gRPC API for glyph submission and Merkle proof retrieval
- `src/crates/digital-twin-groot/` — digital twin + entanglement-based latency prediction
- `src/crates/portal-zero/` — CLI ingress portal for IntentGlyph intake and proof queries
- `src/crates/ledger-explorer/` — SQLite/RocksDB ledger and Merkle proof path queries
- `src/crates/star-lord-orchestrator/` — phase/rollback orchestration and process graph
- `src/crates/mantis-community/` — Twilio/Grok-voice anomaly paging and community signals
- `src/crates/nebula-guard/` — authN/Z, orbital verification, ZK anomaly sharing
- `src/crates/drax-metrics/` — metrics, SLO enforcement, red-loop compaction hooks

Supporting structure:

- `daemons/phase*/phase.plan.yaml` — phase execution contracts (Phase 1–3 committed)
- `glyphs/schemas/*.schema.json` — canonical glyph schemas (anchor, intent, receipt, daemon status)
- `glyphs/examples/*` — golden glyph fixtures (intent, anchor, receipts, status)
- `glyphs/receipts/receipts.jsonl` — append-only ledger file (genesis planned in Phase 1)
- `config/slo.toml` — binding SLOs for latency, anomaly rate, entanglement quality
- `config/nats.toml` — NATS JetStream message bus topology
- `config/ledger.sqlite.toml`, `config/ledger.rocksdb.toml` — hot/cold ledger configuration
- `config/arweave.yaml`, `config/ipfs.yaml` — permanent anchoring endpoints and tags
- `config/tenants/*.yaml` — multi-tenant isolation (default, xai-memphis-01, spacex-orbit-01, acme example)
- `config/agents/*.yaml` — Guardians org chart and swarm role weights
- `config/orchestrator/routing_rules.yaml` — subject → daemon routing map
- `scripts/weekly_red_loop_compaction.rs` — 7‑day compaction + death-criteria hook
- `scripts/deploy-manifest.sh` — manifest-based deploy/rollback
- `scripts/nats-bootstrap.sh` — NATS bootstrap and subject seeding
- `scripts/ledger-migrate.sh` — ledger schema migrations
- `scripts/check-receipts.sh` — receipts chain/Merkle integrity check
- `tests/test_glyph_chain.rs` — glyph chain + Merkle + signature integration test
- `tests/test_spv_roundtrip.rs` — SPV submit/verify/proof roundtrip test
- `tests/test_digital_twin.rs` — digital twin divergence, Merkle, and receipt tests

---

## Glyph Pipeline

Glyphs are the only interface:

1. **IntentGlyph** — authorized intent (human or Guardian)  
2. **ReceiptGlyph** — atomic event receipts (bore, orbital telemetry, ZK anomaly, entanglement prediction, etc.)  
3. **AnchorGlyph** — Merkle-anchored batch of receipts, Kyber-signed  
4. **DaemonStatusGlyph** — daemon health and SLO status

Canonical definitions and documentation:

- [glyphs/README.md](glyphs/README.md) — glyph lifecycle, hashing/signing rules  
- `glyphs/schemas/intent_glyph.schema.json`  
- `glyphs/schemas/receipt_glyph.schema.json`  
- `glyphs/schemas/anchor_glyph.schema.json`  
- `glyphs/schemas/daemon_status_glyph.schema.json`  
- `glyphs/examples/*.json` and `glyphs/examples/receipts.example.jsonl`

The integration tests under `tests/` validate that chains are consistent, Merkle roots match, Kyber signatures are present, and twin divergence stays below configured thresholds.

---

## 48-Hour Ship Phases (Design Status)

Phase plans currently committed:

1. **Phase 1 — Seed** (`daemons/phase1-seed/phase.plan.yaml`)  
   - Genesis IntentGlyph and AnchorGlyph, bootstrap ledger, first DaemonStatusGlyph.

2. **Phase 2 — Glyph Chain** (`daemons/phase2-glyph-chain/phase.plan.yaml`)  
   - Schema validation, Merkle+Kyber chain, first receipts, ZK + entanglement extensions.

3. **Phase 3 — Ledger** (`daemons/phase3-ledger/phase.plan.yaml`)  
   - SQLite/RocksDB initialization, first 1000 receipts, Arweave/IPFS anchoring, proof sampling.

Phases 4–7 (digital twin, orchestrator, ops, harden) are defined in docs and Eng_Arch and are wired into SLO and routing configs but not yet executed in this repo.

---

## Quick Start (Backend Only)

Requirements: Rust toolchain, `nats-server`, SQLite, RocksDB, Bash.

```bash
# clone
git clone https://github.com/northstaraokeystone/truth-tunnel.git
cd truth-tunnel

# env: copy and edit (TENANT_ID, NATS, ledger paths, Twilio, Grok)
cp config/env.example .env

# bootstrap NATS (optional dry run)
chmod +x scripts/nats-bootstrap.sh
./scripts/nats-bootstrap.sh --dry-run

# build everything
cargo build

# run smoke swarm via single entrypoint
chmod +x ship-all.sh
./ship-all.sh

# or directly:
cargo run -p groot-swarm -- ship
