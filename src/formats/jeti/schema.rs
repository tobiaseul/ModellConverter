/// Jeti Duplex model schema.
///
/// The `.jsn` format is JSON-based but not fully publicly documented.
/// Fields are added incrementally as sample files are analyzed.
///
/// Unknown/unrecognized JSON fields are preserved via the `extra` catch-all
/// so round-trips don't silently drop data.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct JetiModel {
    /// Model name as stored in the Jeti transmitter.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,

    // Additional fields are added here as .jsn files are analyzed.
    // Example (not yet confirmed):
    // #[serde(default)]
    // pub channels: Vec<JetiChannel>,

    /// Catch-all for unrecognized fields — preserves unknown data on round-trip.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_json() {
        let json = r#"{"name": "TestModel"}"#;
        let model: JetiModel = serde_json::from_str(json).unwrap();
        assert_eq!(model.name, "TestModel");
        assert!(model.extra.is_empty());
    }

    #[test]
    fn unknown_fields_preserved() {
        let json = r#"{"name": "M", "someUnknownField": 42, "nested": {"a": 1}}"#;
        let model: JetiModel = serde_json::from_str(json).unwrap();
        assert_eq!(model.name, "M");
        assert!(model.extra.contains_key("someUnknownField"));
        assert!(model.extra.contains_key("nested"));
    }

    #[test]
    fn roundtrip_preserves_unknown_fields() {
        let json = r#"{"name":"M","unknownKey":99}"#;
        let model: JetiModel = serde_json::from_str(json).unwrap();
        let out = serde_json::to_string(&model).unwrap();
        let model2: JetiModel = serde_json::from_str(&out).unwrap();
        assert_eq!(model2.extra.get("unknownKey"), model.extra.get("unknownKey"));
    }
}
