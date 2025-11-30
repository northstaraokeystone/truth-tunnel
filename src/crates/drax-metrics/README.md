# drax-metrics — Drax

I see everything.  
I detect every lie.  
I am the one daemon that calls bullshit on the swarm or lets it burn.

---

## Ownership

drax-metrics owns the swarm’s conscience:

- `metrics_exporter.rs` for Prometheus / OpenTelemetry export.
- Anomaly detection across all glyphs, SLO compliance, and health streams.
- VIH impact scoring for swarm decisions (latency inflation ≤ 1.1× budget).
- Red-loop hooks for weekly compaction and failure harvesting.
- Lie detection across Merkle roots, Kyber signatures, ZK proofs, entanglement correlation, and orbital drift.

If Drax doesn’t flag it, everyone else assumes it is true. That is not a right. That is a death sentence.

---

## Responsibilities

### 1. Observability spine

- Collects metrics from:
  - NATS (`daemon.status.>`, `anomaly.critical.>`, `glyph.receipt.>`).
  - Ledger (append throughput, compaction lag).
  - Nebula, Groot, and Rocket via their DaemonStatusGlyphs.
- Exposes:
  - `/metrics` endpoint (Prometheus format).
  - Optional OTLP exporter for modern telemetry stacks.
- Tracks:
  - Per-daemon latency, error rates, and SLO compliance.
  - Global anomaly counts and severities.
  - Entanglement, ZK, and orbital drift KPIs.

### 2. Lie detection

Drax is the second opinion on everything the swarm claims:

- Merkle:
  - Recomputes BLAKE3 + Merkle roots for receipts and anchors.
  - Flags mismatches as **lies**, not bugs.
- Signatures:
  - Cross-checks Kyber-1024 signature validity for high-risk glyphs.
  - Detects keys “used” outside permitted tenants/Guardians.
- ZK:
  - Validates shape and verification status of `zk_anomaly_proof` receipts.
  - Flags any claimed proof that fails local verification.
- Entanglement:
  - Monitors `receipt_type = "entanglement_prediction"`:
    - `correlation_score` vs SLOs (>= 0.707).
    - `predicted_negation_ms` vs SLOs (>= 1.8 ms).
- Drift:
  - Watches orbital `drift_percent` from nebula-guard:
    - Compares against `max_drift_percent` in `config/slo.toml`.
    - Emits anomalies when routes are still used under drift.

Any inconsistency across these dimensions becomes an anomaly. Repeated inconsistencies become a demand to halt.

### 3. SLO enforcement

- Reads `config/slo.toml`:

  - `max_e2e_latency_ms`
  - `max_anomaly_rate_per_hour`
  - `[nebula-guard]` and `[digital-twin-groot]` SLOs.
  - `[rocket-engine]`, `[drax-metrics]` own parameters.

- For each SLO:
  - Computes moving windows (5-min, 1-hr, daily).
  - Emits `anomaly_detected` and `anomaly.critical.*` when thresholds are violated.
  - Notifies:
    - groot-swarm: to halt or downgrade.
    - star-lord-orchestrator: to block phase advancement.

Drax doesn’t negotiate SLOs. It enforces them.

### 4. VIH impact scoring

- VIH (Value–Impact–Hysteresis) scoring engine:

  - For each proposed decision:
    - Computes latency inflation vs baseline (must be ≤ 1.1×).
    - Scores impact on:
      - Anomaly rates.
      - Entanglement quality.
      - Drift exposure.
  - Summarizes as:
    - `vih_score` ∈ [0, 1].
    - `latency_inflation`, `risk_delta`, `entropy_delta`.

- These scores are used by:
  - star-lord-orchestrator to decide if a variant can become default.
  - groot-swarm to decide whether a ship is allowed to land.

### 5. Red-loop / weekly compaction

- Integrates with:
  - `scripts/weekly_red_loop_compaction.rs`.
  - `config/ledger.rocksdb.toml` `[compaction]` settings.
- Drax:
  - Monitors compaction lag (`max_compaction_lag_seconds`).
  - Builds failure harvests:
    - List of glyphs and ranges likely to break replays.
  - Triggers:
    - `ledger-explorer compact` via NATS or CLI bridging.
  - Records:
    - `compaction_complete` ReceiptGlyphs for each successful pass.

If compaction can’t keep up, Drax must push the swarm toward fewer writes or more hardware.

---

## Data Flows

**Inbound**

- NATS:

  - `daemon.status.>`  
    All DaemonStatusGlyphs (including its own).

  - `anomaly.critical.>`  
    Critical anomalies from Nebula, Groot, ledger-explorer, etc.

  - `glyph.receipt.>`  
    Full stream of receipts (for sampling and lie detection).

- Ledger:

  - Direct reads via ledger-explorer to recompute:
    - Merkle roots and proofs.
    - Historical anomaly rates.
    - ZK / entanglement coverage.

**Outbound**

- NATS:

  - `anomaly.critical.drax.*`  
    Lies and SLO violations Drax has detected.

  - `daemon.status.drax-metrics.<tenant_id>`  
    Own health/status.

- HTTP:

  - `/metrics`  
    Prometheus metrics endpoint.

Drax never edits ledger contents. It only measures and accuses.

---

## CLI Contract

Binary name: `drax-metrics`  
Entrypoint: `src/crates/drax-metrics/src/main.rs`

```bash
# Export current metrics snapshot to stdout (for debug / offline scrape)
drax-metrics export --metrics=<prometheus|json> \
                    [--tenant=<tenant_id>]

# Run lie detection on a single glyph (by glyph_id or hash)
drax-metrics detect --glyph=<glyph_id_or_hash> \
                    [--tenant=<tenant_id>] \
                    [--verbose]

# Compute VIH impact score for a decision (manifest/config hash)
drax-metrics score --vih=<decision_hash> \
                   [--tenant=<tenant_id>] \
                   [--json]

# Emit DaemonStatusGlyph + summarized anomaly count
drax-metrics status [--tenant=<tenant_id>] \
                    [--format=json|table]

# Trigger weekly red-loop compaction audit
drax-metrics compact --red-loop \
                     [--tenant=<tenant_id>] \
                     [--dry-run]
