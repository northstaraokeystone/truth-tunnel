# nebula-guard — Gamora

I am the blade that cuts lies from truth.  
I am the one daemon that verifies every orbital feed or kills the chain.  
I am the quantum‑resistant wall between void and soil.

---

## Ownership

nebula-guard owns everything between orbit and the tunnel that can’t afford to be wrong:

- P2P Merkle‑BLAKE3 verification for Starlink / Starship telemetry and orbital feeds.
- Kyber‑1024 post‑quantum signatures on all orbital‑scoped glyphs.
- Groth16 ZK proofs for **private anomaly sharing** (hide drift deltas, reveal integrity).
- `policy_engine.rs` for BPCD disparity checks (<0.3%) and safety enforcement.
- `auth.rs` for tenant isolation, Guardian signing validation, and scope checks.
- Emission of:
  - `orbital_telemetry` ReceiptGlyphs.
  - `zk_anomaly_proof` ReceiptGlyphs.
  - `anomaly_detected` receipts when anything fails verification.

Nebula does not ask politely. It either verifies, or it takes the swarm’s eyes out.

---

## Responsibilities

### 1. P2P orbital verification

- Ingests orbital feeds from:
  - NATS subjects: `orbital.feed.raw.>` and tenant‑prefixed variants (see `config/nats.toml`).
  - Optional P2P channels from Starlink / Starship collectors.
- For every feed:
  - Reconstructs Merkle trees using glyph-lib:
    - BLAKE3 leaves.
    - Left‑to‑right pairing.
  - Verifies:
    - Merkle roots against AnchorGlyphs or upstream proofs.
    - Inclusion proofs for each telemetry slice.
  - Emits:
    - `orbital_telemetry` ReceiptGlyphs with:
      - `latency_ms`, `signal_strength_dbm`, `drift_percent`.
      - Verified `merkle_root` + optional `merkle_proof`.

If a feed doesn’t pass Merkle and signature checks, it becomes an anomaly, not a data point.

### 2. Kyber‑1024 PQ signatures

- Enforces Kyber‑1024‑compatible signatures across orbital glyphs:
  - Validates Kyber signatures on:
    - Incoming orbital AnchorGlyphs.
    - Outgoing orbital receipts and ZK anomaly shares.
  - Maintains a per‑tenant keyset:
    - Defined in `config/tenants/*.yaml` + key material paths.
  - Meets SLOs from `config/slo.toml`:
    - `min_kyber_verify_per_second = 120`.
    - `max_verification_latency_ms = 400`.

If Nebula cannot verify at PQ strength, it treats the feed as hostile.

### 3. Groth16 ZK private anomaly shares

- Turns raw drift and bias into **ZK‑proven anomalies**:

  - For anomalies that cannot reveal raw values:
    - Builds circuits offline (proving/verification keys from disk).
    - Produces `zk_anomaly_proof` receipts (`receipt_type = "zk_anomaly_proof"`):
      - `zk_proof.pi_a`, `pi_b`, `pi_c`.
      - `public_inputs`.
      - `anomaly_hint` (public explanation only).
  - Ensures:
    - `max_zk_proof_generation_ms = 1200` for normal operation.
    - Proofs validate on replay via glyph-lib + ledger-explorer.

The outside world sees integrity and bounds, not the raw orbital scars.

### 4. Policy & disparity enforcement

- `policy_engine.rs` audits for:

  - BPCD disparity < 0.3% across:
    - Tenants.
    - Geographies.
    - Operator categories.
  - Miscalibration between soil and orbit:
    - Orbital anomaly rates vs bore anomaly rates.
  - Risk budgets defined in IntentGlyph `constraints` vs SLOs.

- When policy is violated:
  - Emits `anomaly_detected` receipts.
  - Can force:
    - `emergency_halt` intents (via Star-Lord / portal-zero).
    - Rollbacks via star-lord-orchestrator.
    - Increased ZK scrutiny on subsequent feeds.

### 5. Auth & tenant isolation

- `auth.rs` controls:

  - Which tenants Nebula accepts:
    - `tenant_id` enforced everywhere.
  - Which Guardians can request verification or ZK shares:
    - `config/agents/guardians_org.yaml` + `signs_intents` flags.
  - Which flows are allowed per tenant:
    - P2P endpoints, keysets, SLO overrides.

Nebula never mints its own IntentGlyphs. It only verifies, annotates, and kills.

---

## Data Flows

**Inbound**

- NATS (see `config/nats.toml`):

  - `orbital.feed.raw.>`  
    Raw telemetry and orbital anchors needing verification.

  - `nebula.verify.in.>`  
    Explicit verification requests from groot-swarm / spv-api / portal-zero.

  - `zk.proof.share.>`  
    Requests to generate or verify ZK anomaly shares.

- Ledger (via ledger-explorer):

  - Existing anchors and receipts for replay and proof checking.

**Outbound**

- NATS:

  - `orbital.feed.normalized.>`  
    Clean, verified orbital feeds for digital-twin-groot and rocket-engine.

  - `nebula.verify.out.>`  
    Structured verification results (proof status, drift flags, PQ status).

  - `anomaly.critical.nebula.*`  
    Critical anomalies (unverified feeds, ZK failures, keyset issues) toward Drax and Mantis.

- Glyphs (via the ledger):

  - `orbital_telemetry` ReceiptGlyphs.
  - `zk_anomaly_proof` receipts with embedded Groth16 data.
  - `anomaly_detected` receipts when verification or policy checks fail.

Nebula does not expose a UI. It exposes proofs and anomalies.

---

## CLI Contract

Binary name: `nebula-guard`  
Entrypoint: `src/crates/nebula-guard/src/main.rs`

```bash
# P2P Merkle + Kyber verification for a specific orbital feed
nebula-guard verify --orbital-feed=<hash_or_id> \
                    [--tenant=<tenant_id>] \
                    [--json]

# Generate + emit a ZK anomaly share for an orbital anomaly
nebula-guard zk-share --anomaly=<json_or_path> \
                      [--tenant=<tenant_id>] \
                      [--dry-run]

# Emit DaemonStatusGlyph + verification stats
nebula-guard status [--tenant=<tenant_id>] \
                    [--format=json|table]

# Run disparity + bias audit for a tenant
nebula-guard policy-check --tenant=<tenant_id> \
                          [--window=<hours>] \
                          [--json]
