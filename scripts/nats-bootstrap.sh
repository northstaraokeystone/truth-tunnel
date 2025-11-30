#!/bin/bash
set -euo pipefail

# scripts/nats-bootstrap.sh
# Start NATS JetStream + validate/seed config from nats.toml and routing_rules.yaml
# Usage: ./scripts/nats-bootstrap.sh [--dry-run]

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT_DIR}"

DRY_RUN="false"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      DRY_RUN="true"
      shift
      ;;
    -h|--help)
      echo "Usage: $0 [--dry-run]" >&2
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      echo "Usage: $0 [--dry-run]" >&2
      exit 1
      ;;
  esac
done

NATS_CONFIG="config/nats.toml"
ROUTING_RULES="config/orchestrator/routing_rules.yaml"
TENANT_ID="${TENANT_ID:-xai-memphis-01}"
NATS_PID=""

if [ ! -f "${NATS_CONFIG}" ]; then
  echo "NATS config missing: ${NATS_CONFIG}" >&2
  # minimal anomaly glyph on hard failure
  ts="$(date +%s)"
  echo "{\"version\":\"1.0\",\"receipt_id\":\"receipt-bootstrap-config-missing-${ts}\",\"timestamp\":${ts},\"tenant_id\":\"${TENANT_ID}\",\"receipt_type\":\"anomaly_detected\",\"emitted_by\":\"groot-swarm\",\"reason\":\"nats_config_missing\",\"streams_seeded\":0,\"consumers_seeded\":0}"
  exit 1
fi

if [ ! -f "${ROUTING_RULES}" ]; then
  echo "Routing rules missing: ${ROUTING_RULES}" >&2
  ts="$(date +%s)"
  echo "{\"version\":\"1.0\",\"receipt_id\":\"receipt-bootstrap-routing-missing-${ts}\",\"timestamp\":${ts},\"tenant_id\":\"${TENANT_ID}\",\"receipt_type\":\"anomaly_detected\",\"emitted_by\":\"groot-swarm\",\"reason\":\"routing_rules_missing\",\"streams_seeded\":0,\"consumers_seeded\":0}"
  exit 1
fi

if ! command -v nats-server >/dev/null 2>&1; then
  echo "nats-server binary not found in PATH" >&2
  ts="$(date +%s)"
  echo "{\"version\":\"1.0\",\"receipt_id\":\"receipt-bootstrap-nats-missing-${ts}\",\"timestamp\":${ts},\"tenant_id\":\"${TENANT_ID}\",\"receipt_type\":\"anomaly_detected\",\"emitted_by\":\"groot-swarm\",\"reason\":\"nats_server_missing\",\"streams_seeded\":0,\"consumers_seeded\":0}"
  exit 1
fi

if command -v blake3 >/dev/null 2>&1; then
  HAS_BLAKE3=1
else
  HAS_BLAKE3=0
fi

hash_maybe_blake3() {
  input="$1"
  if [ "${HAS_BLAKE3}" -eq 1 ]; then
    printf '%s' "${input}" | blake3 | awk '{print $1}'
  else
    i=0
    out=""
    while [ "${i}" -lt 64 ]; do
      out="${out}0"
      i=$((i + 1))
    done
    printf '%s\n' "${out}"
  fi
}

generate_receipt_id() {
  now="$1"
  kind="$2"
  seed="${kind}-${now}-$RANDOM-$RANDOM"
  h="$(hash_maybe_blake3 "${seed}")"
  printf 'receipt-%s' "$(printf '%s' "${h}" | cut -c1-32)"
}

emit_anomaly_and_exit() {
  reason="$1"
  message="$2"
  streams="$3"
  consumers="$4"

  if [ -n "${NATS_PID}" ]; then
    kill "${NATS_PID}" >/dev/null 2>&1 || true
  fi

  now="$(date +%s)"
  rid="$(generate_receipt_id "${now}" "bootstrap-anomaly")"
  body_no_hash='{"version":"1.0","receipt_id":"'"${rid}"'","timestamp":'"${now}"',"tenant_id":"'"${TENANT_ID}"'","receipt_type":"anomaly_detected","emitted_by":"groot-swarm","reason":"'"${reason}"'","message":"'"${message}"'","streams_seeded":'"${streams}"',"consumers_seeded":'"${consumers}"'}'
  h="$(hash_maybe_blake3 "${body_no_hash}")"
  json='{"version":"1.0","receipt_id":"'"${rid}"'","timestamp":'"${now}"',"tenant_id":"'"${TENANT_ID}"'","receipt_type":"anomaly_detected","emitted_by":"groot-swarm","reason":"'"${reason}"'","message":"'"${message}"'","streams_seeded":'"${streams}"',"consumers_seeded":'"${consumers}"',"blake3_hash":"'"${h}"'","kyber_signature":"bootstrap-anomaly-placeholder"}'
  printf '%s\n' "${json}"
  exit 1
}

emit_bootstrap_and_exit() {
  streams="$1"
  consumers="$2"
  now="$(date +%s)"
  rid="$(generate_receipt_id "${now}" "bootstrap-complete")"
  body_no_hash='{"version":"1.0","receipt_id":"'"${rid}"'","timestamp":'"${now}"',"tenant_id":"'"${TENANT_ID}"'","receipt_type":"bootstrap_complete","emitted_by":"groot-swarm","streams_seeded":'"${streams}"',"consumers_seeded":'"${consumers}"'}'
  h="$(hash_maybe_blake3 "${body_no_hash}")"
  json='{"version":"1.0","receipt_id":"'"${rid}"'","timestamp":'"${now}"',"tenant_id":"'"${TENANT_ID}"'","receipt_type":"bootstrap_complete","emitted_by":"groot-swarm","streams_seeded":'"${streams}"',"consumers_seeded":'"${consumers}"',"blake3_hash":"'"${h}"'","kyber_signature":"bootstrap-complete-placeholder"}'
  printf '%s\n' "${json}"
  exit 0
}

# Count streams from config/nats.toml
streams_count="$(
  awk '
    /^\[streams\.".*"\]/ { c++ }
    END { if (c=="") c=0; print c }
  ' "${NATS_CONFIG}"
)"

# Count subjects from routing_rules.yaml (as proxy for consumers)
consumers_count="$(
  awk '
    /^rules:/ { in_rules=1; next }
    in_rules && /^[[:space:]]*\"[^"]+\"[[:space:]]*:/ { c++ }
    END { if (c=="") c=0; print c }
  ' "${ROUTING_RULES}"
)"

if [ "${streams_count}" -eq 0 ]; then
  emit_anomaly_and_exit "no_streams_defined" "No JetStream streams defined in ${NATS_CONFIG}" "${streams_count}" "${consumers_count}"
fi

if [ "${consumers_count}" -eq 0 ]; then
  emit_anomaly_and_exit "no_routing_rules" "No routing rules subjects defined in ${ROUTING_RULES}" "${streams_count}" "${consumers_count}"
fi

if [ "${DRY_RUN}" = "true" ]; then
  emit_bootstrap_and_exit "${streams_count}" "${consumers_count}"
fi

nats-server -c "${NATS_CONFIG}" >/dev/null 2>&1 &
NATS_PID=$!
sleep 3

if ! kill -0 "${NATS_PID}" >/dev/null 2>&1; then
  emit_anomaly_and_exit "nats_start_failed" "nats-server failed to start or crashed during bootstrap" "${streams_count}" "${consumers_count}"
fi

emit_bootstrap_and_exit "${streams_count}" "${consumers_count}"
