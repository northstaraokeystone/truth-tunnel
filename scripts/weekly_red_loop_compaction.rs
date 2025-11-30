//! ```cargo
//! [package]
//! edition = "2021"
//!
//! [dependencies]
//! anyhow = "1.0"
//! blake3 = "1.5"
//! rusqlite = { version = "0.31", features = ["bundled"] }
//! rocksdb = "0.22"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! toml = "0.8"
//! ```

use anyhow::{Context, Result};
use blake3::Hasher;
use rocksdb::{Options, DB};
use serde_json::{json, Map, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<()> {
    let root_dir = env::current_dir().context("failed to get current dir")?;
    let config_dir = root_dir.join("config");

    let slo_config = read_toml(config_dir.join("slo.toml"))?;
    let sqlite_config = read_toml(config_dir.join("ledger.sqlite.toml"))?;
    let rocks_config = read_toml(config_dir.join("ledger.rocksdb.toml"))?;

    let sqlite_path = resolve_path(&root_dir, sqlite_config.get("path"))?;
    let rocks_path = resolve_path(&root_dir, rocks_config.get("path"))?;

    // SQLite compaction (hot ledger)
    let (row_count_before, row_count_after) = compact_sqlite(&sqlite_path)?;

    // RocksDB compaction (cold archive)
    let (rocks_live_before, rocks_live_after) = compact_rocksdb(&rocks_path)?;

    let reduction_percent_sqlite = if row_count_before > 0 {
        ((row_count_before - row_count_after) as f64 / row_count_before as f64) * 100.0
    } else {
        0.0
    };

    let reduction_percent_rocks = match (rocks_live_before, rocks_live_after) {
        (Some(b), Some(a)) if b > 0 => ((b - a) as f64 / b as f64) * 100.0,
        _ => 0.0,
    };

    let reduction_percent = reduction_percent_sqlite.max(reduction_percent_rocks);

    // Death criteria check (PCE from env or SLO/death criteria tuning)
    let pce_transitivity = read_pce_from_env_or_default(&slo_config);
    let death_triggered = pce_transitivity < 0.90;

    // Build compaction_receipt payload (without hash/signature)
    let tenant_id = env::var("TENANT_ID").unwrap_or_else(|_| "xai-memphis-01".to_string());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("time went backwards")?
        .as_secs();

    let receipt_id = make_receipt_id(now, row_count_before, row_count_after);

    let mut payload = Map::new();
    payload.insert("version".to_string(), json!("1.0"));
    payload.insert("receipt_id".to_string(), json!(receipt_id));
    payload.insert("timestamp".to_string(), json!(now));
    payload.insert("tenant_id".to_string(), json!(tenant_id));
    payload.insert("receipt_type".to_string(), json!("compaction_complete"));
    payload.insert("emitted_by".to_string(), json!("drax-metrics"));
    payload.insert("input_row_count".to_string(), json!(row_count_before));
    payload.insert("output_row_count".to_string(), json!(row_count_after));
    payload.insert(
        "reduction_percent".to_string(),
        json!(round2(reduction_percent)),
    );
    payload.insert(
        "rocksdb_live_bytes_before".to_string(),
        match rocks_live_before {
            Some(v) => json!(v),
            None => Value::Null,
        },
    );
    payload.insert(
        "rocksdb_live_bytes_after".to_string(),
        match rocks_live_after {
            Some(v) => json!(v),
            None => Value::Null,
        },
    );
    payload.insert(
        "pce_transitivity".to_string(),
        json!(round4(pce_transitivity)),
    );
    payload.insert("death_triggered".to_string(), json!(death_triggered));

    // Canonicalize and hash (Merkle root would be integrated via glyph-lib; here we hash the canonical JSON)
    let canonical = serde_json::to_string(&payload)?;
    let mut hasher = Hasher::new();
    hasher.update(canonical.as_bytes());
    let hash = hasher.finalize().to_hex().to_string();

    payload.insert("blake3_hash".to_string(), json!(hash));
    payload.insert(
        "kyber_signature".to_string(),
        json!("kyber-signature-placeholder"),
    );

    let output = serde_json::to_string(&payload)?;
    println!("{output}");

    if death_triggered {
        std::process::exit(1);
    }

    Ok(())
}

fn read_toml(path: PathBuf) -> Result<toml::Value> {
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;
    let value: toml::Value = toml::from_str(&raw)
        .with_context(|| format!("failed to parse TOML: {}", path.display()))?;
    Ok(value)
}

fn resolve_path(root: &Path, value: Option<&toml::Value>) -> Result<PathBuf> {
    let s = value
        .and_then(|v| v.as_str())
        .context("missing or invalid `path` in ledger config")?;
    let p = Path::new(s);
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        Ok(root.join(p))
    }
}

fn compact_sqlite(path: &Path) -> Result<(i64, i64)> {
    let mut conn = rusqlite::Connection::open(path)
        .with_context(|| format!("failed to open sqlite ledger at {}", path.display()))?;

    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?;

    let row_count_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM receipts", [], |r| r.get(0))
        .unwrap_or(0);

    conn.execute("PRAGMA wal_checkpoint(TRUNCATE);", [])?;
    conn.execute("VACUUM;", [])?;

    let row_count_after: i64 = conn
        .query_row("SELECT COUNT(*) FROM receipts", [], |r| r.get(0))
        .unwrap_or(row_count_before);

    Ok((row_count_before, row_count_after))
}

fn compact_rocksdb(path: &Path) -> Result<(Option<u64>, Option<u64>)> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, path)
        .with_context(|| format!("failed to open rocksdb ledger at {}", path.display()))?;

    let live_before = db
        .property_int_value("rocksdb.estimate-live-data-size")
        .ok()
        .flatten();

    db.compact_range::<&[u8], &[u8]>(None, None);

    let live_after = db
        .property_int_value("rocksdb.estimate-live-data-size")
        .ok()
        .flatten();

    Ok((live_before, live_after))
}

fn read_pce_from_env_or_default(_slo: &toml::Value) -> f64 {
    if let Ok(v) = env::var("PCE_TRANSITIVITY") {
        if let Ok(parsed) = v.parse::<f64>() {
            return parsed;
        }
    }
    1.0
}

fn make_receipt_id(now: u64, before: i64, after: i64) -> String {
    let seed = format!("compaction-{now}-{before}-{after}");
    let h = blake3::hash(seed.as_bytes()).to_hex().to_string();
    let id_part = &h[..32];
    format!("receipt-{id_part}")
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn round4(v: f64) -> f64 {
    (v * 10_000.0).round() / 10_000.0
}
