# ledger-explorer — Kraglin

I am the unbreakable chain.  
I am the append-only truth that outlives the swarm.  
I am the one daemon that turns glyphs into eternal proof.

---

## Ownership

ledger-explorer owns everything that decides whether a glyph is real:

- SQLite hot-path + RocksDB cold archive (append-only, no deletes).
- `glyphs/receipts/receipts.jsonl` as the canonical append log.
- Merkle proof path generation for every AnchorGlyph and ReceiptGlyph.
- `query.rs` for inclusion proofs from orbital feeds to tunnel receipts.
- Provenance queries that surface ZK anomaly proofs and entanglement metadata.
- The ledger view that `spv-api`, groot-swarm, and every Guardian must trust.

If ledger-explorer doesn’t see it, nobody else is allowed to swear it happened.

---

## Responsibilities

1. **Append-only ledger**

   - Owns on-disk state for the entire swarm:
     - **Hot path**: SQLite at the path defined in `config/ledger.sqlite.toml`.
     - **Cold archive**: RocksDB at `config/ledger.rocksdb.toml`.
     - **Global log**: `glyphs/receipts/receipts.jsonl` (all tenants).
   - Enforces:
     - No deletes.
     - No in-place updates.
     - Every new record is an append, or it is rejected.

2. **Merkle and SPV proofs**

   - Computes Merkle trees over receipt batches, using glyph-lib’s BLAKE3 + Merkle rules.
   - Ensures:
     - `merkle_root` on ReceiptGlyphs and AnchorGlyphs matches recomputed roots.
     - `merkle_proof` objects verify against stored trees.
   - Serves proof paths to:
     - `spv-api` for HTTP SPV endpoints.
     - groot-swarm for shipping receipts.
     - portal-zero for CLI proof queries.

3. **ZK + entanglement provenance**

   - Indexes and exposes:
     - ZK anomaly receipts (`receipt_type = "zk_anomaly_proof"`).
       - `zk_proof.pi_a`, `pi_b`, `pi_c`.
       - `public_inputs`.
       - `anomaly_hint`.
     - Entanglement predictions (`receipt_type = "entanglement_prediction"`).
       - `correlation_score`.
       - `predicted_negation_ms`.
       - `bell_state`.
   - Allows queries that link:
     - Orbital feeds → ZK anomaly proofs → Bore receipts → EntanglementGlyphs → AnchorGlyphs.

4. **Compaction and cold storage**

   - Drives compaction of historical data:
     - Coalesces older windows into RocksDB column families.
     - Ensures no loss of proof ability:
       - Merkle roots and anchors remain reconstructable.
   - Integrates with:
     - `scripts/weekly_red_loop_compaction.rs`.
     - `scripts/check-receipts.sh`.
   - SLO-aware:
     - Respects `max_compaction_lag_seconds` from `config/slo.toml`.

5. **Health and observability**

   - Emits DaemonStatusGlyphs about:
     - Append throughput.
     - Ledger size per tenant.
     - Compaction lag.
   - Reports to Drax (drax-metrics) so the swarm knows exactly how close to the storage wall it is.

---

## Data Layout

ledger-explorer treats storage as a three-layer pyramid:

1. **JSONL log**  
   - `glyphs/receipts/receipts.jsonl`
   - Every ReceiptGlyph is appended here first.
   - Acts as the canonical replay source.

2. **SQLite hot path**  
   - Path from `config/ledger.sqlite.toml` (default: `/data/ledger/sqlite/truth_tunnel.db`).
   - Indexed for:
     - Fast lookup by glyph ID and `tenant_id`.
     - Common queries (recent tunnel segments, recent anomalies, last entanglement predictions).
   - WAL mode, `synchronous = NORMAL`, tuned for ingestion.

3. **RocksDB cold archive**  
   - Path from `config/ledger.rocksdb.toml` (default: `/data/ledger/rocksdb`).
   - Column families:
     - `glyphs` — generic glyph metadata.
     - `receipts` — receipt payloads.
     - `anchors` — AnchorGlyph metadata and Merkle roots.
   - Used for:
     - Long-term storage.
     - Full-path provenance queries.
     - History beyond the hot SQLite window.

All three must agree. If they don’t, ledger-explorer surfaces the inconsistency and refuses to serve proofs.

---

## CLI Contract

Binary name: `ledger-explorer`  
Entrypoint: `src/crates/ledger-explorer/src/main.rs`

```bash
# Return Merkle proof path + related receipts for a glyph or anchor
ledger-explorer query --glyph=<glyph_id_or_hash> \
                      [--tenant=<tenant_id>] \
                      [--json]

# Validate + append a ReceiptGlyph to the ledger
ledger-explorer append --receipt=<json_or_path> \
                       [--tenant=<tenant_id>] \
                       [--dry-run]

# Trigger compaction for a phase or full ledger (ties into weekly red-loop)
ledger-explorer compact --phase=<1-7|all> \
                        [--tenant=<tenant_id>]

# Emit DaemonStatusGlyph + ledger stats
ledger-explorer status [--tenant=<tenant_id>]
