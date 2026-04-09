/// Jeti Duplex model schema (`.jsn` format).
///
/// Each top-level section is typed. Unknown fields within a section
/// and unknown top-level sections are preserved via `extra` catch-alls
/// so round-trips don't silently drop data.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// ── top-level model ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct JetiModel {
    #[serde(rename = "Global")]
    pub global: JetiGlobal,

    #[serde(rename = "Servos", default)]
    pub servos: Option<JetiSection<JetiServo>>,

    #[serde(rename = "Functions", default)]
    pub functions: Option<JetiSection<JetiFunction>>,

    /// Each entry is a raw array: [servo_idx, func_idx, sub, ...]
    #[serde(rename = "Mixes-Main", default)]
    pub mixes_main: Option<JetiSection<Vec<Value>>>,

    #[serde(rename = "Mixes-Values", default)]
    pub mixes_values: Option<Vec<JetiMixValue>>,

    /// User-defined flight modes with switch assignments.
    #[serde(rename = "Flight-Modes", default)]
    pub flight_modes: Option<JetiSection<JetiFlightMode>>,

    /// Per-flight-mode expo/DR settings for each function (axis).
    /// Bare JSON array (not wrapped in {Type, Data}).
    #[serde(rename = "Function-Specs", default)]
    pub function_specs: Option<Vec<JetiFunctionSpec>>,

    #[serde(rename = "Timers", default)]
    pub timers: Option<JetiSection<JetiTimer>>,

    #[serde(rename = "Telem-Detect", default)]
    pub telem_detect: Option<JetiSection<JetiTelemSensor>>,

    /// All other sections are preserved unchanged.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

// ── generic section container ────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct JetiSection<T> {
    #[serde(rename = "Type")]
    pub type_name: String,
    #[serde(rename = "Data")]
    pub data: Vec<T>,
}

// ── Global ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct JetiGlobal {
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "Version", default)]
    pub version: u32,
    #[serde(rename = "TxVers", default)]
    pub tx_vers: String,
    #[serde(rename = "Model-Type", default)]
    pub model_type: u32,
    #[serde(rename = "Filename", default)]
    pub filename: String,
    #[serde(rename = "Desc", default)]
    pub desc: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

// ── Servos ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct JetiServo {
    #[serde(rename = "Index")]
    pub index: u8,
    /// Center offset in percent (±100 scale; 100 ≈ 500 µs)
    #[serde(rename = "Middle", default)]
    pub middle: i32,
    #[serde(rename = "Max-Positive", default)]
    pub max_positive: i32,
    #[serde(rename = "Max-Negative", default)]
    pub max_negative: i32,
    #[serde(rename = "Servo-Reverse", default)]
    pub reversed: u8,
    #[serde(rename = "Delay-Positive", default)]
    pub delay_positive: u32,
    #[serde(rename = "Delay-Negative", default)]
    pub delay_negative: u32,
    #[serde(rename = "Curve", default)]
    pub curve: Vec<i32>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

// ── Functions ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct JetiFunction {
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Label", default)]
    pub label: String,
    /// Jeti control string: "axis,sub,c1,c2,c3,p1,p2,p3"
    #[serde(rename = "Control", default)]
    pub control: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

// ── Mixes-Values ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct JetiMixValue {
    #[serde(rename = "Flight-Mode", default)]
    pub flight_mode: u32,
    #[serde(rename = "Intensity", default)]
    pub intensity: i32,
    /// 1 = normal, -1 = reversed
    #[serde(rename = "Direction", default = "default_direction")]
    pub direction: i32,
    #[serde(rename = "Switch", default)]
    pub switch: String,
    #[serde(rename = "Curve-Type", default)]
    pub curve_type: u32,
    #[serde(rename = "Points-In", default)]
    pub points_in: Vec<i32>,
    #[serde(rename = "Points-Out", default)]
    pub points_out: Vec<i32>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_direction() -> i32 { 1 }

// ── Flight-Modes ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct JetiFlightMode {
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Label", default)]
    pub label: String,
    /// Jeti control string for the activating switch.
    #[serde(rename = "Switch", default)]
    pub switch: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

// ── Function-Specs ────────────────────────────────────────────────────────────

/// Per-flight-mode expo and dual-rate settings for one axis function.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct JetiFunctionSpec {
    /// 0-based index into the flight-modes array.
    #[serde(rename = "Flight-Mode", default)]
    pub flight_mode: u32,
    /// Matches JetiFunction.id.
    #[serde(rename = "Function-Id", default)]
    pub function_id: u32,
    /// [expo%, reserved, reserved] — only first element used.
    #[serde(rename = "Expo-Pos", default)]
    pub expo_pos: Vec<i32>,
    #[serde(rename = "Expo-Neg", default)]
    pub expo_neg: Vec<i32>,
    /// [dr1%, dr2%, dr3%] — dr1 is the primary rate.
    #[serde(rename = "DR-Pos", default)]
    pub dr_pos: Vec<i32>,
    #[serde(rename = "DR-Neg", default)]
    pub dr_neg: Vec<i32>,
    /// 1 = symmetric expo/DR (use Pos values for both directions).
    #[serde(rename = "Sym", default)]
    pub sym: u32,
    #[serde(rename = "Curve-Type", default)]
    pub curve_type: u32,
    #[serde(rename = "Points-In", default)]
    pub points_in: Vec<i32>,
    #[serde(rename = "Points-Out", default)]
    pub points_out: Vec<i32>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

// ── Timers ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct JetiTimer {
    #[serde(rename = "ID", default)]
    pub id: u32,
    #[serde(rename = "Label", default)]
    pub label: String,
    /// 0 = up, 1 = down (countdown)
    #[serde(rename = "Mode", default)]
    pub mode: u32,
    /// Start value in seconds
    #[serde(rename = "Start", default)]
    pub start: u32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

// ── Telem-Detect ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct JetiTelemSensor {
    #[serde(rename = "Label", default)]
    pub label: String,
    #[serde(rename = "Unit", default)]
    pub unit: String,
    /// Number of decimal places (0 = integer, 2 = hundredths, etc.)
    #[serde(rename = "Decimal", default)]
    pub decimal: u32,
    #[serde(rename = "Saving", default)]
    pub saving: u32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_global_section() {
        let json = r#"{"Global":{"Name":"TestModel","Version":680,"TxVers":"5.08","Model-Type":1,"Filename":"test.jsn","Desc":""}}"#;
        let model: JetiModel = serde_json::from_str(json).unwrap();
        assert_eq!(model.global.name, "TestModel");
        assert_eq!(model.global.version, 680);
    }

    #[test]
    fn unknown_sections_preserved() {
        let json = r#"{"Global":{"Name":"M"},"SomeUnknown":{"foo":42}}"#;
        let model: JetiModel = serde_json::from_str(json).unwrap();
        assert!(model.extra.contains_key("SomeUnknown"));
    }
}
