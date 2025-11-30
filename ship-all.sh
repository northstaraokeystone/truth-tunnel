#!/bin/bash
set -euo pipefail

PHASE=""
PHASE_MAP="config/orchestrator/phase_map.yaml"

while [ $# -gt 0 ]; do
  case "$1" in
    --phase=*)
      PHASE="${1#*=}"
      ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 1
      ;;
  esac
  shift
done

if [ -z "${PHASE}" ]; then
  if command -v yq >/dev/null 2>&1 && [ -f "${PHASE_MAP}" ]; then
    PHASE="$(yq e '.current_phase // .phase // 1' "${PHASE_MAP}")"
  else
    PHASE="1"
  fi
fi

DAEMONS=""
if command -v yq >/dev/null 2>&1 && [ -f "${PHASE_MAP}" ]; then
  DAEMONS="$(yq e -r ".phases.\"${PHASE}\".daemons[]" "${PHASE_MAP}" 2>/dev/null || true)"
fi

if [ -z "${DAEMONS}" ]; then
  case "${PHASE}" in
    1)
      DAEMONS="groot-swarm ledger-explorer"
      ;;
    2)
      DAEMONS="groot-swarm nebula-guard rocket-engine digital-twin-groot"
      ;;
    3)
      DAEMONS="groot-swarm ledger-explorer"
      ;;
    4)
      DAEMONS="rocket-engine nebula-guard digital-twin-groot"
      ;;
    5)
      DAEMONS="star-lord-orchestrator groot-swarm"
      ;;
    6)
      DAEMONS="spv-api mantis-community nebula-guard"
      ;;
    7)
      DAEMONS="drax-metrics groot-swarm"
      ;;
    *)
      DAEMONS="groot-swarm"
      ;;
  esac
fi

if [ -z "${DAEMONS}" ]; then
  echo "No daemons configured for phase ${PHASE}" >&2
  exit 1
fi

PIDS=()
FAILED=0

for daemon in ${DAEMONS}; do
  echo "Starting daemon: ${daemon} (phase ${PHASE})"
  cargo run --bin "${daemon}" -- ship --phase="${PHASE}" &
  pid=$!
  PIDS+=("${pid}")
done

sleep 5

for pid in "${PIDS[@]}"; do
  if ! kill -0 "${pid}" 2>/dev/null; then
    FAILED=1
  fi
done

timestamp="$(date +%s)"

build_daemon_array() {
  local json="["
  local first=1
  for d in ${DAEMONS}; do
    if [ "${first}" -eq 0 ]; then
      json="${json},"
    fi
    json="${json}\"${d}\""
    first=0
  done
  json="${json}]"
  printf '%s' "${json}"
}

DAEMONS_JSON="$(build_daemon_array)"

if [ "${FAILED}" -ne 0 ]; then
  for pid in "${PIDS[@]}"; do
    kill "${pid}" 2>/dev/null || true
  done
  printf '{"version":"1.0","receipt_type":"anomaly_receipt","timestamp":%s,"phase":%s,"reason":"daemon_start_failed","daemons":%s,"emitted_by":"ship-all.sh"}\n' \
    "${timestamp}" "${PHASE}" "${DAEMONS_JSON}"
  exit 1
fi

printf '{"version":"1.0","receipt_type":"shipping_receipt","timestamp":%s,"phase":%s,"daemons_started":%s,"emitted_by":"ship-all.sh"}\n' \
  "${timestamp}" "${PHASE}" "${DAEMONS_JSON}"

echo "Phase ${PHASE} daemons started."
