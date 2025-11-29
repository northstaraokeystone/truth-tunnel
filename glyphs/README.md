# Glyph System — Truth-Tunnel

**Version:** 0.1 (Nov 29 2025)  
**Root Hash:** [BLAKE3 to be filled on commit]

---

## Purpose

Glyphs are the only language of Truth-Tunnel: every state transition, decision, anomaly, and prediction must be encoded as a verifiable glyph or it did not happen.

---

## Glyph Lifecycle (The Only Truth)

From Memphis soil to orbit and back, every action proceeds through the same pipeline.

```mermaid
flowchart LR
    I[IntentGlyph<br/>glyphs/schemas/intent_glyph.schema.json] --> G[Guardian daemon<br/>(Rocket / Gamora / Groot / etc.)]
    G --> R[ReceiptGlyph(s)<br/>glyphs/schemas/receipt_glyph.schema.json]
    R --> A[AnchorGlyph<br/>glyphs/schemas/anchor_glyph.schema.json]
    A --> L[Ledger (SQLite/RocksDB + receipts.jsonl)]
    L --> P[Arweave/IPFS anchor<br/>(via scripts/check-receipts.sh)]

    subgraph Guardians
      G1[rocket-engine]:::daemon
      G2[nebula-guard]:::daemon
      G3[digital-twin-groot]:::daemon
      G4[drax-metrics]:::daemon
      G5[groot-swarm]:::daemon
    end

    I --> G1
    I --> G2
    I --> G3

    classDef daemon fill=#111111,stroke=#555555,color=#f5f5f5;