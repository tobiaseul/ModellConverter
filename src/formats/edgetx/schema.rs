/// EdgeTX model YAML schema — matches the format EdgeTX firmware reads/writes
/// (semver 2.11.x, field names verified against MODEL01.yml).
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── top-level model ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EdgeTxModel {
    pub semver: String,
    pub header: ModelHeader,

    // Model-level integer flags (all default 0)
    pub no_global_functions: u8,
    pub thr_trim: u8,
    pub trim_inc: u8,
    pub display_trims: u8,
    pub ignore_sensor_ids: u8,
    pub show_instance_ids: u8,
    pub disable_throttle_warning: u8,
    pub enable_custom_throttle_warning: u8,
    pub custom_throttle_warning_position: u8,
    #[serde(rename = "beepANACenter")]
    pub beep_ana_center: u8,
    pub extended_limits: u8,
    pub extended_trims: u8,
    pub throttle_reversed: u8,
    pub checklist_interactive: u8,

    // Main mixing table: outputs ← inputs
    pub mix_data: Vec<MixLine>,
    // Input processing (expo / dual-rates): stick → named input channel
    pub expo_data: Vec<ExpoLine>,
    // Named input channels (key = channel index as string)
    pub input_names: HashMap<String, InputName>,

    pub swash_r: SwashR,
    #[serde(default = "default_thr_trace")]
    pub thr_trace_src: String,
    pub switch_warning: HashMap<String, SwitchPos>,
    pub thr_trim_sw: u8,
    #[serde(default = "default_pots_warn_mode")]
    pub pots_warn_mode: String,
    pub pots_warn_enabled: u8,
    #[serde(default = "default_global")]
    pub jitter_filter: String,
    pub display_checklist: u8,
    pub telemetry_protocol: u8,
    pub vario_data: VarioData,
    #[serde(default = "default_none_str")]
    pub rssi_source: String,
    pub rf_alarms: RfAlarms,
    pub disable_telemetry_warning: u8,
    pub trainer_data: TrainerData,
    pub model_registration_id: String,
    #[serde(default = "default_global")]
    pub hats_mode: String,
    pub usb_joystick_ext_mode: u8,
    #[serde(default = "default_joystick")]
    pub usb_joystick_if_mode: String,
    pub usb_joystick_circular_cut: u8,

    // Per-feature global/disable overrides
    #[serde(default = "default_global")] pub radio_gf_disabled: String,
    #[serde(default = "default_global")] pub radio_trainer_disabled: String,
    #[serde(default = "default_global")] pub model_heli_disabled: String,
    #[serde(default = "default_global")] pub model_fm_disabled: String,
    #[serde(default = "default_global")] pub model_curves_disabled: String,
    #[serde(default = "default_global")] pub model_gv_disabled: String,
    #[serde(default = "default_global")] pub model_ls_disabled: String,
    #[serde(default = "default_global")] pub model_sf_disabled: String,
    #[serde(default = "default_global")] pub model_custom_scripts_disabled: String,
    #[serde(default = "default_global")] pub model_telemetry_disabled: String,

    // Optional sections (absent = empty / no limits set)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_channels: Vec<OutputChannel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub curves: Vec<CurveDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub logical_switches: Vec<LogicalSwitch>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub special_functions: Vec<SpecialFunction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub telemetry: Vec<TelemetrySensor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timers: Vec<TimerDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_data: Vec<ModuleData>,
}

impl Default for EdgeTxModel {
    fn default() -> Self {
        Self {
            semver: String::new(),
            header: ModelHeader::default(),
            no_global_functions: 0, thr_trim: 0, trim_inc: 0, display_trims: 0,
            ignore_sensor_ids: 0, show_instance_ids: 0, disable_throttle_warning: 0,
            enable_custom_throttle_warning: 0, custom_throttle_warning_position: 0,
            beep_ana_center: 0, extended_limits: 0, extended_trims: 0,
            throttle_reversed: 0, checklist_interactive: 0,
            mix_data: vec![], expo_data: vec![], input_names: Default::default(),
            swash_r: SwashR::default(),
            thr_trace_src: default_thr_trace(),
            switch_warning: Default::default(),
            thr_trim_sw: 0,
            pots_warn_mode: default_pots_warn_mode(),
            pots_warn_enabled: 0,
            jitter_filter: default_global(),
            display_checklist: 0, telemetry_protocol: 0,
            vario_data: VarioData::default(),
            rssi_source: default_none_str(),
            rf_alarms: RfAlarms::default(),
            disable_telemetry_warning: 0,
            trainer_data: TrainerData::default(),
            model_registration_id: String::new(),
            hats_mode: default_global(),
            usb_joystick_ext_mode: 0,
            usb_joystick_if_mode: default_joystick(),
            usb_joystick_circular_cut: 0,
            radio_gf_disabled: default_global(),
            radio_trainer_disabled: default_global(),
            model_heli_disabled: default_global(),
            model_fm_disabled: default_global(),
            model_curves_disabled: default_global(),
            model_gv_disabled: default_global(),
            model_ls_disabled: default_global(),
            model_sf_disabled: default_global(),
            model_custom_scripts_disabled: default_global(),
            model_telemetry_disabled: default_global(),
            output_channels: vec![], curves: vec![], logical_switches: vec![],
            special_functions: vec![], telemetry: vec![], timers: vec![],
            module_data: vec![],
        }
    }
}

fn default_global() -> String { "GLOBAL".into() }
fn default_joystick() -> String { "JOYSTICK".into() }
fn default_pots_warn_mode() -> String { "WARN_OFF".into() }
fn default_none_str() -> String { "none".into() }
fn default_thr_trace() -> String { "Thr".into() }
pub fn default_swtch() -> String { "NONE".into() }
pub fn default_flight_modes() -> String { "000000000".into() }

// ── header ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ModelHeader {
    pub name: String,
    pub bitmap: String,
    pub labels: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// ── mixData ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MixLine {
    pub dest_ch: u8,
    /// "I0"–"I15" (input channel), "Ail"/"Ele"/"Thr"/"Rud", "CH1"–"CH32", etc.
    pub src_raw: String,
    pub weight: i32,
    #[serde(default = "default_swtch")]
    pub swtch: String,
    pub curve: MixCurve,
    pub delay_prec: u8,
    pub delay_up: u8,
    pub delay_down: u8,
    pub speed_prec: u8,
    pub speed_up: u8,
    pub speed_down: u8,
    pub carry_trim: u8,
    /// "ADD", "MULTIPLY", "REPLACE"
    #[serde(default = "default_mltpx")]
    pub mltpx: String,
    pub mix_warn: u8,
    #[serde(default = "default_flight_modes")]
    pub flight_modes: String,
    pub offset: i32,
    pub name: String,
}

fn default_mltpx() -> String { "ADD".into() }

impl Default for MixLine {
    fn default() -> Self {
        Self {
            dest_ch: 0, src_raw: String::new(), weight: 0,
            swtch: default_swtch(), curve: MixCurve::default(),
            delay_prec: 0, delay_up: 0, delay_down: 0,
            speed_prec: 0, speed_up: 0, speed_down: 0, carry_trim: 0,
            mltpx: default_mltpx(), mix_warn: 0,
            flight_modes: default_flight_modes(),
            offset: 0, name: String::new(),
        }
    }
}

/// Curve reference embedded in a mix / expo line.
/// type=0 → no curve; type=6 → custom curve by index (value = index).
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct MixCurve {
    #[serde(rename = "type")]
    pub curve_type: u8,
    pub value: i32,
}

// ── expoData ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExpoLine {
    /// Raw stick: "Ail", "Ele", "Thr", "Rud", "S1", "S2", "LS", "RS"
    pub src_raw: String,
    pub scale: u8,
    /// 3 = both directions (normal mode)
    pub mode: u8,
    /// Index into the input channel array (feeds I{chn} in mixData)
    pub chn: u8,
    #[serde(default = "default_swtch")]
    pub swtch: String,
    #[serde(default = "default_flight_modes")]
    pub flight_modes: String,
    pub weight: i32,
    pub offset: i32,
    pub curve: MixCurve,
    pub trim_source: u8,
    pub name: String,
}

impl Default for ExpoLine {
    fn default() -> Self {
        Self {
            src_raw: String::new(), scale: 0, mode: 3, chn: 0,
            swtch: default_swtch(), flight_modes: default_flight_modes(),
            weight: 100, offset: 0, curve: MixCurve::default(),
            trim_source: 0, name: String::new(),
        }
    }
}

// ── inputNames ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct InputName {
    pub val: String,
}

// ── swashR ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SwashR {
    #[serde(rename = "type")]
    pub swash_type: String,
    pub value: u8,
    pub collective_source: String,
    pub aileron_source: String,
    pub elevator_source: String,
    pub collective_weight: i8,
    pub aileron_weight: i8,
    pub elevator_weight: i8,
}

impl Default for SwashR {
    fn default() -> Self {
        Self {
            swash_type: "TYPE_NONE".into(),
            value: 0,
            collective_source: "NONE".into(),
            aileron_source: "NONE".into(),
            elevator_source: "NONE".into(),
            collective_weight: 0,
            aileron_weight: 0,
            elevator_weight: 0,
        }
    }
}

// ── switchWarning ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SwitchPos {
    pub pos: String,
}

// ── varioData ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VarioData {
    pub source: String,
    pub center_silent: u8,
    pub center_max: i32,
    pub center_min: i32,
    pub min: i32,
    pub max: i32,
}

impl Default for VarioData {
    fn default() -> Self {
        Self { source: "none".into(), center_silent: 0, center_max: 0, center_min: 0, min: 0, max: 0 }
    }
}

// ── rfAlarms ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct RfAlarms {
    pub warning: u8,
    pub critical: u8,
}

impl Default for RfAlarms {
    fn default() -> Self { Self { warning: 45, critical: 42 } }
}

// ── trainerData ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TrainerData {
    pub mode: String,
    pub channels_start: u8,
    pub channels_count: i8,
    pub frame_length: u8,
    pub delay: u8,
    pub pulse_pol: u8,
}

impl Default for TrainerData {
    fn default() -> Self {
        Self { mode: "OFF".into(), channels_start: 0, channels_count: -8, frame_length: 0, delay: 0, pulse_pol: 0 }
    }
}

// ── outputChannels ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OutputChannel {
    pub name: String,
    pub min: i32,
    pub max: i32,
    pub offset: i32,
    pub revert: bool,
    pub curve: MixCurve,
    pub ppm_center: u16,
    pub symetrical: u8,
    pub failsafe: i32,
}

// ── curves ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CurveDef {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub curve_type: String,
    #[serde(default)]
    pub points: Vec<[i32; 2]>,
}

// ── logicalSwitches ───────────────────────────────────────────────────────────

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

// ── specialFunctions ─────────────────────────────────────────────────────────

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

fn default_true() -> bool { true }

// ── telemetry ─────────────────────────────────────────────────────────────────

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

// ── timers ────────────────────────────────────────────────────────────────────

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

// ── moduleData ────────────────────────────────────────────────────────────────

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
