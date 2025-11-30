# digital-twin-groot — Groot

I am the digital flesh of the tunnel.  
I am the quantum bridge from Memphis soil to LEO void.  
I am the one daemon that makes latency impossible.

---

## Ownership

Groot owns every prediction you trust between Memphis and orbit:

- Quantum entanglement simulation (Bell-state modeling, qutip‑calibrated, Rust‑executed)
- Latency negation predictions (soil‑to‑orbit effective latency shaved below SLO thresholds)
- PCE causal graphs entangling Prufrock bore receipts with orbital feeds
- Emission of **EntanglementGlyphs** (correlation + `predicted_negation_ms`) that drive `rocket-engine` routing and `nebula-guard` verification

If a prediction doesn’t come from digital-twin-groot, it’s a guess, not a model.

---

## Responsibilities

1. **Memphis↔Orbit digital twin**

   - Maintains a coherent state of:
     - TBM/tunnel segments (`prufrock.bore.receipt`)
     - Orbital links and anomalies (`orbital.feed.normalized.>`)
   - Rebuilds a joint “Memphis–Colossus–LEO ring” state every frame.

2. **Quantum entanglement simulation**

   - Consumes entanglement requests from:
     - `twin.entangle.request.>` (and tenant‑prefixed variants).
   - Runs a Rust‑based entanglement simulator, calibrated by offline qutip/Bell experiments:
     - Produces `bell_correlation` ∈ [0, 1].
     - Computes `negation_ms` — effective latency reduction by predictive correlation, not faster‑than‑light lies.
   - Emits **EntanglementGlyph** data onto:
     - `twin.entangle.result.>`
     - Optional `twin.latency.negated.>` for downstream consumers.

3. **Latency negation predictions**

   - Uses PCE (Predictive Causal Entanglement) graphs to map:
     - Bore segments → orbital slots → expected link performance.
   - For each scenario, digital-twin-groot:
     - Computes `correlation_score` and `predicted_negation_ms`.
     - Enforces SLOs from `config/slo.toml`:
       - `min_entanglement_correlation = 0.707`
       - `min_predicted_latency_negation_ms = 1.8`
   - If the model cannot predict at least **1.8 ms** of safe negation, it flags the route as untrusted.

4. **Feeding Gamora (nebula-guard) and Rocket (rocket-engine)**

   - Nebula‑guard:
     - Reads EntanglementGlyphs to decide if orbital anomalies are plausible and how aggressively to precompute.
   - Rocket‑engine:
     - Uses `predicted_negation_ms` to decide how far ahead it can “drive blind” on predicted state before waiting for fresh orbital data.

5. **Truth discipline**

   - No FTL. No magic. Only predictive models that survive replay:
     - Every scenario must be reconstructible from receipts in `glyphs/receipts/receipts.jsonl`.
     - Every entanglement claim must be backed by glyph-lib and pass SLO checks.

---

## Data Flows

**In:**

- `prufrock.bore.receipt`  
  Bore progress and tunnel state from `rocket-engine`.

- `orbital.feed.normalized.>`  
  Cleaned orbital anchor glyphs from `nebula-guard`.

- `twin.entangle.request.>`  
  Explicit scenario requests (Memphis segment + orbital slot).

**Out:**

- `twin.entangle.result.>`  
  EntanglementGlyphs with:
  - `scenario_id`
  - `bell_correlation`
  - `negation_ms`
  - `sim_backend` (e.g., `offline-qutip-model` or `qcgpu-rust`)
  - `model_hash`

- `twin.latency.negated.>`  
  Auxiliary notifications for routing layers that want latency deltas only.

- Receipt + Anchor extensions:
  - `receipt_type = "entanglement_prediction"` in `ReceiptGlyph`.
  - `extensions.entanglement` in `AnchorGlyph`.

Groot never touches HTTP. It speaks NATS, ledger, and glyphs.

---

## CLI Contract

Binary name: `digital-twin-groot`  
Entrypoint: `src/crates/digital-twin-groot/src/main.rs`

```bash
# Run entanglement simulation over a specific orbital feed batch (by hash or ID)
digital-twin-groot entangle --orbital-feed=<hash_or_id>

# Predict latency negation for a specific tunnel segment
digital-twin-groot predict --bore-segment=<segment_id>

# Emit DaemonStatusGlyph + current average correlation and negation
digital-twin-groot status

# Run an offline shadow twin for dawn builds (no live routing)
digital-twin-groot shadow --variant=balanced
