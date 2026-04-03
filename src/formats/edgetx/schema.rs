use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level EdgeTX model YAML structure.
/// Fields are kept optional to handle version differences gracefully.
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EdgeTxModel {
    pub header: ModelHeader,
    #[serde(default)]
    pub mixes: Vec<MixLine>,
    #[serde(default)]
    pub outputs: Vec<OutputChannel>,
    #[serde(default)]
    pub curves: Vec<CurveDef>,
    #[serde(default, rename = "logicalSwitches")]
    pub logical_switches: Vec<LogicalSwitch>,
    #[serde(default, rename = "specialFunctions")]
    pub special_functions: Vec<SpecialFunction>,
    #[serde(default)]
    pub telemetry: Vec<TelemetrySensor>,
    #[serde(default)]
    pub timers: Vec<TimerDef>,
    #[serde(default, rename = "moduleData")]
    pub module_data: Vec<ModuleData>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ModelHeader {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MixLine {
    #[serde(rename = "ch")]
    pub channel: u8,
    pub name: Option<String>,
    pub source: String,
    pub weight: i32,
    pub offset: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch: Option<String>,
    #[serde(default)]
    pub mode: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OutputChannel {
    pub name: Option<String>,
    pub min: i32,
    pub max: i32,
    pub offset: i32,
    #[serde(default)]
    pub reverse: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CurveDef {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub curve_type: String,
    #[serde(default)]
    pub points: Vec<[i32; 2]>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LogicalSwitch {
    pub func: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v2: Option<String>,
    #[serde(rename = "andSwitch", skip_serializing_if = "Option::is_none")]
    pub and_switch: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SpecialFunction {
    pub switch: String,
    pub func: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TelemetrySensor {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio: Option<f32>,
    #[serde(rename = "type")]
    pub sensor_type: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TimerDef {
    pub name: Option<String>,
    pub mode: String,
    #[serde(rename = "start")]
    pub start_seconds: u32,
    #[serde(default)]
    pub countdown: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ModuleData {
    pub slot: String,
    pub protocol: String,
    #[serde(rename = "subType", skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<String>,
    #[serde(rename = "channelStart", default)]
    pub channel_start: u8,
    #[serde(rename = "channelEnd", default)]
    pub channel_end: u8,
    #[serde(default)]
    pub options: HashMap<String, String>,
}
