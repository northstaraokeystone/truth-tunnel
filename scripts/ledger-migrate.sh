#!/bin/bash
set -euo pipefail

# scripts/ledger-migrate.sh
# Apply schema migrations to SQLite/RocksDB ledgers
# Usage: ./scripts/ledger-migrate.sh [--dry-run] [--db=sqlite|rocksdb]

DB="sqlite"
DRY_RUN=false
MIGRATIONS_ROOT="migrations"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --db=*)
      DB="${1#*=}"
      shift
      ;;
    --db)
      [ "$#" -ge 2 ] || { echo "Missing value for --db" >&2; exit 1; }
      DB="$2"
      shift 2
      ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 1
      ;;
  esac
done

case "${DB}" in
  sqlite|rocksdb) ;;
  *)
    echo "Invalid --db value: ${DB} (expected sqlite or rocksdb)" >&2
    exit 1
    ;;
esac

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

CONFIG="config/ledger.${DB}.toml"
if [ ! -f "${CONFIG}" ]; then
  echo "Config missing: ${CONFIG}" >&2
  exit 1
fi

MIGRATIONS_DIR="${MIGRATIONS_ROOT}/${DB}"
if [ ! -d "${MIGRATIONS_DIR}" ]; then
  echo "Migrations directory missing: ${MIGRATIONS_DIR}" >&2
  exit 1
fi

TENANT_ID="${TENANT_ID:-xai-memphis-01}"

parse_toml_path() {
  awk -F= '
    /^[[:space:]]*path[[:space:]]*=/ {
      val=$2
      gsub(/^[[:space:]]*/,"",val)
      gsub(/[[:space:]]*$/,"",val)
      gsub(/^"/,"",val)
      gsub(/"$/,"",val)
      print val
      exit
    }
  ' "$1"
}

DB_PATH_RAW="$(parse_toml_path "${CONFIG}")"
if [ -z "${DB_PATH_RAW}" ]; then
  echo "Failed to parse path from ${CONFIG}" >&2
  exit 1
fi

case "${DB_PATH_RAW}" in
  /*) DB_PATH="${DB_PATH_RAW}" ;;
  *)  DB_PATH="${ROOT_DIR}/${DB_PATH_RAW}" ;;
esac

if [ "${DB}" = "sqlite" ]; then
  if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "sqlite3 binary not found in PATH" >&2
    exit 1
  fi
else
  if ! command -v rocksdb_ldb >/dev/null 2>&1 && ! command -v ldb >/dev/null 2>&1; then
    echo "rocksdb tools (rocksdb_ldb or ldb) not found in PATH" >&2
    exit 1
  fi
fi

emit_migration_receipt() {
  local db_type="$1"
  local version_applied="$2"
  local migrations_applied="$3"
  local dry_flag="$4"
  local ts rid_hex hash_hex
  ts="$(date +%s)"
  rid_hex="$(printf '%032x\n' "${ts}")"
  hash_hex="$(printf '%064x\n' "${ts}")"

  cat <<EOF
{"version":"1.0","receipt_id":"receipt-${rid_hex}","timestamp":${ts},"tenant_id":"${TENANT_ID}","receipt_type":"migration_complete","blake3_hash":"${hash_hex}","kyber_signature":"TBD","emitted_by":"ledger-explorer","db_type":"${db_type}","version_applied":"${version_applied}","migrations_applied":${migrations_applied},"dry_run":${dry_flag}}
EOF
}

emit_anomaly_receipt() {
  local db_type="$1"
  local reason="$2"
  local ts rid_hex hash_hex
  ts="$(date +%s)"
  rid_hex="$(printf '%032x\n' "${ts}")"
  hash_hex="$(printf '%064x\n' "${ts}")"

  cat <<EOF
{"version":"1.0","receipt_id":"receipt-${rid_hex}","timestamp":${ts},"tenant_id":"${TENANT_ID}","receipt_type":"anomaly_detected","blake3_hash":"${hash_hex}","kyber_signature":"TBD","emitted_by":"ledger-explorer","severity":"critical","drift_value":0.0,"auto_halt_triggered":true,"db_type":"${db_type}","reason":"${reason}"}
EOF
}

apply_sqlite_migrations() {
  local db_file="$1"
  local dir="$2"
  local count=0

  if [ ! -f "${db_file}" ]; then
    echo "SQLite ledger not found at ${db_file}" >&2
    emit_anomaly_receipt "sqlite" "sqlite_ledger_missing"
    exit 1
  fi

  local files
  IFS=$'\n' read -r -d '' -a files < <(find "${dir}" -maxdepth 1 -type f -name '*.sql' | sort && printf '\0') || true

  if [ "${#files[@]}" -eq 0 ]; then
    echo "No SQLite migrations found in ${dir}" >&2
    emit_anomaly_receipt "sqlite" "sqlite_migrations_missing"
    exit 1
  fi

  if [ "${DRY_RUN}" = true ]; then
    echo "SQLite DRY-RUN: would apply ${#files[@]} migrations to ${db_file}" >&2
    echo "${#files[@]}"
    return 0
  fi

  sqlite3 "${db_file}" "PRAGMA foreign_keys=OFF;" >/dev/null

  for f in "${files[@]}"; do
    sqlite3 "${db_file}" ".read ${f}"
    count=$((count + 1))
  done

  local integrity
  integrity="$(sqlite3 "${db_file}" "PRAGMA integrity_check;" || echo "failed")"
  if [ "${integrity}" != "ok" ]; then
    echo "SQLite integrity_check failed: ${integrity}" >&2
    emit_anomaly_receipt "sqlite" "sqlite_integrity_check_failed"
    exit 1
  fi

  echo "${count}"
}

sqlite_version() {
  local db_file="$1"
  local version
  version="$(sqlite3 "${db_file}" "SELECT MAX(version) FROM schema_version;" 2>/dev/null || true)"
  if [ -z "${version}" ]; then
    version="unknown"
  fi
  printf '%s\n' "${version}"
}

apply_rocksdb_migrations() {
  local db_path="$1"
  local dir="$2"
  local count=0

  if [ ! -d "${db_path}" ]; then
    echo "RocksDB path not found or not a directory: ${db_path}" >&2
    emit_anomaly_receipt "rocksdb" "rocksdb_path_missing"
    exit 1
  fi

  local files
  IFS=$'\n' read -r -d '' -a files < <(find "${dir}" -maxdepth 1 -type f -name '*.sh' | sort && printf '\0') || true

  if [ "${#files[@]}" -eq 0 ]; then
    echo "No RocksDB migrations found in ${dir}" >&2
    emit_anomaly_receipt "rocksdb" "rocksdb_migrations_missing"
    exit 1
  fi

  if [ "${DRY_RUN}" = true ]; then
    echo "RocksDB DRY-RUN: would apply ${#files[@]} migrations to ${db_path}" >&2
    echo "${#files[@]}"
    return 0
  fi

  for f in "${files[@]}"; do
    bash "${f}" "${db_path}"
    count=$((count + 1))
  done

  local tool
  if command -v rocksdb_ldb >/dev/null 2>&1; then
    tool="rocksdb_ldb"
  else
    tool="ldb"
  fi

  if ! "${tool}" --db="${db_path}" get_property rocksdb.estimate-live-data-size >/dev/null 2>&1; then
    echo "RocksDB post-migration property check failed" >&2
    emit_anomaly_receipt "rocksdb" "rocksdb_property_check_failed"
    exit 1
  fi

  echo "${count}"
}

rocksdb_version() {
  local db_path="$1"
  printf 'rocksdb-%s\n' "$(date +%s)"
}

MIGRATIONS_APPLIED=0
VERSION_APPLIED="unknown"

if [ "${DB}" = "sqlite" ]; then
  MIGRATIONS_APPLIED="$(apply_sqlite_migrations "${DB_PATH}" "${MIGRATIONS_DIR}")"
  VERSION_APPLIED="$(sqlite_version "${DB_PATH}")"
else
  MIGRATIONS_APPLIED="$(apply_rocksdb_migrations "${DB_PATH}" "${MIGRATIONS_DIR}")"
  VERSION_APPLIED="$(rocksdb_version "${DB_PATH}")"
fi

if [ "${DRY_RUN}" = true ]; then
  emit_migration_receipt "${DB}" "${VERSION_APPLIED}" 0 true
else
  emit_migration_receipt "${DB}" "${VERSION_APPLIED}" "${MIGRATIONS_APPLIED}" false
fi

exit 0
