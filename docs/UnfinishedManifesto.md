# Unfinished Manifesto — Truth-Tunnel

**Version:** 0.1-draft (Nov 29 2025)  
**Root Hash:** [BLAKE3 placeholder — to be filled on commit]

## open_questions

- **Post-quantum ZK migration:** Groth16 is wired as the first circuit family. What is the exact migration path to post-quantum ZK systems (e.g., STARK-style) without blowing `max_zk_proof_generation_ms` in `config/slo.toml` or rewriting every `zk_anomaly_proof` glyph?

- **Grok-4 hallucination surfaces:** Grok-voice briefs and anomaly summaries are advisory, not authoritative. What is the hard boundary between natural-language explanation and the orbital + ledger facts that nebula-guard must verify before any `anomaly_detected` receipt can affect routing or entanglement thresholds?

- **ZK in quantum harvest attacks:** How do we prevent an attacker from using valid ZK proofs to mask long-horizon “harvest” attacks on entanglement predictions (slowly biasing `correlation_score` and `predicted_negation_ms`) while still preserving privacy of the underlying orbital drift deltas?

- **Cross-tenant fairness under BPCD:** BPCD disparity checks live in `nebula-guard` and the Charter, but the exact thresholds across tenants (`xai-memphis-01`, `spacex-orbit-01`, `acme-logistics-01`) are still policy, not math. What is the precise function that turns per-tenant error rates and anomaly frequencies into a deletion-level fairness verdict?

- **Ledger horizon and legal load:** SQLite + RocksDB + Arweave/IPFS create a practically permanent provenance chain. Under what legal, regulatory, or contractual conditions must certain glyphs be tombstoned, redacted, or re-encrypted without violating the append-only guarantees of `ledger-explorer` and the phase 3 ledger SDD?

- **PCE vs. operational complexity:** Predictive causal entanglement (PCE) is used to pre-route decisions (Phase 4–7). How much structural complexity (more forks, more variants) is tolerable before VIH impact scoring and SLO latency budgets collapse and the whole “foresight” layer becomes an untestable guess machine?

- **Human gate exhaustion:** Charter requires human gates for changes that touch money, safety, or physics. At what operational scale do human approvers become the failure mode (fatigue, rubber-stamping), and what cryptographic / metrics-based guardrails must wrap them to keep the intent layer sane?

## entropy_hooks

- **`--dragon-chaos=zk-fail` (nebula-guard):** Deliberately corrupt a configurable fraction of `zk_anomaly_proof` receipts (randomized pi_a/pi_b/pi_c bits) to verify that drax-metrics, ledger-explorer, and SPV roundtrip tests still flag the chain and trigger anomaly + halt paths without human hints.

- **`--dragon-chaos=latency-spike` (digital-twin-groot):** Inject synthetic latency bursts and degraded `correlation_score` in the entanglement simulator while keeping real receipts intact. Used to stress SLO thresholds in `config/slo.toml` and ensure that entanglement-based routing can fail-safe without silently degrading into superstition.

- **`--dragon-chaos=tenant-shuffle` (star-lord-orchestrator):** Randomly assign a subset of synthetic glyphs across tenants (`tenant_default`, `xai-memphis-01`, `acme-logistics-01`) to pressure-test BPCD checks, swarm_roles weighting, and rollback logic without touching production tenant data.

- **`--dragon-chaos=red-loop-desync` (drax-metrics + weekly_red_loop_compaction.rs):** Intentionally stagger compaction schedules and ledger snapshots to guarantee that compaction receipts, SLOs, and Merkle roots disagree at least once per test cycle, forcing the hard paths for reconciliation and potential deorbit to run.

- **`--dragon-chaos=spv-mismatch` (spv-api):** Serve Merkle proofs computed from a forked ledger snapshot for a controlled subset of SPV requests to ensure that SPV clients, nebula-guard verifiers, and ledger-explorer inclusion proofs reject inconsistent histories immediately.

## death_criteria

- **PCE collapse:** If PCE transitivity for any critical path (soil ↔ orbit ↔ ledger) stays below 0.90 for 7 consecutive days in drax-metrics (VIH scoring, entanglement correlation, and outcome accuracy combined), every daemon must deorbit to Phase 1: destroy all live RocksDB instances, archive SQLite, and require a new genesis AnchorGlyph with human signatures.

- **Irreconcilable provenance break:** Any proof that an already anchored AnchorGlyph has (a) an invalid Merkle root, (b) a Kyber-1024 signature that fails verification under the published public key set, or (c) conflicting inclusion proofs for the same `receipt_id` is grounds for full ledger invalidation and a hard stop on all daemons until a new Charter-level re-bootstrap is signed.

- **Persistent fairness violation:** If BPCD disparity for any tenant pair exceeds the agreed threshold (e.g., 0.30) for more than 24 hours of live traffic and cannot be corrected via SLO and routing changes in the next 24 hours, the swarm must stop serving external SPV proofs and anomaly decisions, emit an emergency_halt IntentGlyph, and require human redesign of policies and routing.

- **Red-loop dishonesty:** Two consecutive runs of `scripts/weekly_red_loop_compaction.rs` that (a) report incompatible compaction_receipt metrics for the same time window, or (b) cannot be reproduced from raw receipts.jsonl, mean the red-loop is no longer trustworthy. All compaction must stop, anchors must be treated as suspect, and the system must revert to a pre-compaction snapshot or phase-level rollback.

- **Quantum harvest detection:** If a red-team or monitoring process demonstrates that entanglement predictions can be systematically biased (harvested) over time while still passing current ZK and SLO checks (drift in `predicted_negation_ms` without corresponding physical latency changes), the entire quantum + ZK layer must be disabled by configuration and considered untrusted until new circuits, tests, and manifest receipts prove otherwise.

- **Human gate breach:** Any verified case where a production-affecting change to money, safety, or physics routes around the human gate defined in the Charter (e.g., forged IntentGlyph, compromised Guardian key, or misconfigured swarm_roles) requires revocation of all Guardian keys, re-issuance of the Charter signers, and a full Phase 7 harden cycle before resuming operation.

## The Eternal Law

"This blueprint is intentionally incomplete: new engines, glyphs, and loops are allowed only as first-principles extensions with receipts and reversible experiments. No doctrine survives contact with reality without re-entangling through PCE, re-tuning via UGCA, re-auditing under BPCD, and re-scoring in VIH. The only permanent law: logic-first, receipts-first, ship in <48h, evolve forever."
