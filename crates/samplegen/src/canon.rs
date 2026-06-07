//! Deterministic canonical-JSON + SHA-256 helpers shared across the crate.
//!
//! Mirrors the repo's `canonical_json_sha256` convention: object keys are
//! sorted and all insignificant whitespace is removed before hashing, so the
//! same logical record always produces the same digest regardless of field
//! insertion order. Used by the accountability seal and the signed patrol
//! trajectory so both sign over identical byte sequences.

use serde_json::Value;
use sha2::{Digest, Sha256};

/// Hex-encoded SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Deterministic canonical JSON: object keys sorted, no insignificant space.
pub fn canonicalize(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let inner: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    let v = map.get(k).map_or_else(|| "null".to_string(), canonicalize);
                    let key = serde_json::to_string(k).unwrap_or_else(|_| "\"\"".to_string());
                    format!("{key}:{v}")
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        Value::Array(items) => {
            let inner: Vec<String> = items.iter().map(canonicalize).collect();
            format!("[{}]", inner.join(","))
        }
        other => serde_json::to_string(other).unwrap_or_else(|_| "null".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_is_key_order_independent() {
        let a = json!({ "b": 1, "a": 2, "c": [3, 2, 1] });
        let b = json!({ "c": [3, 2, 1], "a": 2, "b": 1 });
        assert_eq!(canonicalize(&a), canonicalize(&b));
        assert_eq!(canonicalize(&a), r#"{"a":2,"b":1,"c":[3,2,1]}"#);
    }

    #[test]
    fn sha256_is_stable_and_64_hex() {
        let h = sha256_hex(b"openagents");
        assert_eq!(h.len(), 64);
        assert_eq!(h, sha256_hex(b"openagents"));
    }
}
