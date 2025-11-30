#[cfg(test)]
mod test_digital_twin {
    use blake3::{hash, Hasher};
    use serde_json::{json, Map, Value};
    use std::collections::BTreeMap;

    const TENANT_ID: &str = "xai-memphis-01";
    const MAX_DIVERGENCE: f64 = 0.05; // 5%

    #[derive(Clone, Debug)]
    struct AssetState {
        asset_id: String,
        epoch: u64,
        position: f64,
        health: f64,
    }

    impl AssetState {
        fn new(asset_id: &str, epoch: u64, position: f64, health: f64) -> Self {
            Self {
                asset_id: asset_id.to_string(),
                epoch,
                position,
                health,
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum Side {
        Left,
        Right,
    }

    #[derive(Clone, Debug)]
    struct ProofNode {
        side: Side,
        hash_hex: String,
    }

    fn relative_divergence(a: f64, b: f64) -> f64 {
        if a == 0.0 && b == 0.0 {
            0.0
        } else if a == 0.0 {
            1.0
        } else {
            ((a - b).abs() / a.abs()).min(1.0)
        }
    }

    fn hash_state(state: &AssetState) -> String {
        let canonical = format!(
            "asset_id={}|epoch={}|position={:.6}|health={:.6}",
            state.asset_id, state.epoch, state.position, state.health
        );
        hash(canonical.as_bytes()).to_hex().to_string()
    }

    fn from_hex_digit(d: u8) -> u8 {
        match d {
            b'0'..=b'9' => d - b'0',
            b'a'..=b'f' => 10 + (d - b'a'),
            b'A'..=b'F' => 10 + (d - b'A'),
            _ => panic!("invalid hex digit: {}", d as char),
        }
    }

    fn hex_to_bytes32(hex: &str) -> [u8; 32] {
        assert_eq!(
            hex.len(),
            64,
            "expected 64 hex chars to convert to 32 bytes, got {}",
            hex.len()
        );
        let mut out = [0u8; 32];
        let bytes = hex.as_bytes();
        let mut i = 0usize;
        while i < 32 {
            let hi = from_hex_digit(bytes[2 * i]) as u8;
            let lo = from_hex_digit(bytes[2 * i + 1]) as u8;
            out[i] = (hi << 4) | lo;
            i += 1;
        }
        out
    }

    fn bytes32_to_hex(bytes: &[u8; 32]) -> String {
        let mut s = String::with_capacity(64);
        for b in bytes.iter() {
            use std::fmt::Write;
            write!(&mut s, "{:02x}", b).expect("write hex");
        }
        s
    }

    fn merkle_root(hashes: &[String]) -> String {
        assert!(!hashes.is_empty(), "cannot build Merkle tree over empty set");
        let mut level: Vec<[u8; 32]> = hashes.iter().map(|h| hex_to_bytes32(h)).collect();
        while level.len() > 1 {
            let mut next: Vec<[u8; 32]> = Vec::with_capacity((level.len() + 1) / 2);
            let mut i = 0usize;
            while i < level.len() {
                let left = level[i];
                let right = if i + 1 < level.len() {
                    level[i + 1]
                } else {
                    left
                };
                let mut hasher = Hasher::new();
                hasher.update(&left);
                hasher.update(&right);
                let mut parent = [0u8; 32];
                parent.copy_from_slice(hasher.finalize().as_bytes());
                next.push(parent);
                i += 2;
            }
            level = next;
        }
        bytes32_to_hex(&level[0])
    }

    fn build_merkle_root_and_proof(
        hashes: &[String],
        target_index: usize,
    ) -> (String, Vec<ProofNode>) {
        assert!(!hashes.is_empty(), "cannot build Merkle tree over empty set");
        assert!(
            target_index < hashes.len(),
            "target index out of bounds for Merkle tree"
        );

        let mut level: Vec<[u8; 32]> = hashes.iter().map(|h| hex_to_bytes32(h)).collect();
        let mut index = target_index;
        let mut proof: Vec<ProofNode> = Vec::new();

        while level.len() > 1 {
            let mut next: Vec<[u8; 32]> = Vec::with_capacity((level.len() + 1) / 2);
            let mut i = 0usize;

            while i < level.len() {
                let left = level[i];
                let (right, right_index) = if i + 1 < level.len() {
                    (level[i + 1], i + 1)
                } else {
                    (left, i)
                };

                let mut hasher = Hasher::new();
                hasher.update(&left);
                hasher.update(&right);
                let mut parent = [0u8; 32];
                parent.copy_from_slice(hasher.finalize().as_bytes());
                next.push(parent);

                if index == i || index == right_index {
                    if index == i {
                        proof.push(ProofNode {
                            side: Side::Right,
                            hash_hex: bytes32_to_hex(&right),
                        });
                    } else {
                        proof.push(ProofNode {
                            side: Side::Left,
                            hash_hex: bytes32_to_hex(&left),
                        });
                    }
                    index = next.len() - 1;
                }

                i += 2;
            }

            level = next;
        }

        (bytes32_to_hex(&level[0]), proof)
    }

    fn recompute_root_from_leaf_and_proof(
        leaf_hash_hex: &str,
        proof: &[ProofNode],
    ) -> String {
        let mut acc = hex_to_bytes32(leaf_hash_hex);
        for node in proof {
            let sibling = hex_to_bytes32(&node.hash_hex);
            let mut hasher = Hasher::new();
            match node.side {
                Side::Left => {
                    hasher.update(&sibling);
                    hasher.update(&acc);
                }
                Side::Right => {
                    hasher.update(&acc);
                    hasher.update(&sibling);
                }
            }
            let mut out = [0u8; 32];
            out.copy_from_slice(hasher.finalize().as_bytes());
            acc = out;
        }
        bytes32_to_hex(&acc)
    }

    fn canonical_without_crypto(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut sorted = BTreeMap::<String, Value>::new();
                for (k, v) in map.iter() {
                    if k == "blake3_hash" || k == "kyber_signature" || k == "receipt_id" {
                        continue;
                    }
                    sorted.insert(k.clone(), canonical_without_crypto(v));
                }
                let mut out = Map::new();
                for (k, v) in sorted {
                    out.insert(k, v);
                }
                Value::Object(out)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(canonical_without_crypto).collect())
            }
            _ => value.clone(),
        }
    }

    fn compute_receipt_blake3(value: &Value) -> String {
        let canonical = canonical_without_crypto(value);
        let bytes =
            serde_json::to_vec(&canonical).expect("canonical JSON serialization failed");
        hash(&bytes).to_hex().to_string()
    }

    fn emit_twin_receipt(
        asset_id: &str,
        root_real: &str,
        root_twin: &str,
        divergence_percent: f64,
        receipt_type: &str,
    ) -> Value {
        use std::time::{SystemTime, UNIX_EPOCH};

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_secs() as i64;

        let mut obj = Map::new();
        obj.insert("version".to_string(), json!("1.0"));
        obj.insert("timestamp".to_string(), json!(ts));
        obj.insert("tenant_id".to_string(), json!(TENANT_ID));
        obj.insert("receipt_type".to_string(), json!(receipt_type));
        obj.insert("emitted_by".to_string(), json!("digital-twin-groot"));
        obj.insert("asset_id".to_string(), json!(asset_id));
        obj.insert("root_real".to_string(), json!(root_real));
        obj.insert("root_twin".to_string(), json!(root_twin));
        obj.insert(
            "divergence_percent".to_string(),
            json!(divergence_percent),
        );

        let mut v = Value::Object(obj);
        let hash_hex = compute_receipt_blake3(&v);

        if let Value::Object(ref mut m) = v {
            let receipt_id = format!("receipt-{}", &hash_hex[..32]);
            let kyber = format!("kyber-{}", &hash_hex[..32]);
            m.insert("receipt_id".to_string(), json!(receipt_id));
            m.insert("blake3_hash".to_string(), json!(hash_hex));
            m.insert("kyber_signature".to_string(), json!(kyber));
        }

        v
    }

    fn assert_receipt_basic(receipt: &Value, expected_type: &str) {
        let obj = receipt.as_object().expect("receipt must be object");
        assert_eq!(
            obj.get("version").and_then(Value::as_str),
            Some("1.0")
        );
        assert_eq!(
            obj.get("tenant_id").and_then(Value::as_str),
            Some(TENANT_ID)
        );
        assert_eq!(
            obj.get("receipt_type").and_then(Value::as_str),
            Some(expected_type)
        );
        assert_eq!(
            obj.get("emitted_by").and_then(Value::as_str),
            Some("digital-twin-groot")
        );
        let rid = obj
            .get("receipt_id")
            .and_then(Value::as_str)
            .expect("receipt_id must be present");
        assert!(
            rid.starts_with("receipt-"),
            "receipt_id must start with receipt-"
        );
        let hash_hex = obj
            .get("blake3_hash")
            .and_then(Value::as_str)
            .expect("blake3_hash must be present");
        assert_eq!(hash_hex.len(), 64);
        let sig = obj
            .get("kyber_signature")
            .and_then(Value::as_str)
            .expect("kyber_signature must be present");
        assert!(
            sig.starts_with("kyber-"),
            "kyber_signature must start with kyber-"
        );
        assert!(
            obj.get("asset_id").and_then(Value::as_str).is_some(),
            "asset_id must be present"
        );
        assert!(
            obj.get("root_real").and_then(Value::as_str).is_some(),
            "root_real must be present"
        );
        assert!(
            obj.get("root_twin").and_then(Value::as_str).is_some(),
            "root_twin must be present"
        );
        assert!(
            obj.get("divergence_percent")
                .and_then(Value::as_f64)
                .is_some(),
            "divergence_percent must be present"
        );
    }

    #[test]
    fn baseline_twin_sync() {
        let asset_id = "tesla-vin-5YJ3E1EA7KF317000";

        let real = vec![
            AssetState::new(asset_id, 0, 0.0, 1.00),
            AssetState::new(asset_id, 1, 12.5, 0.99),
            AssetState::new(asset_id, 2, 25.0, 0.98),
        ];

        let twin = vec![
            AssetState::new(asset_id, 0, 0.0, 1.00),
            AssetState::new(asset_id, 1, 12.6, 0.985),
            AssetState::new(asset_id, 2, 24.9, 0.975),
        ];

        assert_eq!(real.len(), twin.len());

        let mut worst_divergence = 0.0;
        for (r, t) in real.iter().zip(twin.iter()) {
            let pos_div = relative_divergence(r.position, t.position);
            let health_div = relative_divergence(r.health, t.health);
            worst_divergence = worst_divergence
                .max(pos_div)
                .max(health_div);
        }

        assert!(
            worst_divergence <= MAX_DIVERGENCE,
            "baseline twin divergence {:.4} exceeds 5%",
            worst_divergence
        );

        let real_hashes: Vec<String> = real.iter().map(hash_state).collect();
        let twin_hashes: Vec<String> = twin.iter().map(hash_state).collect();

        let real_root = merkle_root(&real_hashes);
        let twin_root = merkle_root(&twin_hashes);

        assert_eq!(
            real_root, twin_root,
            "baseline twin Merkle root must match real"
        );

        let receipt = emit_twin_receipt(
            asset_id,
            &real_root,
            &twin_root,
            worst_divergence * 100.0,
            "entanglement_prediction",
        );
        assert_receipt_basic(&receipt, "entanglement_prediction");
    }

    #[test]
    fn fork_anomaly_detects_divergence_and_root_mismatch() {
        let asset_id = "tesla-vin-5YJ3E1EA7KF317000";

        let real = vec![
            AssetState::new(asset_id, 0, 0.0, 1.00),
            AssetState::new(asset_id, 1, 15.0, 0.99),
            AssetState::new(asset_id, 2, 30.0, 0.98),
        ];

        let mut twin = real.clone();
        twin[1].position = 18.0;
        twin[1].health = 0.90;

        let mut worst_divergence = 0.0;
        for (r, t) in real.iter().zip(twin.iter()) {
            let pos_div = relative_divergence(r.position, t.position);
            let health_div = relative_divergence(r.health, t.health);
            worst_divergence = worst_divergence
                .max(pos_div)
                .max(health_div);
        }

        assert!(
            worst_divergence > MAX_DIVERGENCE,
            "fork anomaly must exceed 5% divergence"
        );

        let real_hashes: Vec<String> = real.iter().map(hash_state).collect();
        let twin_hashes: Vec<String> = twin.iter().map(hash_state).collect();

        let real_root = merkle_root(&real_hashes);
        let twin_root = merkle_root(&twin_hashes);

        assert_ne!(
            real_root, twin_root,
            "fork anomaly must change Merkle root"
        );

        let receipt = emit_twin_receipt(
            asset_id,
            &real_root,
            &twin_root,
            worst_divergence * 100.0,
            "anomaly_detected",
        );
        assert_receipt_basic(&receipt, "anomaly_detected");

        let div = receipt["divergence_percent"]
            .as_f64()
            .expect("divergence_percent must be f64");
        assert!(
            div > 5.0,
            "anomaly receipt must show divergence > 5%, got {}",
            div
        );
    }

    #[test]
    fn duress_state_recovery_closes_divergence() {
        let asset_id = "tesla-vin-5YJ3E1EA7KF317000";

        let real = vec![
            AssetState::new(asset_id, 0, 0.0, 1.00),
            AssetState::new(asset_id, 1, 20.0, 0.99),
            AssetState::new(asset_id, 2, 40.0, 0.98),
        ];

        let mut forked = real.clone();
        forked[2].position = 50.0;
        forked[2].health = 0.88;

        let mut worst_div_fork = 0.0;
        for (r, f) in real.iter().zip(forked.iter()) {
            let pos_div = relative_divergence(r.position, f.position);
            let health_div = relative_divergence(r.health, f.health);
            worst_div_fork = worst_div_fork
                .max(pos_div)
                .max(health_div);
        }
        assert!(
            worst_div_fork > MAX_DIVERGENCE,
            "duress fork must exceed 5% divergence"
        );

        let recovered = real.clone();

        let mut worst_div_recovered = 0.0;
        for (r, t) in real.iter().zip(recovered.iter()) {
            let pos_div = relative_divergence(r.position, t.position);
            let health_div = relative_divergence(r.health, t.health);
            worst_div_recovered = worst_div_recovered
                .max(pos_div)
                .max(health_div);
        }
        assert!(
            worst_div_recovered <= MAX_DIVERGENCE,
            "recovered twin divergence {:.4} must be ≤ 5%",
            worst_div_recovered
        );

        let real_hashes: Vec<String> = real.iter().map(hash_state).collect();
        let fork_hashes: Vec<String> = forked.iter().map(hash_state).collect();
        let recovered_hashes: Vec<String> =
            recovered.iter().map(hash_state).collect();

        let real_root = merkle_root(&real_hashes);
        let fork_root = merkle_root(&fork_hashes);
        let recovered_root = merkle_root(&recovered_hashes);

        assert_ne!(real_root, fork_root, "fork root must differ");
        assert_eq!(
            real_root, recovered_root,
            "recovered root must match real"
        );

        let anomaly_receipt = emit_twin_receipt(
            asset_id,
            &real_root,
            &fork_root,
            worst_div_fork * 100.0,
            "anomaly_detected",
        );
        assert_receipt_basic(&anomaly_receipt, "anomaly_detected");

        let recovery_receipt = emit_twin_receipt(
            asset_id,
            &real_root,
            &recovered_root,
            worst_div_recovered * 100.0,
            "phase_transition",
        );
        assert_receipt_basic(&recovery_receipt, "phase_transition");

        let div_recovered = recovery_receipt["divergence_percent"]
            .as_f64()
            .expect("divergence_percent must be f64");
        assert!(
            div_recovered <= 5.0,
            "recovery receipt must show divergence ≤ 5%, got {}",
            div_recovered
        );
    }

    #[test]
    fn orbital_replication_matches_roots() {
        let asset_id = "starship-pad-a";

        let real = vec![
            AssetState::new(asset_id, 0, 1.0, 0.99),
            AssetState::new(asset_id, 1, 1.0, 0.99),
            AssetState::new(asset_id, 2, 1.0, 0.99),
            AssetState::new(asset_id, 3, 1.0, 0.99),
        ];

        let twin = vec![
            AssetState::new(asset_id, 0, 1.0, 0.99),
            AssetState::new(asset_id, 1, 1.0, 0.985),
            AssetState::new(asset_id, 2, 1.0, 0.98),
            AssetState::new(asset_id, 3, 1.0, 0.98),
        ];

        let mut worst_divergence = 0.0;
        for (r, t) in real.iter().zip(twin.iter()) {
            let health_div = relative_divergence(r.health, t.health);
            worst_divergence = worst_divergence.max(health_div);
        }
        assert!(
            worst_divergence <= MAX_DIVERGENCE,
            "orbital replication divergence {:.4} must be ≤ 5%",
            worst_divergence
        );

        let real_hashes: Vec<String> = real.iter().map(hash_state).collect();
        let twin_hashes: Vec<String> = twin.iter().map(hash_state).collect();

        let real_root = merkle_root(&real_hashes);
        let twin_root = merkle_root(&twin_hashes);
        assert_eq!(
            real_root, twin_root,
            "orbital replication twin root must match real"
        );

        let (root_with_proof, proof) =
            build_merkle_root_and_proof(&real_hashes, 0);
        assert_eq!(root_with_proof, real_root);
        assert!(
            !proof.is_empty(),
            "orbital replication proof path must not be empty"
        );

        let recomputed = recompute_root_from_leaf_and_proof(&real_hashes[0], &proof);
        assert_eq!(
            real_root, recomputed,
            "SPV-style proof must verify for orbital replication"
        );

        let receipt = emit_twin_receipt(
            asset_id,
            &real_root,
            &twin_root,
            worst_divergence * 100.0,
            "orbital_telemetry",
        );
        assert_receipt_basic(&receipt, "orbital_telemetry");
    }

    #[test]
    fn red_loop_closure_emits_stable_receipt() {
        let asset_id = "tesla-fleet-memphis";

        let real = vec![
            AssetState::new(asset_id, 0, 1000.0, 0.99),
            AssetState::new(asset_id, 1, 1005.0, 0.99),
            AssetState::new(asset_id, 2, 1010.0, 0.985),
            AssetState::new(asset_id, 3, 1015.0, 0.985),
            AssetState::new(asset_id, 4, 1020.0, 0.98),
            AssetState::new(asset_id, 5, 1025.0, 0.98),
            AssetState::new(asset_id, 6, 1030.0, 0.98),
        ];

        let twin = vec![
            AssetState::new(asset_id, 0, 1000.0, 0.99),
            AssetState::new(asset_id, 1, 1004.5, 0.989),
            AssetState::new(asset_id, 2, 1010.5, 0.983),
            AssetState::new(asset_id, 3, 1016.0, 0.982),
            AssetState::new(asset_id, 4, 1021.0, 0.979),
            AssetState::new(asset_id, 5, 1025.5, 0.978),
            AssetState::new(asset_id, 6, 1030.0, 0.978),
        ];

        assert_eq!(real.len(), twin.len());

        let mut worst_divergence = 0.0;
        let mut sum_divergence = 0.0;
        for (r, t) in real.iter().zip(twin.iter()) {
            let pos_div = relative_divergence(r.position, t.position);
            let health_div = relative_divergence(r.health, t.health);
            let d = pos_div.max(health_div);
            worst_divergence = worst_divergence.max(d);
            sum_divergence += d;
        }
        let avg_divergence = sum_divergence / real.len() as f64;

        assert!(
            worst_divergence <= MAX_DIVERGENCE,
            "red-loop worst divergence {:.4} must be ≤ 5%",
            worst_divergence
        );
        assert!(
            avg_divergence <= 0.03,
            "red-loop average divergence {:.4} must be ≤ 3%",
            avg_divergence
        );

        let real_hashes: Vec<String> = real.iter().map(hash_state).collect();
        let twin_hashes: Vec<String> = twin.iter().map(hash_state).collect();

        let real_root = merkle_root(&real_hashes);
        let twin_root = merkle_root(&twin_hashes);
        assert_eq!(
            real_root, twin_root,
            "red-loop final twin root must match real"
        );

        let red_loop_root = {
            let combined = format!("red-loop-root:{}", real_root);
            hash(combined.as_bytes()).to_hex().to_string()
        };

        assert_eq!(red_loop_root.len(), 64);

        let receipt = emit_twin_receipt(
            asset_id,
            &real_root,
            &twin_root,
            worst_divergence * 100.0,
            "compaction_complete",
        );
        assert_receipt_basic(&receipt, "compaction_complete");

        let div = receipt["divergence_percent"]
            .as_f64()
            .expect("divergence_percent must be f64");
        assert!(
            div <= 5.0,
            "red-loop closure receipt must report divergence ≤ 5%, got {}",
            div
        );
    }
}
