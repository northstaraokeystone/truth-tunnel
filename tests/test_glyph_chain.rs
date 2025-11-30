#[cfg(test)]
mod test_glyph_chain {
    use blake3;
    use hex;
    use serde_json::{json, Number, Value};
    use std::fs;
    use std::path::PathBuf;

    fn manifest_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn load_example(name: &str) -> Value {
        let path = manifest_root()
            .join("glyphs")
            .join("examples")
            .join(name);
        let data = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        serde_json::from_str(&data)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
    }

    fn set_number(obj: &mut serde_json::Map<String, Value>, key: &str, v: u64) {
        obj.insert(key.to_string(), Value::Number(Number::from(v)));
    }

    fn set_f64(obj: &mut serde_json::Map<String, Value>, key: &str, v: f64) {
        obj.insert(
            key.to_string(),
            Value::Number(Number::from_f64(v).expect("valid f64")),
        );
    }

    fn make_receipt_id(i: u32) -> String {
        format!("receipt-{i:032x}")
    }

    fn canonical_for_hash(mut v: Value) -> Vec<u8> {
        if let Some(obj) = v.as_object_mut() {
            obj.insert("blake3_hash".to_string(), Value::String(String::new()));
            obj.insert("kyber_signature".to_string(), Value::String(String::new()));
        }
        serde_json::to_vec(&v).expect("canonicalization")
    }

    fn recompute_blake3(receipt: &mut Value) -> String {
        let canonical = canonical_for_hash(receipt.clone());
        let h = blake3::hash(&canonical);
        let hex = h.to_hex().to_string();
        if let Some(obj) = receipt.as_object_mut() {
            obj.insert("blake3_hash".to_string(), Value::String(hex.clone()));
        }
        hex
    }

    fn kyber_stub_sign(root: &[u8]) -> String {
        let h = blake3::hash(root);
        hex::encode(h.as_bytes())
    }

    fn hex_to_array32(h: &str) -> [u8; 32] {
        let bytes = hex::decode(h).expect("valid hex");
        assert_eq!(bytes.len(), 32, "expected 32-byte blake3 hash");
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        arr
    }

    fn merkle_root(hashes: &[[u8; 32]]) -> [u8; 32] {
        assert!(!hashes.is_empty(), "hash list must be non-empty");
        let mut level: Vec<[u8; 32]> = hashes.to_vec();
        while level.len() > 1 {
            let mut next = Vec::with_capacity((level.len() + 1) / 2);
            let mut i = 0;
            while i < level.len() {
                let left = level[i];
                let right = if i + 1 < level.len() {
                    level[i + 1]
                } else {
                    left
                };
                let mut combined = [0u8; 64];
                combined[..32].copy_from_slice(&left);
                combined[32..].copy_from_slice(&right);
                let h = blake3::hash(&combined);
                next.push(*h.as_bytes());
                i += 2;
            }
            level = next;
        }
        level[0]
    }

    fn validate_receipt_schema_like(receipt: &Value) -> Result<(), String> {
        let obj = receipt
            .as_object()
            .ok_or_else(|| "receipt is not an object".to_string())?;

        let version = obj
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing version".to_string())?;
        if version != "1.0" {
            return Err(format!("unexpected version: {version}"));
        }

        let receipt_id = obj
            .get("receipt_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing receipt_id".to_string())?;
        if !receipt_id.starts_with("receipt-") || receipt_id.len() != 39 {
            return Err(format!("bad receipt_id format: {receipt_id}"));
        }
        if !receipt_id[8..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("receipt_id not hex: {receipt_id}"));
        }

        if !obj
            .get("timestamp")
            .and_then(Value::as_i64)
            .is_some()
        {
            return Err("missing or invalid timestamp".to_string());
        }

        obj.get("tenant_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing tenant_id".to_string())?;

        let receipt_type = obj
            .get("receipt_type")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing receipt_type".to_string())?;

        let blake3_hash = obj
            .get("blake3_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing blake3_hash".to_string())?;
        if blake3_hash.len() != 64
            || !blake3_hash.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err("invalid blake3_hash".to_string());
        }

        let kyber_signature = obj
            .get("kyber_signature")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing kyber_signature".to_string())?;
        if kyber_signature.is_empty() {
            return Err("empty kyber_signature".to_string());
        }

        obj.get("emitted_by")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing emitted_by".to_string())?;

        match receipt_type {
            "bore_progress" => {
                if !obj
                    .get("meters_advanced")
                    .and_then(Value::as_f64)
                    .is_some()
                {
                    return Err("missing meters_advanced".to_string());
                }
                if !obj
                    .get("cutter_head_rpm")
                    .and_then(Value::as_i64)
                    .is_some()
                {
                    return Err("missing cutter_head_rpm".to_string());
                }
            }
            "orbital_telemetry" => {
                obj.get("satellite_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "missing satellite_id".to_string())?;
                if !obj
                    .get("signal_strength_dbm")
                    .and_then(Value::as_f64)
                    .is_some()
                {
                    return Err("missing signal_strength_dbm".to_string());
                }
                if !obj
                    .get("latency_ms")
                    .and_then(Value::as_f64)
                    .is_some()
                {
                    return Err("missing latency_ms".to_string());
                }
            }
            "zk_anomaly_proof" => {
                let zk = obj
                    .get("zk_proof")
                    .and_then(Value::as_object)
                    .ok_or_else(|| "missing zk_proof".to_string())?;
                if !zk.get("pi_a").and_then(Value::as_array).is_some() {
                    return Err("missing zk_proof.pi_a".to_string());
                }
                if !zk.get("pi_b").and_then(Value::as_array).is_some() {
                    return Err("missing zk_proof.pi_b".to_string());
                }
                if !zk.get("pi_c").and_then(Value::as_array).is_some() {
                    return Err("missing zk_proof.pi_c".to_string());
                }
                if !obj
                    .get("public_inputs")
                    .and_then(Value::as_array)
                    .is_some()
                {
                    return Err("missing public_inputs".to_string());
                }
            }
            "entanglement_prediction" => {
                if !obj
                    .get("correlation_score")
                    .and_then(Value::as_f64)
                    .is_some()
                {
                    return Err("missing correlation_score".to_string());
                }
                if !obj
                    .get("predicted_negation_ms")
                    .and_then(Value::as_f64)
                    .is_some()
                {
                    return Err("missing predicted_negation_ms".to_string());
                }
            }
            _ => {}
        }

        Ok(())
    }

    #[test]
    fn test_glyph_chain_build_merkle_and_sign() {
        let bore_example = load_example("receipt_glyph.bore.example.json");
        let orbital_example = load_example("receipt_glyph.orbital.example.json");
        let zk_example = load_example("receipt_glyph.zk_anomaly.example.json");
        let ent_example = load_example("receipt_glyph.entanglement.example.json");

        assert_eq!(
            bore_example["tenant_id"],
            json!("xai-memphis-01"),
            "bore example tenant must be xai-memphis-01"
        );

        let mut chain: Vec<Value> = Vec::with_capacity(10);
        let base_ts: i64 = 1_763_000_000;

        for i in 0..6u32 {
            let mut r = bore_example.clone();
            let mut obj = r.as_object_mut().expect("object");
            obj.insert("receipt_id".to_string(), json!(make_receipt_id(i)));
            set_number(&mut obj, "timestamp", (base_ts + i as i64) as u64);
            set_f64(&mut obj, "meters_advanced", 10.0 + i as f64);
            obj.insert("emitted_by".to_string(), json!("rocket-engine"));
            drop(obj);
            let hash_hex = recompute_blake3(&mut r);
            let sig = kyber_stub_sign(&hex::decode(&hash_hex).unwrap());
            if let Some(obj_mut) = r.as_object_mut() {
                obj_mut.insert("kyber_signature".to_string(), json!(sig));
            }
            assert!(
                validate_receipt_schema_like(&r).is_ok(),
                "bore_progress receipt must validate"
            );
            chain.push(r);
        }

        {
            let mut r = orbital_example.clone();
            let mut obj = r.as_object_mut().expect("object");
            obj.insert("receipt_id".to_string(), json!(make_receipt_id(100)));
            set_number(&mut obj, "timestamp", (base_ts + 100) as u64);
            obj.insert("emitted_by".to_string(), json!("nebula-guard"));
            drop(obj);
            let hash_hex = recompute_blake3(&mut r);
            let sig = kyber_stub_sign(&hex::decode(&hash_hex).unwrap());
            if let Some(obj_mut) = r.as_object_mut() {
                obj_mut.insert("kyber_signature".to_string(), json!(sig));
            }
            assert!(
                validate_receipt_schema_like(&r).is_ok(),
                "orbital_telemetry receipt must validate"
            );
            chain.push(r);
        }

        {
            let mut r = zk_example.clone();
            let mut obj = r.as_object_mut().expect("object");
            obj.insert("receipt_id".to_string(), json!(make_receipt_id(200)));
            set_number(&mut obj, "timestamp", (base_ts + 200) as u64);
            obj.insert("emitted_by".to_string(), json!("nebula-guard"));
            drop(obj);
            let hash_hex = recompute_blake3(&mut r);
            let sig = kyber_stub_sign(&hex::decode(&hash_hex).unwrap());
            if let Some(obj_mut) = r.as_object_mut() {
                obj_mut.insert("kyber_signature".to_string(), json!(sig));
            }
            assert!(
                validate_receipt_schema_like(&r).is_ok(),
                "zk_anomaly_proof receipt must validate"
            );
            chain.push(r);
        }

        {
            let mut r = ent_example.clone();
            let mut obj = r.as_object_mut().expect("object");
            obj.insert("receipt_id".to_string(), json!(make_receipt_id(300)));
            set_number(&mut obj, "timestamp", (base_ts + 300) as u64);
            obj.insert(
                "emitted_by".to_string(),
                json!("digital-twin-groot"),
            );
            drop(obj);
            let hash_hex = recompute_blake3(&mut r);
            let sig = kyber_stub_sign(&hex::decode(&hash_hex).unwrap());
            if let Some(obj_mut) = r.as_object_mut() {
                obj_mut.insert("kyber_signature".to_string(), json!(sig));
            }
            assert!(
                validate_receipt_schema_like(&r).is_ok(),
                "entanglement_prediction receipt must validate"
            );
            chain.push(r);
        }

        assert_eq!(chain.len(), 10, "expected 10 receipts in chain");

        let mut prev_ts = None;
        for r in &chain {
            let obj = r.as_object().expect("object");
            let ts = obj
                .get("timestamp")
                .and_then(Value::as_i64)
                .expect("timestamp");
            if let Some(p) = prev_ts {
                assert!(
                    ts >= p,
                    "timestamps must be non-decreasing for chain continuity"
                );
            }
            prev_ts = Some(ts);

            let stored = obj
                .get("blake3_hash")
                .and_then(Value::as_str)
                .expect("blake3_hash");
            let mut tmp = r.clone();
            let recomputed = recompute_blake3(&mut tmp);
            assert_eq!(
                stored, recomputed,
                "blake3_hash must match recomputed canonical hash"
            );

            validate_receipt_schema_like(r)
                .unwrap_or_else(|e| panic!("schema-like validation failed: {e}"));
        }

        let leaf_hashes: Vec<[u8; 32]> = chain
            .iter()
            .map(|r| {
                let h = r["blake3_hash"].as_str().expect("hash");
                hex_to_array32(h)
            })
            .collect();

        let root1 = merkle_root(&leaf_hashes);
        let root1_hex = hex::encode(root1);

        assert_eq!(
            root1_hex.len(),
            64,
            "Merkle root must be 32-byte blake3 hex"
        );

        let kyber_sig = kyber_stub_sign(&root1);
        assert!(
            !kyber_sig.is_empty(),
            "Kyber stub signature over Merkle root must be non-empty"
        );

        let mut mutated_chain = chain.clone();
        if let Some(last) = mutated_chain.last_mut() {
            if let Some(obj) = last.as_object_mut() {
                if let Some(m) = obj.get_mut("meters_advanced") {
                    if let Some(v) = m.as_f64() {
                        *m = Value::Number(
                            Number::from_f64(v + 0.001).expect("valid f64"),
                        );
                    }
                }
            }
            let _ = recompute_blake3(last);
        }

        let mutated_leaf_hashes: Vec<[u8; 32]> = mutated_chain
            .iter()
            .map(|r| {
                let h = r["blake3_hash"].as_str().expect("hash");
                hex_to_array32(h)
            })
            .collect();

        let root2 = merkle_root(&mutated_leaf_hashes);
        let root2_hex = hex::encode(root2);

        assert_ne!(
            root1_hex, root2_hex,
            "Merkle root must change when a leaf receipt changes"
        );
    }
}
