# groot-swarm — Star-Lord

I am the final arbiter.  
I am the one binary that ships or kills everything.

---

## Ownership

groot-swarm owns the parts of the system that can never be ambiguous:

- Final **AnchorGlyph** emission for every phase and tenant
- Phase transitions (1 → 7) via `phase.transition.*` subjects
- The one true shipping **ReceiptGlyph** for each deploy window
- Emergency swarm halt on critical anomaly or SLO breach
- Permanent storage triggers (Arweave/IPFS anchoring)
- The only daemon allowed to emit `glyph.anchor.pending` and confirm `glyph.anchor.*` finalization

If groot-swarm does not see it, it did not happen.

---

## Responsibilities

1. **Intent intake and normalization**

   - Subscribes to `glyph.intent.>` (and tenant-prefixed equivalents).
   - Validates every IntentGlyph against:
     - `glyphs/schemas/intent_glyph.schema.json`
     - `config/tenants/*.yaml`
     - `config/agents/guardians_org.yaml`
   - Rejects any intent that:
     - Lacks Guardian signature authority.
     - Exceeds risk bounds in `constraints` vs `config/slo.toml`.

2. **Phase orchestration**

   - Works with `star-lord-orchestrator` using:
     - `config/orchestrator/routing_rules.yaml`
     - `config/orchestrator/phase_map.yaml`
   - Enforces StepLock:
     - A phase is **not** complete until required ReceiptGlyphs exist and validate.
     - A new phase **cannot** begin until StepLock is satisfied for the previous one.

3. **Final anchoring**

   - Consumes verified ReceiptGlyphs from `glyph.receipt.>` and NATS streams described in `config/nats.toml`.
   - Builds Merkle batches.
   - Emits a single AnchorGlyph per batch to:
     - `glyph.anchor.pending` (internal).
     - `glyph.anchor.final` (ledger and Arweave/IPFS trigger).
   - Only groot-swarm is allowed to promote pending → final.

4. **Emergency halt**

   - Listens to:
     - `anomaly.critical.>`
     - `daemon.status.>`
   - Applies SLO/stop rules from `config/slo.toml`:
     - e2e latency breach
     - anomaly rate breach
     - entanglement quality breach
     - ZK proof-time breach
     - daemon status loss
   - On breach:
     - Emits an `IntentGlyph` of type `emergency_halt` (authorized_by = Star-Lord).
     - Coordinates a swarm-wide `halt` with Yondu and Drax.
     - Forces Mantis to page humans.

5. **Permanent anchoring**

   - Drives anchoring to Arweave/IPFS by invoking:
     - `scripts/check-receipts.sh`
     - deploy manifests in `ops/manifests/spacex_stage0_manifest.yaml` and `spacex_stage1_manifest.yaml`
   - Uses `config/arweave.yaml` and `config/ipfs.yaml` to decide when, what, and how to anchor.
   - Treats Arweave hashes as irreversible truth.

---

## Data Flows

- **Incoming:**
  - `glyph.intent.>`
  - `glyph.receipt.>`
  - `daemon.status.>`
  - `anomaly.critical.>`
  - `swarm.vote.>`

- **Outgoing:**
  - `phase.transition.*`
  - `glyph.anchor.pending`
  - `glyph.anchor.final`
  - `swarm.vote.*`
  - `voice.page.critical` (via mantis-community, indirect)

All flows must match `config/nats.toml`. groot-swarm does not invent subjects.

---

## CLI Contract

Binary name: `groot-swarm`  
Entrypoint: `src/crates/groot-swarm/src/main.rs`  

```bash
# normal operation — run the entire constellation under Star-Lord
groot-swarm ship

# ship a specific phase (for controlled StepLock testing)
# requires a valid IntentGlyph of type trigger_phase_transition with authorized_by = "Yondu" or "Star-Lord"
groot-swarm ship --phase=3

# immediate coordinated shutdown
# emits emergency_halt IntentGlyph + phase_transition to halted
groot-swarm halt --reason="critical_anomaly_or_manual_intervention"

# show current phase, last AnchorGlyph, and SLO state snapshot
groot-swarm status

Recommended dev usage:

# workspace build
cargo build -p groot-swarm

# run from repo root
cargo run -p groot-swarm -- ship


⸻

Environment Contract

groot-swarm reads configuration exclusively from:
	•	Files:
	•	config/slo.toml
	•	config/nats.toml
	•	config/ledger.sqlite.toml
	•	config/ledger.rocksdb.toml
	•	config/arweave.yaml
	•	config/ipfs.yaml
	•	config/tenants/*.yaml
	•	config/agents/guardians_org.yaml
	•	config/agents/swarm_roles.yaml
	•	config/orchestrator/routing_rules.yaml
	•	config/orchestrator/phase_map.yaml
	•	Environment variables:
	•	TENANT_ID
	•	Required. Must match tenant_id in glyphs and config/tenants/*.yaml.
	•	TENANT_CONFIG
	•	Path to tenant YAML (default: ./config/tenants/tenant_default.yaml).
	•	NATS_URL
	•	NATS/JetStream endpoint (e.g., nats://localhost:4222).
	•	NATS_JWT / NATS_SEED
	•	Credentials for the swarm user defined in config/nats.toml.
	•	LEDGER_SQLITE_PATH
	•	Overrides path in config/ledger.sqlite.toml when needed.
	•	Example: /data/ledger/sqlite/truth_tunnel.db.
	•	LEDGER_ROCKSDB_PATH
	•	Overrides path in config/ledger.rocksdb.toml.
	•	ARWEAVE_WALLET
	•	Overrides wallet in config/arweave.yaml when running in different environments.
	•	GROK_API_KEY
	•	Required when groot-swarm uses Grok-4 style agents for dialed swarm critique and dawn builds (via mantis-community / swarm_roles).

If any critical file or environment variable is missing, groot-swarm must fail fast and emit no glyphs.

⸻

One True Shipping Receipt

groot-swarm is responsible for emitting the “shipping receipt” for each deploy and phase:
	•	Encoded as a ReceiptGlyph with:
	•	receipt_type = "phase_transition" or "compaction_complete" as appropriate.
	•	result = "ok" when a phase is genuinely complete.
	•	ref_glyph_id pointing to:
	•	The controlling IntentGlyph.
	•	Or the AnchorGlyph batch that defines the phase boundary.
	•	This receipt must:
	•	Carry a valid blake3_hash, merkle_root, and kyber_signature.
	•	Be included in exactly one AnchorGlyph.
	•	Be replayable through ledger-explorer down to the byte.

If there is no shipping receipt, the phase is not done. If groot-swarm does not sign it, it is not real.

⸻

Failure Semantics
	•	If groot-swarm detects:
	•	Merkle mismatch.
	•	Broken SLOs from config/slo.toml.
	•	Conflicting receipts for the same ref_glyph_id.
	•	Guardian key misconfigurations.
	•	Then it must:
	•	Emit an anomaly_detected ReceiptGlyph.
	•	Emit an emergency_halt IntentGlyph.
	•	Drive a phase.transition.* into a halted state.
	•	Refuse to emit new AnchorGlyphs until a human acknowledges via human.ack.*.

No partial, degraded Star-Lord mode is allowed. Either groot-swarm is in control, or the swarm is stopped.

⸻

Law

If groot-swarm dies, the entire swarm dies.

