#[cfg(test)]
mod test_spv_roundtrip {
    use blake3;
    use hex;
    use serde_json::{json, Map, Value};
    use std::fs;

    const TENANT_ID: &str = "xai-memphis-01";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Side {
        Left,
        Right,
    }

    #[derive(Clone, Debug)]
    struct ProofNode {
        sibling: [u8; 32],
        side: Side,
    }

    #[test]
    fn spv_roundtrip_submit_verify_and_prove() {
        let golden_bore =
            load_json("glyphs/examples/receipt_glyph.bore.example.json");
        let golden_orbital =
            load_json("glyphs/examples/receipt_glyph.orbital.example.json");
        let golden_zk =
            load_json("glyphs/examples/receipt_glyph.zk_anomaly.example.json");
        let golden_ent =
            load_json("glyphs/examples/receipt_glyph.entanglement.example.json");

        let base_ts: u64 = 1_767_000_000;

        // "Client → spv-api": submit target orbital receipt
        let mut receipts: Vec<Value> = Vec::new();
        let mut target = golden_orbital.clone();
        target["version"] = json!("1.0");
        target["tenant_id"] = json!(TENANT_ID);
        target["receipt_id"] = json!("receipt-spv-orbital-0000");
        target["timestamp"] = json!(base_ts);
        target["blake3_hash"] = Value::Null;
        target["kyber_signature"] = Value::Null;

        let target_hash = compute_canonical_blake3_hex(&target);
        target["blake3_hash"] = json!(target_hash.clone());
        let target_sig = derive_kyber_stub_signature(&target_hash, TENANT_ID);
        target["kyber_signature"] = json!(target_sig);
        validate_receipt_shape(&target);
        receipts.push(target.clone());

        // Additional receipts to form a small chain:
        // 1 × bore_progress, 1 × zk_anomaly_proof, 1 × entanglement_prediction
        let templates = [golden_bore, golden_zk.clone(), golden_ent];
        for (i, tmpl) in templates.into_iter().enumerate() {
            let idx = i as u32 + 1;
            let mut r = tmpl.clone();
            r["version"] = json!("1.0");
            r["tenant_id"] = json!(TENANT_ID);
            r["receipt_id"] = json!(format!("receipt-spv-chain-{idx:04x}"));
            r["timestamp"] = json!(base_ts + idx as u64);
            r["blake3_hash"] = Value::Null;
            r["kyber_signature"] = Value::Null;

            let h = compute_canonical_blake3_hex(&r);
            r["blake3_hash"] = json!(h.clone());
            let sig = derive_kyber_stub_signature(&h, TENANT_ID);
            r["kyber_signature"] = json!(sig);

            validate_receipt_shape(&r);
            receipts.push(r);
        }

        assert_eq!(receipts.len(), 4, "SPV test chain must have 4 receipts");

        // "nebula-guard": verify Kyber + ZK
        assert!(
            verify_kyber_stub(&target),
            "Kyber stub verification failed for target orbital receipt"
        );

        let zk_receipt = receipts
            .iter()
            .find(|r| r["receipt_type"] == "zk_anomaly_proof")
            .expect("chain must contain zk_anomaly_proof receipt");
        assert!(
            zk_verify_stub(zk_receipt),
            "ZK Groth16 stub verification failed for zk_anomaly_proof receipt"
        );

        // "ledger-explorer": build Merkle root + proof for target leaf index 0
        let leaf_hashes: Vec<String> = receipts
            .iter()
            .map(|r| {
                r.get("blake3_hash")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();

        assert_eq!(leaf_hashes[0], target_hash, "leaf hash must match target");
        let (merkle_root_hex, proof_path) =
            build_merkle_and_proof(&leaf_hashes, 0);

        assert!(
            !merkle_root_hex.is_empty(),
            "Merkle root must not be empty"
        );
        assert!(
            is_hex(&merkle_root_hex) && merkle_root_hex.len() == 64,
            "Merkle root must be 64 hex chars"
        );

        // "Client SPV": recompute Merkle root from leaf + proof and compare
        let recomputed_root = verify_merkle_proof(&target_hash, &proof_path);
        assert_eq!(
            merkle_root_hex, recomputed_root,
            "Merkle root mismatch in SPV verification"
        );

        let submitted_id = target["receipt_id"]
            .as_str()
            .expect("receipt_id must be string");
        assert!(
            !submitted_id.is_empty(),
            "submitted glyph_id must not be empty"
        );
    }

    fn load_json(path: &str) -> Value {
        let s = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path));
        serde_json::from_str(&s)
            .unwrap_or_else(|e| panic!("failed to parse JSON {}: {e}", path))
    }

    fn canonical_without_meta(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                let mut out = Map::new();
                for k in keys {
                    if k == "blake3_hash" || k == "kyber_signature" {
                        continue;
                    }
                    let v = map.get(k).unwrap();
                    out.insert(k.clone(), canonical_without_meta(v));
                }
                Value::Object(out)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(canonical_without_meta).collect())
            }
            _ => value.clone(),
        }
    }

    fn compute_canonical_blake3_hex(v: &Value) -> String {
        let canon = canonical_without_meta(v);
        let bytes =
            serde_json::to_vec(&canon).expect("canonical JSON serialization");
        let hash = blake3::hash(&bytes);
        hash.to_hex().to_string()
    }

    fn derive_kyber_stub_signature(hash_hex: &str, tenant_id: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(hash_hex.as_bytes());
        hasher.update(tenant_id.as_bytes());
        hasher.update(b"|kyber-1024-stub");
        let out = hasher.finalize();
        hex::encode(out.as_bytes())
    }

    fn decode_hex32(h: &str) -> [u8; 32] {
        let bytes = hex::decode(h).expect("hex decode");
        if bytes.len() != 32 {
            panic!("expected 32-byte hash, got {}", bytes.len());
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        arr
    }

    fn build_merkle_and_proof(
        leaf_hashes: &[String],
        target_index: usize,
    ) -> (String, Vec<ProofNode>) {
        assert!(!leaf_hashes.is_empty(), "leaf hashes must not be empty");
        assert!(
            target_index < leaf_hashes.len(),
            "target index out of bounds"
        );

        let mut level: Vec<[u8; 32]> =
            leaf_hashes.iter().map(|h| decode_hex32(h)).collect();
        let mut idx = target_index;
        let mut path: Vec<ProofNode> = Vec::new();

        while level.len() > 1 {
            let mut next: Vec<[u8; 32]> = Vec::new();
            let mut i = 0usize;
            let next_idx = idx / 2;

            while i < level.len() {
                let left = level[i];
                let right = if i + 1 < level.len() {
                    level[i + 1]
                } else {
                    left
                };

                if i == idx || i + 1 == idx {
                    if idx % 2 == 0 {
                        let sibling = if i + 1 < level.len() {
                            right
                        } else {
                            left
                        };
                        path.push(ProofNode {
                            sibling,
                            side: Side::Right,
                        });
                    } else {
                        let sibling = left;
                        path.push(ProofNode {
                            sibling,
                            side: Side::Left,
                        });
                    }
                }

                let mut hasher = blake3::Hasher::new();
                hasher.update(&left);
                hasher.update(&right);
                let out = hasher.finalize();
                let mut parent = [0u8; 32];
                parent.copy_from_slice(out.as_bytes());
                next.push(parent);

                i += 2;
            }

            level = next;
            idx = next_idx;
        }

        let root = level[0];
        (hex::encode(root), path)
    }

    fn verify_merkle_proof(leaf_hash_hex: &str, path: &[ProofNode]) -> String {
        let mut acc = decode_hex32(leaf_hash_hex);
        for node in path {
            let (left, right) = match node.side {
                Side::Left => (node.sibling, acc),
                Side::Right => (acc, node.sibling),
            };
            let mut hasher = blake3::Hasher::new();
            hasher.update(&left);
            hasher.update(&right);
            let out = hasher.finalize();
            acc = {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(out.as_bytes());
                arr
            };
        }
        hex::encode(acc)
    }

    fn validate_receipt_shape(r: &Value) {
        assert_eq!(
            r.get("version").and_then(Value::as_str),
            Some("1.0"),
            "version must be 1.0"
        );

        let rid = r
            .get("receipt_id")
            .and_then(Value::as_str)
            .expect("receipt_id must be string");
        assert!(
            rid.starts_with("receipt-"),
            "receipt_id must start with 'receipt-'"
        );

        let ts = r
            .get("timestamp")
            .and_then(Value::as_u64)
            .expect("timestamp must be u64");
        assert!(ts > 0, "timestamp must be > 0");

        let tenant = r
            .get("tenant_id")
            .and_then(Value::as_str)
            .expect("tenant_id must be string");
        assert!(
            !tenant.is_empty(),
            "tenant_id must not be empty"
        );

        let rtype = r
            .get("receipt_type")
            .and_then(Value::as_str)
            .expect("receipt_type must be string");
        match rtype {
            "bore_progress"
            | "orbital_telemetry"
            | "zk_anomaly_proof"
            | "entanglement_prediction"
            | "anomaly_detected"
            | "phase_transition"
            | "swarm_vote"
            | "compaction_complete"
            | "voice_page_sent" => {}
            other => panic!("unexpected receipt_type: {other}"),
        }

        let emitted_by = r
            .get("emitted_by")
            .and_then(Value::as_str)
            .expect("emitted_by must be string");
        match emitted_by {
            "rocket-engine"
            | "nebula-guard"
            | "digital-twin-groot"
            | "drax-metrics"
            | "mantis-community"
            | "star-lord-orchestrator"
            | "groot-swarm"
            | "ledger-explorer" => {}
            other => panic!("unexpected emitted_by: {other}"),
        }

        let hash = r
            .get("blake3_hash")
            .and_then(Value::as_str)
            .expect("blake3_hash must be string");
        assert_eq!(
            hash.len(),
            64,
            "blake3_hash must be 64 hex chars"
        );
        assert!(is_hex(hash), "blake3_hash must be hex");

        let sig = r
            .get("kyber_signature")
            .and_then(Value::as_str)
            .expect("kyber_signature must be string");
        assert!(
            sig.len() >= 64,
            "kyber_signature must be at least 64 hex chars"
        );
        assert!(is_hex(sig), "kyber_signature must be hex");

        match rtype {
            "bore_progress" => {
                assert!(r.get("meters_advanced").is_some());
                assert!(r.get("cutter_head_rpm").is_some());
            }
            "orbital_telemetry" => {
                assert!(r.get("satellite_id").is_some());
                assert!(r.get("signal_strength_dbm").is_some());
                assert!(r.get("latency_ms").is_some());
            }
            "zk_anomaly_proof" => {
                let zk_proof = r
                    .get("zk_proof")
                    .expect("zk_anomaly_proof must have zk_proof");
                assert!(zk_proof.get("pi_a").is_some());
                assert!(zk_proof.get("pi_b").is_some());
                assert!(zk_proof.get("pi_c").is_some());
                assert!(r.get("public_inputs").is_some());
            }
            "entanglement_prediction" => {
                let corr = r
                    .get("correlation_score")
                    .and_then(Value::as_f64)
                    .expect("entanglement_prediction must have correlation_score");
                assert!(
                    corr >= 0.707,
                    "correlation_score must be >= 0.707, got {corr}"
                );
                let neg = r
                    .get("predicted_negation_ms")
                    .and_then(Value::as_f64)
                    .expect("entanglement_prediction must have predicted_negation_ms");
                assert!(
                    neg >= 0.0,
                    "predicted_negation_ms must be >= 0, got {neg}"
                );
            }
            _ => {}
        }
    }

    fn verify_kyber_stub(r: &Value) -> bool {
        let hash = match r.get("blake3_hash").and_then(Value::as_str) {
            Some(h) => h,
            None => return false,
        };
        let tenant = match r.get("tenant_id").and_then(Value::as_str) {
            Some(t) => t,
            None => return false,
        };
        let actual = match r.get("kyber_signature").and_then(Value::as_str) {
            Some(s) => s,
            None => return false,
        };
        let expected = derive_kyber_stub_signature(hash, tenant);
        actual == expected
    }

    fn zk_verify_stub(r: &Value) -> bool {
        if r
            .get("receipt_type")
            .and_then(Value::as_str)
            != Some("zk_anomaly_proof")
        {
            return false;
        }
        let zk = match r.get("zk_proof") {
            Some(v) => v,
            None => return false,
        };
        let obj = match zk.as_object() {
            Some(o) => o,
            None => return false,
        };
        let pi_a_ok = obj
            .get("pi_a")
            .and_then(Value::as_array)
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        let pi_b_ok = obj
            .get("pi_b")
            .and_then(Value::as_array)
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        let pi_c_ok = obj
            .get("pi_c")
            .and_then(Value::as_array)
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        let inputs_ok = r
            .get("public_inputs")
            .and_then(Value::as_array)
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        let hint_ok = r
            .get("anomaly_hint")
            .and_then(Value::as_str)
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        pi_a_ok && pi_b_ok && pi_c_ok && inputs_ok && hint_ok
    }

    fn is_hex(s: &str) -> bool {
        s.chars()
            .all(|c| matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F'))
    }
}
