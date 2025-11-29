# glyph-lib — Swarm Glyph Engine

`glyph-lib` is the core library crate of Truth-Tunnel: The Groot Line Swarm.  
It owns the definitions and rules for every glyph (Intent, Receipt, Anchor, DaemonStatus, ZK, entanglement) and enforces the hashing, Merkle, and post-quantum signatures that all daemons must obey.

No glyph-lib, no swarm.

---

## Purpose

- Provide **one canonical Rust model** for all glyphs used in the system.
- Enforce **BLAKE3 + Merkle** invariants and **Kyber-1024** signatures.
- Wrap **Groth16 ZK** and **entanglement/latency** helpers in a deterministic, testable API.
- Bind schemas in `glyphs/schemas/*.schema.json` to the daemons that emit and consume them.

This crate is pure backend-only: no UI, no windowing, no HTTP. Just glyphs, math, and truth.

---

## Responsibilities

`glyph-lib` is responsible for:

- Defining strongly-typed Rust structs for:
  - `IntentGlyph`
  - `ReceiptGlyph` (all `receipt_type` variants)
  - `AnchorGlyph`
  - `DaemonStatusGlyph`
  - Orbital and challenge extensions:
    - `OrbitalAnchorGlyph`
    - `ZKAnomalyGlyph` shapes
    - `EntanglementGlyph` shapes

- Enforcing:
  - Canonical JSON serialization rules (field ordering, stable encoding).
  - BLAKE3 hashing for glyph IDs and content hashes.
  - Merkle tree construction and inclusion proofs.
  - Kyber-1024-compatible signature verification hooks.

- Providing helpers for:
  - Groth16 ZK proof wiring (nebula-guard’s anomaly circuits).
  - Entanglement/latency simulation data structures for digital-twin-groot.
  - Validation paths that ensure glyphs conform to their schemas before any daemon trusts them.

It does **not** talk to NATS, the filesystem, or the network directly. Higher-level crates own I/O.

---

## Features

| Feature             | Description                                                                                      | Default |
|---------------------|--------------------------------------------------------------------------------------------------|---------|
| `schemas`           | Loads and binds `glyphs/schemas/*.schema.json` to internal Rust types                            | yes     |
| `hashing`           | BLAKE3-based hashing for glyph IDs and content hashes                                            | yes     |
| `merkle`            | Merkle tree construction, roots, and inclusion proofs for receipt batches                        | yes     |
| `pq-kyber`          | Post-quantum Kyber-1024-compatible signature encoding/decoding helpers                           | yes     |
| `zk-groth16`        | Adapters and types for Groth16 ZK anomaly proofs (no proof system implementation here)           | yes     |
| `entanglement`      | Data structures and validation for entanglement correlation and latency negation predictions     | yes     |
| `serde`             | Serialization/deserialization of glyphs via `serde`                                              | yes     |
| `validation`        | Strict structural validation of glyphs before use                                                | yes     |
| `debug`             | Extra assertions on hashes, Merkle roots, and signatures                                         | no      |

The default feature set is tuned for the swarm: maximum integrity with no frontend baggage.

---

## Crate Structure

Logical layout (inside `src/crates/glyph-lib/`):

- `src/lib.rs`  
  - Public entrypoint. Re-exports all glyph types, hashing helpers, and verification APIs.

- `src/anchors/anchor_types.rs`  
  - Core glyph type definitions:
    - AnchorGlyph
    - IntentGlyph
    - ReceiptGlyph
    - DaemonStatusGlyph
    - Orbital extensions

- `src/anchors/merkle.rs`  
  - Merkle tree utilities:
    - Leaf hashing (BLAKE3).
    - Batch root computation.
    - Inclusion proof building and verification.

Additional internal modules (names are descriptive; implementation must match):

- `hashing`  
  - Canonical JSON encoding.
  - ID derivation (`anchor-*`, `intent-*`, `receipt-*`).

- `pq`  
  - Kyber-1024-compatible signature payload types.
  - Quorum representation used in AnchorGlyph `kyber_signature`.

- `zk`  
  - Structures for Groth16 proof coordinates (`pi_a`, `pi_b`, `pi_c`).
  - Public input vectors and hashing (`public_inputs_hash`).

- `entangle`  
  - Fields for entanglement correlation, `negation_ms`, and `bell_state`.
  - Validation against minimum correlation thresholds.

All of this must be deterministic and side-effect free.

---

## Invariants (Enforced by glyph-lib)

Every consumer (nebula-guard, rocket-engine, digital-twin-groot, drax-metrics, groot-swarm, spv-api, ledger-explorer) relies on glyph-lib to enforce these invariants:

1. **Schema alignment**
   - Rust types must round-trip cleanly with:
     - `glyphs/schemas/anchor_glyph.schema.json`
     - `glyphs/schemas/intent_glyph.schema.json`
     - `glyphs/schemas/receipt_glyph.schema.json`
     - `glyphs/schemas/daemon_status_glyph.schema.json`
   - Golden examples in `glyphs/examples/*.json` and `glyphs/examples/receipts.example.jsonl` must decode and validate.

2. **Canonical hashing**
   - `glyph_id` and `receipt_id` patterns:
     - `intent-[a-f0-9]{32}`
     - `anchor-[a-f0-9]{32}`
     - `receipt-[a-f0-9]{32}`
   - `blake3_hash` always derived from canonical JSON excluding signature fields.

3. **Merkle correctness**
   - `merkle_root` must match:
     - BLAKE3 tree over the set of receipts involved in a batch.
   - `merkle_proof` must be verifiable for its `leaf_index` and `siblings`.

4. **Signature consistency**
   - Kyber signatures must be checked against:
     - The correct hash (IntentGlyph → payload, ReceiptGlyph → `blake3_hash`, AnchorGlyph → `blake3_hash` + `merkle_root`).
   - Guardian IDs and key IDs must map to `config/agents/guardians_org.yaml` and `config/agents/swarm_roles.yaml`.

5. **Tenant isolation**
   - Every glyph must carry a non-empty `tenant_id`.
   - glyph-lib must fail validation if `tenant_id` is missing or malformed.

If any of these invariants fail, the consuming daemon must treat the glyph as invalid and follow the halt rules defined in the SDD and Charter.

---

## Who Uses glyph-lib

- **nebula-guard**  
  - `verify_orbital`, ZK anomaly proof handling, Kyber validation.

- **digital-twin-groot**  
  - Entanglement glyph construction, correlation sanity checks, latency negation metadata.

- **rocket-engine**  
  - Bore progress glyphs, tunnel segment anchors.

- **drax-metrics**  
  - DaemonStatusGlyph creation and glyph integrity monitoring.

- **groot-swarm**  
  - Final AnchorGlyph assembly, shipping receipts, and anchoring decisions.

- **spv-api**  
  - SPV proof serving over ledger-backed glyphs.

- **ledger-explorer**  
  - Ledger reconstruction, full replay, and provenance queries.

Every daemon must call into glyph-lib for glyph creation, validation, hashing, and verification. No daemon is allowed to hand-roll glyph formats.

---

## Testing and Golden Fixtures

glyph-lib is the reference implementation for all glyph tests:

- Golden fixtures:
  - `glyphs/examples/anchor_glyph.example.json`
  - `glyphs/examples/intent_glyph.example.json`
  - `glyphs/examples/receipt_glyph.*.example.json`
  - `glyphs/examples/daemon_status_glyph.example.json`
  - `glyphs/examples/receipts.example.jsonl`

- Integration tests (in `tests/`):
  - Must use glyph-lib to load, validate, and hash these examples.
  - Any schema change requires updating glyph-lib types and golden fixtures together.

If glyph-lib cannot validate the examples, the build is considered broken.

---

## Law

glyph-lib defines what a glyph is and how it proves itself.  
If a value does not pass through glyph-lib’s types and validators, it is not a glyph and it does not exist.

No daemon may emit a glyph that fails validation against its schema. Ever.