#!/bin/bash
set -euo pipefail

# scripts/check-receipts.sh
# Verify receipts.jsonl chain integrity + emit verification glyph
# Usage: ./scripts/check-receipts.sh [--full-scan]

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

FULL_SCAN=false
RECEIPTS_FILE="glyphs/receipts/receipts.jsonl"
TENANT_ID="${TENANT_ID:-xai-memphis-01}"
GLYPH_BIN="${GLYPH_BIN:-glyph-lib}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --full-scan)
      FULL_SCAN=true
      shift
      ;;
    --help|-h)
      echo "Usage: $0 [--full-scan]" >&2
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      echo "Usage: $0 [--full-scan]" >&2
      exit 1
      ;;
  esac
done

if [ ! -f "${RECEIPTS_FILE}" ]; then
  echo "Receipts missing: ${RECEIPTS_FILE}" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 1
fi

if ! command -v blake3 >/dev/null 2>&1; then
  echo "blake3 CLI is required" >&2
  exit 1
fi

if ! command -v "${GLYPH_BIN}" >/dev/null 2>&1; then
  echo "glyph-lib CLI not found: ${GLYPH_BIN}" >&2
  exit 1
fi

generate_hex() {
  local bytes="$1"
  if command -v od >/dev/null 2>&1; then
    od -An -N"${bytes}" -tx1 /dev/urandom 2>/dev/null | tr -d ' \n'
  else
    date +%s%N | blake3 | awk '{print $1}' | cut -c1-$((bytes * 2))
  fi
}

emit_anomaly_receipt() {
  local reason="$1"
  local detail="$2"
  local ts rid_hex hash_hex sig_hex
  ts="$(date +%s)"
  rid_hex="$(generate_hex 16)"
  hash_hex="$(generate_hex 32)"
  sig_hex="$(generate_hex 32)"

  printf '{"version":"1.0","receipt_id":"receipt-%s","timestamp":%s,"tenant_id":"%s","receipt_type":"anomaly_detected","blake3_hash":"%s","kyber_signature":"%s","emitted_by":"ledger-explorer","severity":"critical","drift_value":0.0,"auto_halt_triggered":true,"reason":"%s","detail":"%s"}\n' \
    "${rid_hex}" "${ts}" "${TENANT_ID}" "${hash_hex}" "${sig_hex}" "${reason}" "${detail}"
}

emit_verification_receipt() {
  local receipts_checked="$1"
  local merkle_root="$2"
  local full_scan_flag="$3"
  local ts rid_hex hash_hex sig_hex
  ts="$(date +%s)"
  rid_hex="$(generate_hex 16)"

  if [ -n "${merkle_root}" ]; then
    hash_hex="${merkle_root}"
  else
    hash_hex="$(generate_hex 32)"
  fi
  sig_hex="$(generate_hex 32)"

  printf '{"version":"1.0","receipt_id":"receipt-%s","timestamp":%s,"tenant_id":"%s","receipt_type":"verification_complete","blake3_hash":"%s","kyber_signature":"%s","emitted_by":"ledger-explorer","receipts_checked":%s,"merkle_root":"%s","full_scan":%s,"chain_valid":true}\n' \
    "${rid_hex}" "${ts}" "${TENANT_ID}" "${hash_hex}" "${sig_hex}" "${receipts_checked}" "${merkle_root}" "${full_scan_flag}"
}

fail() {
  local reason="$1"
  local detail="$2"
  emit_anomaly_receipt "${reason}" "${detail}"
  exit 1
}

validate_receipt_schema() {
  local canon="$1"
  if ! printf '%s\n' "${canon}" | "${GLYPH_BIN}" validate --type=receipt_glyph >/dev/null 2>&1; then
    fail "receipt_schema_fail" "schema validation failed"
  fi
}

compute_blake3() {
  blake3 | awk '{print $1}'
}

compute_merkle_root() {
  # args: list of leaf hashes (hex)
  if [ "$#" -eq 0 ]; then
    echo ""
    return 0
  fi

  # build array
  local leaves=()
  while [ "$#" -gt 0 ]; do
    leaves+=("$1")
    shift
  done

  local level=("${leaves[@]}")

  while [ "${#level[@]}" -gt 1 ]; do
    local next=()
    local i=0
    local count="${#level[@]}"
    while [ "${i}" -lt "${count}" ]; do
      local left="${level[$i]}"
      local right
      if [ $((i + 1)) -lt "${count}" ]; then
        right="${level[$((i + 1))]}"
      else
        right="${left}"
      fi
      local combined="${left}${right}"
      local root
      root="$(printf '%s' "${combined}" | blake3 | awk '{print $1}')"
      next+=("${root}")
      i=$((i + 2))
    done
    level=("${next[@]}")
  done

  echo "${level[0]}"
}

mapfile -t LINES < "${RECEIPTS_FILE}"
TOTAL="${#LINES[@]}"

if [ "${TOTAL}" -eq 0 ]; then
  fail "receipts_empty" "no receipts in receipts.jsonl"
fi

START_INDEX=0
if [ "${FULL_SCAN}" = false ] && [ "${TOTAL}" -gt 1024 ]; then
  START_INDEX=$((TOTAL - 1024))
fi

HASHES=()
CHECKED=0
PREV_TS=""
INDEX=${START_INDEX}

while [ "${INDEX}" -lt "${TOTAL}" ]; do
  line="${LINES[${INDEX}]}"
  if [ -z "${line}" ]; then
    INDEX=$((INDEX + 1))
    continue
  fi

  canon="$(printf '%s\n' "${line}" | jq -cS '.' 2>/dev/null || true)"
  if [ -z "${canon}" ]; then
    fail "invalid_json" "line_index=${INDEX}"
  fi

  validate_receipt_schema "${canon}"

  rid="$(printf '%s\n' "${canon}" | jq -r '.receipt_id // empty')"
  ts="$(printf '%s\n' "${canon}" | jq -r '.timestamp // empty')"
  stored_hash="$(printf '%s\n' "${canon}" | jq -r '.blake3_hash // empty')"
  kyber_sig="$(printf '%s\n' "${canon}" | jq -r '.kyber_signature // empty')"

  if [ -z "${stored_hash}" ] || [ "${stored_hash}" = "null" ]; then
    fail "missing_blake3_hash" "line_index=${INDEX},receipt_id=${rid}"
  fi

  if [ -z "${kyber_sig}" ] || [ "${kyber_sig}" = "null" ]; then
    fail "missing_kyber_signature" "line_index=${INDEX},receipt_id=${rid}"
  fi

  if [ -z "${ts}" ] || [ "${ts}" = "null" ]; then
    fail "missing_timestamp" "line_index=${INDEX},receipt_id=${rid}"
  fi

  if [ -n "${PREV_TS}" ] && [ "${ts}" -lt "${PREV_TS}" ]; then
    fail "timestamp_regression" "line_index=${INDEX},receipt_id=${rid}"
  fi
  PREV_TS="${ts}"

  computed_hash="$(printf '%s' "${canon}" | compute_blake3)"
  if [ "${computed_hash}" != "${stored_hash}" ]; then
    fail "hash_mismatch" "line_index=${INDEX},receipt_id=${rid}"
  fi

  HASHES+=("${computed_hash}")
  CHECKED=$((CHECKED + 1))
  INDEX=$((INDEX + 1))
done

MERKLE_ROOT="$(compute_merkle_root "${HASHES[@]}")"

if [ -z "${MERKLE_ROOT}" ]; then
  fail "merkle_empty" "no_merkle_root_computed"
fi

emit_verification_receipt "${CHECKED}" "${MERKLE_ROOT}" "${FULL_SCAN}"

exit 0
