# Truth-Tunnel Engineering Architecture  
**Version:** 0.1-draft (Nov 29 2025)  
**Root Hash:** [BLAKE3 placeholder — to be filled on commit]  
**Status:** Binding — every downstream file inherits this flow  

---

## 1. The Canonical Path (One Truth)

There is exactly one golden path from human intent in Memphis soil to orbital verification and back to tunnel routing. Every other flow is a projection or optimization of this path.

```mermaid
flowchart LR
    Human[Human / Tenant CLI<br/>or Grok-voice via Mantis] -->|intent_glyph| PortalZero
    PortalZero[portal-zero<br/>CLI / gRPC ingress] -->|IntentGlyph| StarLord
    StarLord[groot-swarm<br/>+ star-lord-orchestrator] -->|IntentGlyph (start_bore)| Rocket
    Rocket[rocket-engine<br/>prufrock-sim] -->|AnchorGlyph (bore)| Gamora
    subgraph Orbital Ring
        OrbFeed[orbital peers<br/>LEO / Starlink data] -->|OrbitalAnchorGlyph| Gamora
    end
    Gamora[nebula-guard<br/>ZK + Kyber] -->|ReceiptGlyph + ZKAnomalyGlyph| Groot
    Gamora -->|anomaly.critical.*| Drax
    Groot[digital-twin-groot<br/>entanglement sim] -->|EntanglementGlyph| Gamora
    Groot -->|Anchor/ReceiptGlyph (twin_state)| Ledger
    Ledger[ledger-explorer<br/>SQLite/RocksDB + receipts.jsonl] -->|proof_path + Merkle root| Nebula
    Nebula[spv-api<br/>CLI/gRPC SPV] -->|JSONL proofs| Human
    Drax[drax-metrics<br/>SLO + halt signals] -->|DaemonStatusGlyph / halt| StarLord

Canonical sequence:
	1.	A human or tenant system expresses a desired action (e.g. start a new tunnel segment, adjust orbital monitoring) via:
	•	CLI → portal-zero
	•	or Grok-voice → mantis-community → portal-zero
→ results in an IntentGlyph published into the swarm.
	2.	groot-swarm + star-lord-orchestrator (Star-Lord/Yondu) ingest the IntentGlyph, consult config/orchestrator/*, and schedule:
	•	rocket-engine (Rocket) to simulate/drive TBMs.
	•	Future orbital checks via nebula-guard.
	3.	rocket-engine / prufrock-sim produces bore/tunnel AnchorGlyphs and ReceiptGlyphs describing TBM movement and tunnel state, publishing them to prufrock.bore.*.
	4.	Orbital peers (LEO / Starlink data feeds) produce OrbitalAnchorGlyphs on orbital.feed.*. These describe link health, drift, and anomaly candidates.
	5.	nebula-guard (Gamora) consumes:
	•	Bore receipts from prufrock.bore.*.
	•	OrbitalAnchorGlyphs from orbital.feed.*.
	•	Twin predictions from twin.entangle.* (below).
It then:
	•	Computes BLAKE3/Merkle anchors.
	•	Runs Groth16 ZK circuits (via glyph-lib) to produce ZKAnomalyGlyphs that prove anomaly classification without revealing private deltas.
	•	Wraps results in Kyber-1024-quorum signatures.
	•	Emits:
	•	Verified ReceiptGlyphs on nebula.verify.*.
	•	Critical anomaly_receipt glyphs on anomaly.critical.* for Drax.
	6.	digital-twin-groot (Groot) consumes:
	•	Bore glyphs (tunnel state).
	•	OrbitalAnchorGlyphs (orbit state).
It maintains a joint Memphis↔orbit digital twin and runs offline-calibrated entanglement simulations (via glyph-lib) to emit EntanglementGlyphs on twin.entangle.* describing:
	•	bell_correlation
	•	negation_ms (effective latency reduction via prediction)
These glyphs feed back into nebula-guard’s decision logic, closing the Memphis–Colossus–Orbit–Tunnel loop.
	7.	All verified glyphs and receipts are persisted by ledger-explorer (Kraglin) to:
	•	glyphs/receipts/receipts.jsonl (append-only)
	•	SQLite (data/ledger/sqlite/) for hot state
	•	RocksDB (data/ledger/rocksdb/) for cold/archive
	8.	spv-api (Nebula) exposes:
	•	gRPC/JSONL SPV endpoints that return Merkle proofs and Kyber-quorum attestations for any referenced glyph, using ledger-explorer as source of truth.
	9.	drax-metrics (Drax) consumes all glyph streams and emits:
	•	DaemonStatusGlyphs for health/SLO.
	•	Halt signals to Star-Lord/Yondu when stop rules are violated (e.g. Merkle mismatch, broken entanglement quality, quorum failure).

Everything else in the repository is scaffolding around this path.

⸻

2. Component Ownership Matrix (Guardians = Crates)

This matrix is the binding map from Guardians → crates → files → glyphs.

Guardian	Crate(s)	Primary Files	Emits (glyphs)	Consumes (glyphs)
Star-Lord	groot-swarm	src/crates/groot-swarm/src/main.rs	IntentGlyph (normalized), ReceiptGlyph(shipping_receipt), DaemonStatusGlyph	All glyph types (for coordination + shipping decisions)
Gamora	nebula-guard	src/crates/nebula-guard/src/main.rs	OrbitalAnchorGlyph (normalized), ZKAnomalyGlyph, ReceiptGlyph(verification), DaemonStatusGlyph	Orbital feeds, bore receipts, EntanglementGlyph, IntentGlyph
Rocket	rocket-engine / prufrock-sim	src/crates/rocket-engine/src/tbm_sim.rs	AnchorGlyph(bore), ReceiptGlyph(bore_receipt), DaemonStatusGlyph	IntentGlyph(start/stop bore), EntanglementGlyph (for planning)
Groot	digital-twin-groot	src/crates/digital-twin-groot/src/world_model.rs	EntanglementGlyph, AnchorGlyph(twin_state), ReceiptGlyph(twin_update)	Bore receipts, OrbitalAnchorGlyph, ZKAnomalyGlyph
Drax	drax-metrics	src/crates/drax-metrics/src/metrics_exporter.rs	DaemonStatusGlyph, ReceiptGlyph(anomaly_receipt), halt_signal	All glyph types (for metrics & integrity checks)
Nebula	spv-api	src/crates/spv-api/src/grpc_server.rs	ProofGlyph (SPV proof bundles), DaemonStatusGlyph	Ledger-backed glyphs, Merkle roots, ReceiptGlyphs
Mantis	mantis-community	src/crates/mantis-community/src/twilio_hooks.rs	IntentGlyph(voice_page_glyph), AnchorGlyph(community_event)	ReceiptGlyph(anomaly_receipt), DaemonStatusGlyph
Yondu	star-lord-orchestrator	src/crates/star-lord-orchestrator/src/process_graph.rs	phase_transition_glyph, IntentGlyph(phase_control)	DaemonStatusGlyph, IntentGlyph, ReceiptGlyph
Kraglin	ledger-explorer	src/crates/ledger-explorer/src/sqlite_store.rs	proof_path (as glyph payload), DaemonStatusGlyph	ReceiptGlyph stream from glyphs/receipts/receipts.jsonl

Supporting crates:
	•	glyph-lib
	•	Primary files: src/crates/glyph-lib/src/lib.rs + submodules.
	•	Emits nothing on its own; provides glyph types, hashing, Merkle, ZK (Groth16) and quantum helpers used by Gamora, Groot, Drax, and Kraglin.
	•	portal-zero
	•	Primary file: src/crates/portal-zero/src/cli_portal.rs.
	•	Acts as controlled ingress from CLI/gRPC into Star-Lord via IntentGlyphs.

Every crate must use glyph types from glyph-lib and subjects from config/nats.toml. No crate may invent new glyph families without extending the canonical schemas in glyphs/schemas/*.schema.json.

⸻

3. Message Bus Topology (NATS)

All inter-daemon communication rides a NATS bus defined in config/nats.toml. Subject hierarchy is fixed; only tenants, filters, and queue groups vary.

3.1 Subject patterns

The following subject roots are binding:
	•	prufrock.bore.*
	•	orbital.feed.*
	•	nebula.verify.*
	•	twin.entangle.*
	•	anomaly.critical.*

These expand into concrete subjects:
	1.	prufrock.bore.* (Rocket-centric)
	•	prufrock.bore.cmd
	•	Producer: groot-swarm / star-lord-orchestrator
	•	Consumer: rocket-engine
	•	Payload: IntentGlyph (start/stop/update bore run).
	•	prufrock.bore.anchor
	•	Producer: rocket-engine
	•	Consumers: nebula-guard, digital-twin-groot, drax-metrics
	•	Payload: AnchorGlyph(bore).
	•	prufrock.bore.receipt
	•	Producer: rocket-engine
	•	Consumers: ledger-explorer, drax-metrics, groot-swarm.
	2.	orbital.feed.* (Orbital P2P feeds)
	•	orbital.feed.raw
	•	Producers: external orbital peers (via spv-api or offline ingestion).
	•	Consumers: nebula-guard, digital-twin-groot.
	•	Payload: raw OrbitalAnchorGlyph candidates.
	•	orbital.feed.normalized
	•	Producer: nebula-guard.
	•	Consumers: digital-twin-groot, drax-metrics.
	•	Payload: normalized OrbitalAnchorGlyphs.
	3.	nebula.verify.* (Verification results, ZK + Kyber)
	•	nebula.verify.in
	•	Producers: rocket-engine, digital-twin-groot, orbital peers via spv-api.
	•	Consumer: nebula-guard.
	•	Payload: verification requests referencing AnchorGlyphs/OrbitalAnchorGlyphs.
	•	nebula.verify.out
	•	Producer: nebula-guard.
	•	Consumers: ledger-explorer, drax-metrics, groot-swarm.
	•	Payload: ReceiptGlyphs (including ZKAnomalyGlyph attachments).
	4.	twin.entangle.* (Entanglement predictions)
	•	twin.entangle.request
	•	Producers: nebula-guard, rocket-engine.
	•	Consumer: digital-twin-groot.
	•	Payload: scenario descriptors for Memphis↔orbit paths.
	•	twin.entangle.result
	•	Producer: digital-twin-groot.
	•	Consumers: nebula-guard, drax-metrics.
	•	Payload: EntanglementGlyphs (correlation + negation_ms).
	5.	anomaly.critical.* (Hard alerts)
	•	anomaly.critical.orbital
	•	Producer: nebula-guard.
	•	Consumers: drax-metrics, mantis-community, groot-swarm.
	•	anomaly.critical.tunnel
	•	Producer: rocket-engine or digital-twin-groot.
	•	Consumers: drax-metrics, mantis-community.

config/nats.toml must declare these subjects, queue groups, and security settings. No daemon may publish to undeclared subjects.

⸻

4. Ledger & Storage Strategy

Truth-Tunnel maintains a single logical ledger with three layers of storage.

4.1 Append-only receipts.jsonl
	•	File: glyphs/receipts/receipts.jsonl
	•	Writer roles:
	•	groot-swarm (final shipping receipts).
	•	nebula-guard (verification receipts + ZKAnomalyGlyph references).
	•	digital-twin-groot (twin state receipts).
	•	ledger-explorer (compaction and audit receipts).
	•	Rules:
	•	One glyph per line, canonical JSON.
	•	Each entry includes:
	•	tenant_id
	•	glyph_type
	•	blake3_hash
	•	merkle_root (over a window)

4.2 SQLite (hot) and RocksDB (cold)
	•	SQLite:
	•	Config: config/ledger.sqlite.toml
	•	Data path: under data/ledger/sqlite/
	•	Role: hot-path query and SPV proof generation for recent receipts.
	•	Owner: ledger-explorer crate.
	•	RocksDB:
	•	Config: config/ledger.rocksdb.toml
	•	Data path: under data/ledger/rocksdb/
	•	Role: long-term, high-volume storage for older windows and full replay.
	•	Owner: ledger-explorer crate.

ledger-explorer must be able to reconstruct full ledger state from receipts.jsonl + SQLite + RocksDB.

4.3 Merkle compaction & archival
	•	Compaction:
	•	Implemented in scripts/weekly_red_loop_compaction.rs.
	•	Periodically computes new Merkle trees over older windows, emits compaction receipts, and updates RocksDB indexes.
	•	Integrity checking:
	•	Implemented in scripts/check-receipts.sh.
	•	Re-hashes receipts, recomputes Merkle roots, and flags mismatches to Drax via anomaly glyphs.
	•	Arweave/IPFS anchoring:
	•	Configuration in:
	•	config/arweave.yaml
	•	config/ipfs.yaml
	•	Anchoring workflows triggered by:
	•	scripts/check-receipts.sh
	•	Ops manifests in ops/manifests/*.yaml
	•	Strategy:
	•	Periodically push Merkle roots + minimal ledger metadata to Arweave/IPFS.
	•	Treat on-chain/off-chain hash anchors as immutable external proof that the ledger state existed.

⸻

5. ZK + Quantum Integration Points

Groth16 ZK proofs and quantum entanglement simulation are embedded into existing crates; no new crates or folders are introduced.

5.1 nebula-guard → Groth16 + Kyber-1024
	•	Crate: src/crates/nebula-guard/
	•	Dependencies: internal modules in src/crates/glyph-lib/ for:
	•	Groth16 proof generation/verification.
	•	Kyber-1024-based signature and quorum encoding.

Flow inside Gamora:
	1.	Receive OrbitalAnchorGlyph + context via:
	•	orbital.feed.normalized
	•	nebula.verify.in
	2.	Construct a ZK circuit (through glyph-lib) that asserts:
	•	The anomaly classification is consistent with the local model.
	•	Sensitive input deltas (drift, noise) remain private.
	3.	Generate a Groth16 proof:
	•	ZKAnomalyGlyph is created with:
	•	proof
	•	public_inputs_hash
	•	merkle_root
	•	kyber_quorum (computed from guardian keys).
	4.	Wrap the result:
	•	Attach ZKAnomalyGlyph to a ReceiptGlyph’s payload.
	•	Sign or encapsulate the proof with Kyber-1024-quorum keys.
	5.	Publish:
	•	ReceiptGlyph + ZKAnomalyGlyph reference on nebula.verify.out.
	•	Critical anomalies on anomaly.critical.orbital.

5.2 digital-twin-groot → Entanglement sim & latency negation
	•	Crate: src/crates/digital-twin-groot/
	•	Dependencies: quantum helper module in glyph-lib.

Flow inside Groot:
	1.	Consume:
	•	Bore receipts from prufrock.bore.receipt.
	•	OrbitalAnchorGlyphs from orbital.feed.normalized.
	2.	Build a scenario state:
	•	Memphis tunnel segment(s), TBM positions.
	•	Orbital slots and link candidates.
	3.	Call quantum sim helper:
	•	Compute a simple entanglement model for the scenario.
	•	Produce:
	•	bell_correlation
	•	negation_ms (how much latency we can effectively neutralize by predicting state before data arrives).
	4.	Emit EntanglementGlyph:
	•	On twin.entangle.result.
	•	Also append a twin-state ReceiptGlyph to the ledger.
	5.	Feedback:
	•	Nebula-guard reads EntanglementGlyph to decide:
	•	Whether certain anomalies are plausible under current physics.
	•	Whether it can safely act on predictions before all data physically arrives.

5.3 ReceiptGlyph extensions

glyphs/schemas/receipt_glyph.schema.json must support fields for both ZK and quantum outputs, without changing the base file location:
	•	Top-level fields:
	•	zk_proof (optional, opaque blob/string)
	•	public_inputs_hash (optional, BLAKE3 hash of public circuit inputs)
	•	kyber_quorum (optional, integer)
	•	Nested object:
	•	entanglement (optional):
	•	bell_correlation (float)
	•	negation_ms (float)
	•	sim_backend (string)
	•	model_hash (string/BLAKE3)

All crates that emit ReceiptGlyphs must preserve unknown fields to avoid destroying ZK/quantum attachments.

⸻

6. Phase Execution Map (daemons/phase*.yaml)

Each daemons/phase*/phase.plan.yaml file describes which crates run, which glyphs they must handle, and what StepLock conditions must be satisfied before moving on.

6.1 Phase mapping table

Phase Name	Plan File	Crates Involved	Primary Goals
Phase 1 – Seed	daemons/phase1-seed/phase.plan.yaml	glyph-lib, ledger-explorer, groot-swarm	Establish basic glyph schemas, receipts.jsonl append-only behavior, and minimal shipping receipts.
Phase 2 – Glyph Chain	daemons/phase2-glyph-chain/phase.plan.yaml	portal-zero, spv-api, nebula-guard, rocket-engine	MVP glyph ingestion from Memphis soil; prufrock.bore., orbital.feed. wired into nebula.verify.*.
Phase 3 – Ledger	daemons/phase3-ledger/phase.plan.yaml	ledger-explorer, drax-metrics	Full provenance engine: Merkle roots, SQLite/RocksDB integration, and metrics/halt hooks.
Phase 4 – Digital Twin	daemons/phase4-digital-twin/phase.plan.yaml	digital-twin-groot, rocket-engine, nebula-guard	Memphis↔orbit twin online; EntanglementGlyphs produced and fed into verify_orbital decisions.
Phase 5 – Orchestrator	daemons/phase5-orchestrator/phase.plan.yaml	groot-swarm, star-lord-orchestrator, mantis-community	Full phase sequencing, StepLock enforcement, and Grok-voice/CLI integration for tenants.
Phase 6 – Ops	daemons/phase6-ops/phase.plan.yaml	drax-metrics, ledger-explorer, scripts in scripts/, ops/*	Deployment manifests, weekly red loop, Arweave/IPFS anchoring, and ops automation hardened.
Phase 7 – Harden	daemons/phase7-harden/phase.plan.yaml	All of the above	ZK + Kyber + entanglement fully gating orbital/tunnel decisions; death criteria and rollback tested.

6.2 StepLock conditions per phase
	•	Phase 1 StepLock:
	•	Basic AnchorGlyph/ReceiptGlyph roundtrip works.
	•	tests/test_glyph_chain.rs passes.
	•	Phase 2 StepLock:
	•	NATS subjects in Section 3 are live and verified.
	•	tests/test_spv_roundtrip.rs and basic nebula-guard verification tests pass.
	•	Phase 3 StepLock:
	•	Full replay from receipts.jsonl matches SQLite/RocksDB state.
	•	tests/test_prufrock_sim.rs and ledger integrity tests pass.
	•	Phase 4 StepLock:
	•	EntanglementGlyphs are produced and consumed by nebula-guard.
	•	tests/test_digital_twin.rs and tests/test_entanglement.rs pass.
	•	Phase 5 StepLock:
	•	groot-swarm ship spins up all required daemons and respects config/orchestrator/phase_map.yaml.
	•	Mantis can surface anomalies without bypassing human gates.
	•	Phase 6 StepLock:
	•	scripts/check-receipts.sh and scripts/weekly_red_loop_compaction.rs run successfully against real data.
	•	Ops manifests deploy and roll back safely.
	•	Phase 7 StepLock:
	•	ZKAnomalyGlyphs and Kyber-1024 quorums are enforced for orbital flows.
	•	Death criteria and stop rules from SDD/Charter are exercised and produce correct halts.

No downstream file may introduce flows or dependencies that contradict this architecture. If reality forces a change, the Eng_Arch, SDD, and Charter must be updated together and re-hashed before any further ship.

We are Groot.

