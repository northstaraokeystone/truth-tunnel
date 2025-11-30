#!/bin/bash
set -euo pipefail

# scripts/deploy-manifest.sh
# SpaceX-style deploy: Manifest → validate → apply → receipt

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

STAGE=""
MANIFEST=""

usage() {
  echo "Usage: $0 --stage=<0|1> --manifest=<manifest.yaml>" >&2
  exit 1
}

while [ $# -gt 0 ]; do
  case "$1" in
    --stage=*)
      STAGE="${1#*=}"
      shift
      ;;
    --stage)
      [ $# -ge 2 ] || usage
      STAGE="$2"
      shift 2
      ;;
    --manifest=*)
      MANIFEST="${1#*=}"
      shift
      ;;
    --manifest)
      [ $# -ge 2 ] || usage
      MANIFEST="$2"
      shift 2
      ;;
    -h|--help)
      usage
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      ;;
  esac
done

if [ -z "${STAGE}" ]; then
  echo "Error: --stage required" >&2
  usage
fi

if [ -z "${MANIFEST}" ]; then
  echo "Error: --manifest required" >&2
  usage
fi

MANIFEST_PATH="ops/manifests/${MANIFEST}"

if [ ! -f "${MANIFEST_PATH}" ]; then
  echo "Manifest not found: ${MANIFEST_PATH}" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found in PATH" >&2
  exit 1
fi

if ! command -v nats >/dev/null 2>&1; then
  echo "warning: nats CLI not found; glyphs will be printed to stdout only" >&2
fi

emit_glyph() {
  kind="$1"
  subject="$2"
  shift 2

  payload="$(cargo run -q -p glyph-lib -- emit-glyph --kind="${kind}" "$@")" || {
    echo "Failed to emit glyph of kind ${kind}" >&2
    return 1
  }

  if command -v nats >/dev/null 2>&1; then
    printf '%s\n' "${payload}" | nats pub "${subject}" >/dev/null
  else
    printf '%s\n' "${payload}"
  fi
}

emit_anomaly() {
  reason="$1"
  manifest="$2"
  emit_glyph "anomaly_receipt" "glyph.receipt.anomaly" \
    --reason="${reason}" \
    --manifest="${manifest}" \
    --stage="${STAGE}" || true
}

emit_deploy() {
  manifest="$1"
  emit_glyph "deploy_glyph" "glyph.receipt.deploy" \
    --manifest="${manifest}" \
    --stage="${STAGE}" || true
}

validate_manifest() {
  manifest="$1"
  if ! cargo run -q -p glyph-lib -- validate-manifest --manifest="${manifest}"; then
    echo "Manifest validation failed (hash/AnchorGlyph mismatch): ${manifest}" >&2
    emit_anomaly "manifest_hash_mismatch" "${manifest}"
    exit 1
  fi
}

apply_stage0() {
  manifest="$1"
  cargo build --release
  if [ -x "./ship-all.sh" ]; then
    ./ship-all.sh
  fi
  if command -v rsync >/dev/null 2>&1; then
    mkdir -p "${ROOT_DIR}/.deploy/stage0"
    rsync -a --delete "${ROOT_DIR}/target/release/" "${ROOT_DIR}/.deploy/stage0/"
  fi
  echo "Stage0 applied using manifest ${manifest}" >&2
}

apply_stage1() {
  manifest="$1"
  if command -v ansible-playbook >/dev/null 2>&1; then
    ansible-playbook -i ops/inventory "${manifest}"
  else
    echo "[stub] ansible-playbook would apply cluster manifest ${manifest}" >&2
  fi
}

check_slo_and_maybe_rollback() {
  manifest="$1"
  if ! cargo run -q -p drax-metrics -- score --vih=deploy; then
    echo "SLO breach detected after deploy for manifest ${manifest}" >&2
    rollback_name="${manifest%.*}_rollback.${manifest##*.}"
    rollback_path="ops/manifests/${rollback_name}"
    if [ -f "${rollback_path}" ]; then
      echo "Attempting rollback using ${rollback_path}" >&2
      "${ROOT_DIR}/scripts/deploy-manifest.sh" --stage="${STAGE}" --manifest="${rollback_name}" || true
    else
      echo "No rollback manifest found at ${rollback_path}" >&2
    fi
    emit_anomaly "slo_breach_deploy" "${manifest}"
    exit 1
  fi
}

validate_manifest "${MANIFEST_PATH}"

case "${STAGE}" in
  0)
    apply_stage0 "${MANIFEST_PATH}"
    ;;
  1)
    apply_stage1 "${MANIFEST_PATH}"
    ;;
  *)
    echo "Invalid stage: ${STAGE} (expected 0 or 1)" >&2
    exit 1
    ;;
esac

check_slo_and_maybe_rollback "${MANIFEST_PATH}"

emit_deploy "${MANIFEST_PATH}"

echo "Deploy complete: ${MANIFEST} (stage ${STAGE})" >&2
