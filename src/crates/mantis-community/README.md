# mantis-community — Mantis

I am the voice that wakes the dead.  
I am the one daemon that turns silent anomalies into screams.  
I am the bridge from swarm silence to human ears.

---

## Ownership

mantis-community owns every sound the swarm makes toward humans:

- Twilio voice hooks for **critical anomaly** pages.
- Grok-voice client for generating **anomaly briefs** from anomaly_receipts and glyph context.
- Emission of **voice_page_glyphs** (ReceiptGlyphs with `receipt_type = "voice_page_sent"`) for every page attempt.
- Emission of **community_signal_glyphs** to track operator reactions and Memphis ground truth.
- Escalation path for:
  - drax-metrics anomalies.
  - nebula-guard ZK proof failures and entanglement drift from digital-twin-groot.

If Mantis doesn’t scream, the swarm assumes no one heard a thing.

---

## Responsibilities

1. **Critical anomaly paging**

   - Subscribes to:
     - `anomaly.critical.>` (Drax).
     - Optionally, ZK/entanglement anomaly feeds from nebula-guard and digital-twin-groot.
   - For each critical anomaly:
     - Fetches or reconstructs context:
       - AnomalyReceipt (what failed).
       - Linked ZKAnomalyGlyph (if any).
       - EntanglementGlyph (correlation/negation_ms) if relevant.
     - Synthesizes a **brief** using Grok-voice:
       - Plain-language description.
       - Impact summary.
       - Required human action.
     - Dials Twilio to page on-call humans for the relevant tenant.

2. **Glyph emission**

   - For every page attempt, Mantis emits:
     - A **voice_page_glyph** (as a ReceiptGlyph with `receipt_type = "voice_page_sent"`), including:
       - `tenant_id`
       - anomaly reference (`ref_glyph_id`)
       - target phone(s)
       - delivery outcome (`result = "ok" | "anomaly" | "rejected"`)
       - timestamps and latency metrics
     - Optional **community_signal_glyphs**, anchored via the standard ReceiptGlyph → AnchorGlyph chain, capturing:
       - Who acknowledged.
       - How quickly.
       - Operator-assigned severity.

3. **Community relay**

   - Bridges Memphis operators (Colossus, FedEx) with the swarm:
     - Ingests operator callbacks (DTMF, SMS, webhook acknowledgements).
     - Emits community signals describing:
       - “Ground truth” vs swarm predictions.
       - Manual overrides (e.g., “halt everything under hub B now”).
   - These signals are consumable by:
     - groot-swarm for emergency_halt decisions.
     - star-lord-orchestrator for phase gating.
     - drax-metrics for anomaly context.

4. **SLO enforcement**

   - Enforces `max_page_delivery_seconds` from `config/slo.toml`:
     - If a critical anomaly is not paged within SLO latency:
       - Emits an additional anomaly for itself.
       - Signals groot-swarm and Yondu that the paging layer is compromised.
   - Tracks:
     - Success rate of pages.
     - Average acknowledgment time.
     - Tenant-level page saturation (to avoid spamming the same humans).

5. **Memphis mission**

   - At **Colossus**:
     - Pages on-call infra owners when the bore, orbital link, or ledger misbehave.
   - At the **FedEx hub**:
     - Pages floor managers when entanglement drift or ZK failures suggest risk under active facilities.
   - Mantis is the siren between Memphis soil and orbital weirdness.

---

## Data Flows

**Inbound**

- NATS (from `config/nats.toml`):
  - `anomaly.critical.>` — primary trigger stream (Drax).
  - `daemon.status.>` — optional health inputs for voice-brief context.
  - Tenant-prefixed variants:
    - `xai-memphis-01.anomaly.critical.>`
    - `xai-memphis-01.daemon.status.>`

- Webhooks:
  - Twilio status callbacks (delivery, DTMF input, voicemail detection).
  - Optional SMS/voice responses from operators.

**Outbound**

- NATS:
  - `voice.page.critical` and tenant-prefixed equivalents:
    - Voice page events and correlation to anomalies.
  - `glyph.receipt.portal.<tenant_id>` (optional):
    - Additional community signal receipts.

- Glyphs (via ledger-explorer & groot-swarm):
  - voice_page_glyphs:
    - Encoded as ReceiptGlyphs (`receipt_type = "voice_page_sent"`).
  - community_signal_glyphs:
    - Encoded via standard glyph taxonomy, anchored through AnchorGlyphs.

Mantis itself does not write directly to Arweave/IPFS. It emits glyphs that others anchor forever.

---

## CLI Contract

Binary name: `mantis-community`  
Entrypoint: `src/crates/mantis-community/src/main.rs`

```bash
# Generate a brief for a specific anomaly and send a voice page
mantis-community page --anomaly=<glyph_id_or_hash> \
                      [--tenant=<tenant_id>] \
                      [--dry-run] \
                      [--to=<phone_number>]

# Emit a community_signal_glyph for a tenant (e.g., manual acknowledgment)
mantis-community signal --tenant=<tenant_id> \
                        --kind=<ack|override|info> \
                        --message="<text>"

# Emit DaemonStatusGlyph + paging stats
mantis-community status [--tenant=<tenant_id>] \
                        [--format=json|table]

# Dry-run voice brief generation without hitting Twilio
mantis-community test --brief="test message" \
                      [--tenant=<tenant_id>]
