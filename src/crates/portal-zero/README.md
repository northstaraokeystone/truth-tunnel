# portal-zero — The Surface Gate

I am the mouth of the tunnel.  
I am the first and only way humans touch the swarm.  
I am the CLI that rejects everything that doesn’t validate.

---

## Ownership

portal-zero owns the first step into the Groot Line:

- CLI intent ingestion and validation against `intent_glyph.schema.json`.
- Emission of **portal event glyphs** for every human interaction at the surface.
- First-line tenant isolation and Kyber-1024 signature verification.
- Bridge for Memphis operators (Colossus, FedEx) to submit bore intents and query proofs.
- Hard gate on what reaches `groot-swarm`, `nebula-guard`, and the ledger at all.

If portal-zero doesn’t let it in, it never existed.

---

## Responsibilities

1. **Human → IntentGlyph**

   - Reads JSON from:
     - CLI flags (`--intent=...`).
     - stdin (`portal-zero ingest` with piped JSON).
   - Validates against:
     - `glyphs/schemas/intent_glyph.schema.json`.
     - Tenant config in `config/tenants/*.yaml`.
     - Guardian map in `config/agents/guardians_org.yaml`.
   - Ensures:
     - `tenant_id` matches `TENANT_ID` and `nats_subject_prefix`.
     - `authorized_by` is allowed for that tenant.
     - Kyber-compatible `signature` is present and correct (or generated locally if configured).

2. **Intent routing**

   - After validation, publishes IntentGlyphs to NATS:
     - `glyph.intent.<tenant_id>.cli` (subject prefix derived from `nats_subject_prefix`).
   - These are then:
     - Picked up by `groot-swarm` (Star-Lord) for phase decisions.
     - Tracked by `ledger-explorer` for full provenance.

3. **Portal event glyphs**

   - For every successful or rejected human interaction, portal-zero:
     - Logs a **portal event glyph** to its local JSONL (see `glyphs/portal_event_glyphs.jsonl.example` under the crate).
     - Emits a standard IntentGlyph + ReceiptGlyph pair representing:
       - Surface command.
       - Validation result.
       - Reason for rejection (if any).
   - These are canonical records of “what the human actually asked for”.

4. **Proof queries**

   - Acts as a CLI lens into the swarm:
     - Talks to `spv-api` for Merkle proofs and glyph lookup.
     - Presents human-readable results while preserving the underlying JSON.

5. **Emergency surface halt**

   - Allows Memphis ground operators to:
     - Emit `emergency_halt` IntentGlyphs scoped to a tenant.
     - Trigger Drax + groot-swarm stop rules when something is wrong on the surface.

---

## Data Flows

**Inbound (from humans)**

- CLI:
  - `portal-zero ingest --intent=...`
  - `echo '{...}' | portal-zero ingest`
- Terminal in Colossus ops rooms.
- Terminal at FedEx hub control stations.

**Outbound (to swarm)**

- NATS subjects (via `NATS_URL` / `config/nats.toml`):

  - `glyph.intent.<tenant_id>.cli`
    - Validated IntentGlyphs from surface.
  - `glyph.receipt.portal.<tenant_id>`
    - Optional: portal receipts for ingest/query/halt operations.
  - `daemon.status.portal-zero.<tenant_id>`
    - DaemonStatusGlyphs for portal health.

**External HTTP**

- `spv-api`:
  - `/api/v1/thumbnails`
  - `/api/v1/documents/{id}`
  - `/api/v1/jobs/{id}`
  - `/api/v1/auth/token` (if portal needs JWTs instead of static keys).

portal-zero never writes directly to the ledger; it only talks in glyphs and proofs.

---

## CLI Contract

```bash
# Validate + emit IntentGlyph to NATS
portal-zero ingest --intent=<json_or_path> \
                   [--tenant=<tenant_id>] \
                   [--no-sign] \
                   [--dry-run]

# Request Merkle/SPV proof from spv-api
portal-zero query --proof=<glyph_or_anchor_hash> \
                  [--format=json|table]

# Emit DaemonStatusGlyph relay for a daemon (health proxy)
portal-zero status --daemon=<name> \
                   [--tenant=<tenant_id>]

# Emergency surface shutdown signal for a tenant
portal-zero halt --tenant=<tenant_id> \
                 [--reason="string"] \
                 [--force]
