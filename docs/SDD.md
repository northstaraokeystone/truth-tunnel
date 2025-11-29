# Truth-Tunnel: The Groot Line Swarm — System Design Document  
**Version:** 0.1-draft (Nov 29 2025)  
**Root Hash:** [BLAKE3 placeholder — to be filled on first commit]  

---

## 1. System Intent & Invariants

### 1.1 Mission (one sentence)

Truth-Tunnel: The Groot Line Swarm is a pure backend-only, glyph-native daemon constellation that deterministically records, verifies, and predicts the joint state of the Memphis Groot Line (tunnels, TBMs, vehicles) and its orbital counterparts (LEO assets, links, anomalies) using cryptographically anchored glyphs and physics-constrained models, with no central authority and no UI.

### 1.2 Non‑negotiable invariants

These are hard constraints for every crate, daemon, script, test, and ops manifest in this repository:

1. **Deterministic replay**

   - Given the same ordered glyph input stream and the same configuration from `config/`, every daemon in `src/crates/` must produce the same glyph outputs, byte-for-byte.
   - No daemon may depend on uncontrolled external entropy; all randomness must be derived from glyphs and config (entropy hooks are explicit).

2. **Receipt provenance**

   - Every meaningful action must result in a **ReceiptGlyph** written append-only to `glyphs/receipts/receipts.jsonl`.
   - No behavior is considered “real” until a corresponding receipt exists and is anchored by:
     - BLAKE3 content hash of the canonical glyph JSON.
     - Merkle root over a receipt window.
     - Kyber-1024-based signature bundle attesting to quorum agreement.

3. **No central auth**

   - There is no single “root god process”. Identity and authorization are expressed as:
     - Tenant IDs and roles in `config/tenants/*.yaml`.
     - Agent roles in `config/agents/guardians_org.yaml` and `config/agents/swarm_roles.yaml`.
     - Kyber-1024 public keys and quorum parameters.
   - `src/crates/nebula-guard/` enforces policies via rules derived from config and cryptographic proofs, not by central login state.

4. **Quantum-resistance**

   - All new authentication, key exchange, and quorum signatures involving orbital glyphs must assume a post-quantum adversary:
     - Kyber-1024 (or equivalent) KEM/signature parameters are configured via `config/agents/*.yaml`.
     - Classical primitives (BLAKE3, Merkle) remain for hashing/commitments, but keying and signatures for orbital receipts must be quantum-resistant.
   - Groth16 ZK proofs are used for **anomaly privacy**, not for long-term PQ security; Kyber provides the PQ shell.

5. **Backend-only**

   - No front-end, web, or graphical client assumptions.
   - The only interfaces are:
     - CLI commands (e.g. `./ship-all.sh`, `cargo run -p groot-swarm -- ship`).
     - gRPC/JSONL APIs described in `src/crates/spv-api/`.
     - File-based glyph and config paths in this repo.

6. **Physics honesty**

   - Quantum entanglement simulations in `src/crates/digital-twin-groot/` and `glyph-lib` **cannot** violate causality or no-communication; they only provide predictive correlation scores and effective latency estimates.
   - “Latency negation” is defined as **correct precomputation** of expected states under physical latency, not faster-than-light signaling.

### 1.3 Tenant isolation rules

- All glyphs and receipts must carry a `tenant_id` field.
- Tenants are defined and constrained by:
  - `config/tenants/tenant_default.yaml`
  - `config/tenants/tenant_example_acme.yaml`
- Daemons must:
  - Never leak glyph content between tenants.
  - Treat each tenant’s ledger state as logically separate, even if stored in the same SQLite/RocksDB instance.
- Test tenants are declared in `tests/*.rs` and must not overlap with production tenants.

---

## 2. Canonical Glyph Taxonomy

All behavior is expressed as glyphs. The schemas live under `glyphs/schemas/*.schema.json` and are mirrored in `src/crates/glyph-lib/`.

### 2.1 Base glyph types

1. **AnchorGlyph**

   - Minimal, canonical description of “what just happened”.
   - Fields (conceptual):
     - `tenant_id`
     - `anchor_id`
     - `kind` (e.g. `tunnel_segment_started`, `orbital_pass_observed`)
     - `payload` (structured, schema-specific)
     - `ts` (monotonic timestamp)
   - Anchors feed all Merkle trees.

2. **IntentGlyph**

   - Describes a planned action or configuration shift before it happens.
   - Fields:
     - `tenant_id`
     - `intent_id`
     - `kind` (e.g. `start_tbms`, `activate_orbital_peer`)
     - `payload`
     - `ts`
   - Used heavily by `src/crates/star-lord-orchestrator/` and `src/crates/groot-swarm/`.

3. **ReceiptGlyph**

   - Evidence that an AnchorGlyph or IntentGlyph was accepted, checked, and committed.
   - Fields:
     - `tenant_id`
     - `receipt_id`
     - `ref_glyph_id` (anchor or intent ID)
     - `result` (`ok`, `anomaly`, `fraud`, `rejected`)
     - `merkle_root`
     - `blake3_hash`
     - `kyber_quorum` (effective quorum level)
   - All receipts append to `glyphs/receipts/receipts.jsonl`.

4. **DaemonStatusGlyph**

   - Heartbeat-style glyphs indicating daemon health.
   - Fields:
     - `daemon_id` (e.g. `rocket-engine`, `nebula-guard`)
     - `tenant_id` (or `system`)
     - `phase` (phase1–phase7)
     - `status` (`ready`, `degraded`, `halted`)
     - `metrics` (latency, error counts, entanglement quality, etc.)
   - Emitted primarily by `src/crates/drax-metrics/`.

### 2.2 Orbital extensions

New glyph types must still live in the same `glyphs/` tree and must respect the invariants above.

1. **OrbitalAnchorGlyph**

   - Specialization of AnchorGlyph representing orbital observations/commands.
   - Fields:
     - Base AnchorGlyph fields
     - `orbit_slot`, `sat_id`, `link_id`
     - `measurement` (drift, SNR, RTT, etc.)
   - Primary producers/consumers:
     - P2P orbital verification in `src/crates/nebula-guard/`.
     - Orbital-twin modeling in `src/crates/digital-twin-groot/`.
     - Edge ingestion via `src/crates/spv-api/`.

2. **ZKAnomalyGlyph**

   - Represents a Groth16-based proof that an anomaly classification is consistent with local models, without revealing raw deltas.
   - Fields:
     - `tenant_id`
     - `anomaly_id`
     - `ref_orbital_anchor_id`
     - `proof` (opaque Groth16 blob)
     - `public_inputs_hash` (BLAKE3)
     - `merkle_root`
     - `kyber_quorum`
     - `verification_hint` (small public metadata to route checks)
   - Produced by:
     - `src/crates/nebula-guard/` (verification engine).
   - Schema: `glyphs/schemas/receipt_glyph.schema.json` extended or specialized.

3. **EntanglementGlyph**

   - Result of offline quantum entanglement simulation for a given orbital/ground scenario.
   - Fields:
     - `tenant_id`
     - `scenario_id`
     - `bell_correlation` (0.0–1.0)
     - `negation_ms` (effective latency reduction via precomputation)
     - `sim_backend` (e.g. `qcgpu-rust`, `offline-qutip-model`)
     - `model_hash` (BLAKE3)
   - Produced by:
     - `src/crates/digital-twin-groot/` in concert with `src/crates/glyph-lib/`.
   - Consumed by:
     - `src/crates/nebula-guard/` as part of `verify_orbital` decision rules.
     - `src/crates/drax-metrics/` for entanglement quality metrics.

### 2.3 Hashing, Merkle, and Kyber-1024 strategy

1. **Hashing**

   - All glyphs must be canonicalized to JSON (stable key ordering) before hashing.
   - Hash function: BLAKE3 over canonical JSON bytes.

2. **Merkle trees**

   - Receipts are batched into windows (configurable via `config/slo.toml`).
   - Each window forms a Merkle tree with BLAKE3 as leaf/inner hash.
   - `merkle_root` is stored in every **ReceiptGlyph**, including ZKAnomalyGlyph and OrbitalAnchorGlyph receipts.

3. **Kyber-1024 signatures/quorum**

   - Each validation peer has a Kyber-1024 keypair or equivalent, configured in:
     - `config/agents/guardians_org.yaml`
     - `config/agents/swarm_roles.yaml`
   - Receipts include:
     - A quorum-weighted signature bundle or reference.
     - An effective `kyber_quorum` field.
   - Nebula-guard enforces:
     - A minimum quorum per tenant or per glyph type, from `config/slo.toml`.

---

## 3. Core Engines (must exist in every phase)

Three conceptual engines exist across the swarm, implemented across multiple crates.

### 3.1 Provenance Engine

- **Purpose:** Make every mutation and observation reconstructible and provable.
- **Live in:**
  - `src/crates/glyph-lib/` (core glyph types + hashing + Merkle)
  - `src/crates/ledger-explorer/` (query over receipts)
  - `glyphs/` (schemas and receipts file)
- **Responsibilities:**
  - Canonicalize glyphs.
  - Hash and Merkle-commit them.
  - Append receipts to `glyphs/receipts/receipts.jsonl`.
  - Serve deterministic views of state to other daemons.

### 3.2 Verification Engine (nebula-guard)

- **Crate:** `src/crates/nebula-guard/`
- **Purpose:** Decide which glyphs are admissible, suspicious, or fraudulent.
- **Responsibilities:**
  - Parse incoming glyphs (especially OrbitalAnchorGlyph).
  - Verify Merkle and hash invariants via glyph-lib.
  - Run **Groth16 ZK proofs** against anomaly circuits (via glyph-lib).
  - Check Kyber-1024 quorum for receipts.
  - Pull EntanglementGlyph data from digital-twin-groot to decide on latency-aware strategies.
  - Emit:
    - `ReceiptGlyph` with `result = ok/anomaly/fraud/rejected`.
    - ZKAnomalyGlyph for private anomaly confirmation.

### 3.3 Fusion & Entanglement Engine (digital-twin-groot + quantum sim)

- **Crate:** `src/crates/digital-twin-groot/`
- **Purpose:** Maintain a digital twin of the Memphis tunnel and orbital environment that predicts states ahead of time.
- **Responsibilities:**
  - Subscribe to:
    - TBM and tunnel glyphs from `src/crates/rocket-engine/`.
    - OrbitalAnchorGlyphs from `src/crates/spv-api/`.
  - Maintain a coarse-grained state of:
    - Tunnel segments and TBM positions.
    - LEO assets and link topologies.
  - Call quantum sim helpers in `src/crates/glyph-lib/` to:
    - Create EntanglementGlyphs for given scenarios.
  - Feed:
    - Nebula-guard’s `verify_orbital` with correlation and effective latency predictions.
    - Drax-metrics with entanglement health.

---

## 4. The 48-Hour Ship Flow (T0–T48h)

The SDD is the root of the 48h ship pipeline. All implementation phases must respect this order.

### 4.1 T0–2h: This SDD + design_glyph

- Write and lock:
  - `docs/SDD.md` (this file).
- Derive a **design_glyph**:
  - Instantiated as examples in:
    - `glyphs/examples/*.json`
  - Describes root invariants and the initial Groot Line + orbital context.
- No code is written until SDD + design_glyph stabilize.

### 4.2 T2–24h: MVP glyph chain + nebula-guard with ZK + quantum sim

- Implement minimum skeletons and flows:
  - Glyph schemas in `glyphs/schemas/*.schema.json`.
  - Baseline configs in `config/*.toml` and `config/*.yaml`.
  - MVP crates:
    - `src/crates/glyph-lib/`
    - `src/crates/nebula-guard/`
    - `src/crates/digital-twin-groot/`
    - `src/crates/spv-api/`
  - Wire:
    - OrbitalAnchorGlyph ingestion via spv-api.
    - Provenance Engine (hash + Merkle + receipts).
    - ZKAnomalyGlyph generation/verification.
    - EntanglementGlyph generation using minimal offline models.
- Tests:
  - `tests/test_glyph_chain.rs`
  - `tests/test_spv_roundtrip.rs`
  - `tests/test_zk_anomaly.rs`
  - `tests/test_entanglement.rs`

### 4.3 T24–48h: Hardening, swarm consensus, deploy manifest

- Add/strengthen:
  - `src/crates/groot-swarm/` orchestration logic.
  - `src/crates/star-lord-orchestrator/` routing decisions.
  - `src/crates/drax-metrics/` comprehensive SLO tracking.
  - Deployment artifacts in:
    - `ops/manifests/spacex_stage0_manifest.yaml`
    - `ops/manifests/spacex_stage1_manifest.yaml`
    - `ops/k8s/groot-swarm-daemons.yaml`
    - `ops/systemd/groot-swarm.service`
- Tighten:
  - Kyber-1024 quorum parameters.
  - Stop rules and rollback logic in nebula-guard.
- Ensure:
  - All scripts in `scripts/` operate only on glyphs and receipts, never on hidden state.

---

## 5. SLOs & Stop Rules (binding)

### 5.1 SLOs (to be encoded in `config/slo.toml`)

1. **Latency SLOs**

   - SPV ingest → ReceiptGlyph emission:
     - Target: ≤ 500 ms for non-orbital; ≤ 1500 ms for orbital + ZK.
   - Entanglement sim:
     - Target: ≤ 200 ms for generating EntanglementGlyphs per scenario.

2. **Integrity SLOs**

   - 0 tolerated Merkle/hashing mismatches per 10^6 glyphs.
   - 0 tolerated Kyber-1024 verification failures without emitting a fraud ReceiptGlyph.

3. **Entanglement quality SLOs**

   - Average `bell_correlation` for valid scenarios ≥ 0.75.
   - If correlation < 0.707 for the same scenario twice in a row, treat as degraded.

4. **Forgetting / retention SLOs**

   - Glyph retention in ledger: defined by tenant policy.
   - Provenance Engine must support replay from genesis for at least one tenant, always.

5. **Disparity SLO**

   - Difference between twin’s predicted state and ledger-anchored state must be:
     - Within configured bounds in `config/slo.toml`.
     - Any violation must emit an anomaly ReceiptGlyph.

### 5.2 Stop rules (hard halts)

Any of these conditions must cause:

- Emission of a critical DaemonStatusGlyph with `status = halted`.
- Controlled shutdown of affected daemons.

Stop conditions:

1. **Merkle mismatch**

   - If any daemon detects a Merkle root mismatch between:
     - Local recomputation and stored receipt.
   - Immediate halt for:
     - `src/crates/ledger-explorer/`
     - `src/crates/nebula-guard/`
   - Requires manual intervention.

2. **Cryptographic failure**

   - Systemic Kyber-1024 verification failures for more than N consecutive receipts (N set per tenant).
   - Groth16 proof verification repeatedly fails for previously valid circuits.

3. **Entanglement sanity failure**

   - `bell_correlation` < 0.707 with `result = ok` from nebula-guard for the same scenario > 1 time.
   - This indicates the fusion model is lying or misconfigured.

4. **SLO violations**

   - Sustained latency or integrity SLO violations beyond thresholds specified in `config/slo.toml` must move daemons into `degraded` or `halted` states.

---

## 6. Rollback & StepLock Strategy

### 6.1 StepLock

- Each phase (phase1–phase7) defined in:
  - `daemons/phase*/phase.plan.yaml`
- StepLock means:
  - A phase is considered committed only when:
    - All expected ReceiptGlyphs are present and valid.
    - DaemonStatusGlyphs for all required daemons show `status = ready` or `status = completed`.
  - No forward progress to the next phase without StepLock.

### 6.2 Rollback

- Rollback is always defined relative to the **last good AnchorGlyph** window.
- Procedure:
  1. Identify last Merkle root that:
     - Has a valid Kyber-1024 quorum.
     - Passes ledger integrity checks.
  2. Reset daemons’ internal state to:
     - The view implied by that receipt window.
  3. Re-ingest glyphs from that point forward.
- Implementation hooks must exist in:
  - `src/crates/groot-swarm/`
  - `src/crates/star-lord-orchestrator/`
  - `scripts/ledger-migrate.sh`
  - `scripts/check-receipts.sh`

---

## 7. Failure Modes & Death Criteria

Death criteria are conditions under which the swarm must be “deorbited” (taken offline and frozen) until deliberate human intervention.

1. **Systemic ledger corruption**

   - Multiple Merkle mismatches across tenants and daemons.
   - Inability to reconstruct ledger from `glyphs/receipts/receipts.jsonl`.

2. **Cryptographic compromise**

   - Proven compromise or catastrophic bug in Kyber-1024 handling.
   - Proof that Groth16 circuits or implementation leak anomaly data beyond stated privacy guarantees.

3. **Twin collapse**

   - `digital-twin-groot`’s predictions diverge from anchored state beyond configured bounds for more than M intervals (per `config/slo.toml`).
   - EntanglementGlyph metrics show sustained nonsense (e.g. high negation_ms with low correlation).

4. **Consensus breakdown**

   - Guardian agents in `config/agents/guardians_org.yaml` and `config/agents/swarm_roles.yaml` cannot reach quorum for critical decisions across a large window of ReceiptGlyphs.

5. **Unbounded replay failure**

   - `ledger-explorer` cannot perform a full replay from genesis for any tenant without errors.

Upon hitting death criteria:

- A `death_criteria` glyph must be written (schema defined in `docs/death_criteria.md` and `glyphs/schemas/receipt_glyph.schema.json` extension).
- `ops/manifests/*` must include an operational pattern for safe shutdown.

---

## 8. References to Downstream Files

All subsequent files (2–55) derive from this SDD.

### 8.1 Core docs

- `docs/Charter.md` — expands Section 1 into narrative and roles.
- `docs/Eng_Arch.md` — diagrams and enumerates flows defined in Sections 2–3.
- `docs/UnfinishedManifesto.md` — elaborates open questions and death criteria.
- `docs/open_questions.md` — formal list of unresolved design issues.
- `docs/entropy_hooks.md` — defines allowed entropy sources and flows.
- `docs/death_criteria.md` — detailed definitions from Section 7.

### 8.2 Glyph schemas and examples

- `glyphs/schemas/anchor_glyph.schema.json` — encodes AnchorGlyph.
- `glyphs/schemas/intent_glyph.schema.json` — encodes IntentGlyph.
- `glyphs/schemas/receipt_glyph.schema.json` — encodes ReceiptGlyph and ZKAnomalyGlyph variants.
- `glyphs/schemas/daemon_status_glyph.schema.json` — encodes DaemonStatusGlyph.
- `glyphs/examples/intent_glyph.example.json` — example IntentGlyph.
- `glyphs/examples/anchor_glyph.example.json` — example AnchorGlyph.
- `glyphs/examples/receipt_glyph.example.jsonl` — example receipt stream.
- `glyphs/README.md` — human explanation of glyph taxonomy.

### 8.3 Config

- `config/slo.toml` — SLOs and thresholds from Section 5.
- `config/nats.toml` — subject names aligning with engines in Section 3.
- `config/grpc.toml` — gRPC endpoints matching flows in Section 4.
- `config/ledger.sqlite.toml` — storage mapping for provenance.
- `config/ledger.rocksdb.toml` — alternative ledger backend config.
- `config/arweave.yaml` — long-term archival policy for receipts.
- `config/ipfs.yaml` — content-addressable storage options.
- `config/tenants/tenant_default.yaml` — baseline tenant.
- `config/tenants/tenant_example_acme.yaml` — example tenant.
- `config/agents/guardians_org.yaml` — Guardian roles, Kyber parameters.
- `config/agents/swarm_roles.yaml` — swarm consensus and Librarian roles.
- `config/orchestrator/routing_rules.yaml` — per-phase routing derived from Section 4.
- `config/orchestrator/phase_map.yaml` — mapping phases to daemons.

### 8.4 Crate-local READMEs

- `src/crates/groot-swarm/README.md` — CLI entry and orchestration.
- `src/crates/glyph-lib/README.md` — glyph types, hashing, Merkle, ZK, quantum hooks.
- `src/crates/rocket-engine/README.md` — TBM/tunnel physics glyphs.
- `src/crates/spv-api/README.md` — gRPC/CLI SPV flows.
- `src/crates/digital-twin-groot/README.md` — twin and entanglement model.
- `src/crates/portal-zero/README.md` — ingress portal semantics.
- `src/crates/ledger-explorer/README.md` — ledger access patterns.
- `src/crates/star-lord-orchestrator/README.md` — routing/phase control.
- `src/crates/mantis-community/README.md` — Twilio/Grok-voice integration.
- `src/crates/nebula-guard/README.md` — verification, ZK, and `verify_orbital`.
- `src/crates/drax-metrics/README.md` — SLO and entanglement metrics.

### 8.5 Phase plans

- `daemons/phase1-seed/phase.plan.yaml` — implements T0–T2h goals.
- `daemons/phase2-glyph-chain/phase.plan.yaml` — implements core glyph chain.
- `daemons/phase3-ledger/phase.plan.yaml` — implements provenance engine.
- `daemons/phase4-digital-twin/phase.plan.yaml` — implements digital-twin-groot.
- `daemons/phase5-orchestrator/phase.plan.yaml` — implements swarm orchestration.
- `daemons/phase6-ops/phase.plan.yaml` — implements ops integration.
- `daemons/phase7-harden/phase.plan.yaml` — implements final hardening.

### 8.6 Scripts & ops

- `scripts/weekly_red_loop_compaction.rs` — implements red-loop behavior respecting Section 5.
- `scripts/deploy-manifest.sh` — applies ops manifests defined from Section 4.
- `scripts/nats-bootstrap.sh` — sets up NATS subjects aligned with config.
- `scripts/ledger-migrate.sh` — supports rollback strategy (Section 6).
- `scripts/check-receipts.sh` — validates Merkle and hash invariants.
- `ops/manifests/spacex_stage0_manifest.yaml` — local deployment derived from SDD.
- `ops/manifests/spacex_stage1_manifest.yaml` — remote/cluster deployment.
- `ops/k8s/groot-swarm-daemons.yaml` — k8s orchestration for all daemons.
- `ops/systemd/groot-swarm.service` — systemd unit for production swarm.

### 8.7 Tests

- `tests/test_glyph_chain.rs` — validates glyph taxonomy and chain behaviors.
- `tests/test_spv_roundtrip.rs` — validates SPV API ingest/receipt roundtrip.
- `tests/test_prufrock_sim.rs` — validates rocket-engine TBM flows.
- `tests/test_digital_twin.rs` — validates twin consistency.
- `tests/test_zk_anomaly.rs` — validates ZKAnomalyGlyph behavior.
- `tests/test_entanglement.rs` — validates EntanglementGlyph behavior.

### 8.8 README

- `README.md` — is the final outward-facing summary of this SDD, architecture, and 48h flow, and must not contradict anything specified here.

This SDD is the law. All downstream files must either implement or explicitly document any deviations as bugs.

We are Groot.