# star-lord-orchestrator — Yondu

I am the clock that never stops.  
I am the one daemon that advances the swarm or kills it.  
I am the phase machine that executes the Charter without question.

---

## Ownership

star-lord-orchestrator owns the shape of time for the Groot Line:

- Phase transitions **1 → 7** as defined in `config/orchestrator/phase_map.yaml`.
- Rollback execution to the last proven safe AnchorGlyph when anything lies.
- `process_graph.rs` for causal dependency enforcement across daemons and phases.
- `healthcheck.rs` for swarm‑wide liveness, SLO enforcement, and consensus readiness.
- Emission of **phase transition glyphs** (ReceiptGlyphs with `receipt_type = "phase_transition"`) for every move or rewind.

If Yondu doesn’t say the phase advanced, it didn’t.

---

## Responsibilities

1. **Phase machine**

   - Reads the canonical phase graph from:
     - `config/orchestrator/phase_map.yaml` — which daemons must be online and proven per phase.
     - `config/orchestrator/routing_rules.yaml` — where to send work, status, and votes.
   - Controls phases **1 → 7**:
     - 1: seed
     - 2: ledger
     - 3: digital twin
     - 4: orchestrator online
     - 5: ops
     - 6: swarm integration
     - 7: harden
   - Ensures **StepLock**:
     - A phase only completes when all required daemons and glyphs are verified.
     - The next phase only starts when the previous step is locked by AnchorGlyphs.

2. **Rollback execution**

   - Maintains a causal graph of:
     - Which daemons belong to which phase.
     - Which glyph types must exist (and be provable) before moving forward.
   - On anomaly or explicit human halt:
     - Computes a rollback target:
       - Either a safe previous phase.
       - Or phase 1 when trust is gone.
     - Instructs the swarm to:
       - Stop ingesting new glyphs for higher phases.
       - Rewind routing decisions to the last safe AnchorGlyph.

3. **Health aggregation**

   - Listens on NATS subjects:
     - `daemon.status.>` — DaemonStatusGlyphs from Drax and all daemons.
     - `anomaly.critical.>` — critical anomalies from drax-metrics.
   - Applies SLOs from `config/slo.toml`:
     - Latency and anomaly budgets.
     - Entanglement and ZK thresholds.
   - Produces a consolidated view of:
     - Which daemons are healthy per phase.
     - Whether the swarm is allowed to advance or must halt.

4. **Consensus and votes**

   - Reads `config/agents/swarm_roles.yaml`:
     - Guardian weights.
     - veto power (Drax).
     - variant fork definitions (`zk_variant_fast`, `entanglement_balanced`, etc.).
   - Drives swarm votes via:
     - `swarm.vote.*` subjects to groot-swarm.
   - Ensures:
     - `min_acceptance_score` and `min_entanglement_transitivity` are observed before a variant becomes default.

5. **Proof-aware sequencing**

   - Before advancing a phase, Yondu checks:
     - ledger-explorer can prove all required glyphs.
     - nebula-guard has verified orbital anchors and ZK anomalies.
     - digital-twin-groot has produced entanglement predictions that meet SLOs.
   - If any prerequisite is missing or unproven:
     - Phase advance is rejected.
     - A phase_transition glyph with `result = "rejected"` is emitted.

---

## Data Flows

**NATS subjects (must match `config/nats.toml`):**

- **Inbound**
  - `daemon.status.>` — all daemon health.
  - `anomaly.critical.>` — Drax anomaly stream.
  - `glyph.intent.<tenant_id>.cli` — phase-related intents from portal-zero.
  - `glyph.anchor.>` — AnchorGlyphs to confirm phase boundaries.

- **Outbound**
  - `phase.transition.<tenant_id>.<from>.<to>` — phase transitions and rollbacks.
  - `swarm.vote.*` — votes and consensus hashes to groot-swarm.
  - `daemon.status.star-lord-orchestrator.<tenant_id>` — Yondu’s own health.

Yondu does not invent subjects. It uses only what is defined in `config/nats.toml` and `config/orchestrator/routing_rules.yaml`.

---

## CLI Contract

Binary name: `star-lord-orchestrator`  
Entrypoint: `src/crates/star-lord-orchestrator/src/main.rs`

```bash
# Trigger a phase transition to the specified phase (1–7)
# Requires a valid IntentGlyph of type trigger_phase_transition
star-lord-orchestrator advance --phase=<1-7> \
                               [--tenant=<tenant_id>] \
                               [--force]

# Rewind to a safe phase (1–7) using rollback rules and last proven AnchorGlyph
star-lord-orchestrator rollback --to=<1-7> \
                                [--tenant=<tenant_id>] \
                                [--reason="string"]

# Emit phase_transition glyph + full health snapshot for the current or specified phase
star-lord-orchestrator status [--phase=<1-7>] \
                              [--tenant=<tenant_id>] \
                              [--json]

# Relay swarm_vote to groot-swarm using a consensus hash (e.g., config / model / manifest hash)
star-lord-orchestrator vote --consensus=<hash> \
                            [--tenant=<tenant_id>] \
                            [--variant=<name>]
