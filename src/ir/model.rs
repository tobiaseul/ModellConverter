use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct ModelIr {
    pub meta: ModelMeta,
    pub channels: Vec<Channel>,
    pub mixes: Vec<Mix>,
    pub curves: Vec<Curve>,
    pub rf_modules: Vec<RfModule>,
    pub telemetry: Vec<TelemetrySensor>,
    pub logic_switches: Vec<LogicSwitch>,
    pub special_functions: Vec<SpecialFunction>,
    pub timer: Option<Timer>,
    /// Named flight modes in priority order (index 0 = lowest priority / base).
    pub flight_modes: Vec<FlightMode>,
    /// Per-axis, per-flight-mode expo and dual-rate settings.
    pub expo_settings: Vec<ExpoSetting>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelMeta {
    pub name: String,
    pub firmware_origin: FirmwareOrigin,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FirmwareOrigin {
    EdgeTx,
    Ethos,
    JetiDuplex,
    Unknown,
}

/// A physical output channel (servo, ESC, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub index: u8,
    pub name: Option<String>,
    pub min: Microseconds,
    pub max: Microseconds,
    pub center: Microseconds,
    pub reversed: bool,
}

/// A mixer input → output mapping
#[derive(Debug, Clone, PartialEq)]
pub struct Mix {
    pub channel_out: u8,
    pub name: Option<String>,
    pub source: MixSource,
    pub weight: Percent,
    pub offset: Percent,
    pub curve: Option<CurveRef>,
    pub switch: Option<SwitchCondition>,
    pub mode: MixMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MixSource {
    Stick(StickAxis),
    Channel(u8),
    Switch(String),
    Constant(Percent),
    Trainer(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MixMode {
    Add,
    Replace,
    Multiply,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StickAxis {
    Ail,
    Ele,
    Thr,
    Rud,
    S1,
    S2,
    LS,
    RS,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurveRef(pub u8);

#[derive(Debug, Clone, PartialEq)]
pub enum Curve {
    Custom {
        name: Option<String>,
        points: Vec<CurvePoint>,
    },
    Expo {
        name: Option<String>,
        expo: Percent,
        differential: Percent,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RfModule {
    pub slot: RfSlot,
    pub protocol: String,
    pub sub_type: Option<String>,
    pub channel_range: RangeInclusive<u8>,
    pub options: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RfSlot {
    Internal,
    External,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TelemetrySensor {
    pub name: String,
    pub unit: Option<String>,
    pub ratio: Option<f32>,
    pub source: TelemetrySource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TelemetrySource {
    Physical,
    Calculated,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicSwitch {
    pub index: u8,
    pub function: LsFunction,
    pub operand1: Option<String>,
    pub operand2: Option<String>,
    pub and_switch: Option<SwitchCondition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LsFunction {
    And,
    Or,
    Xor,
    Equal,
    Greater,
    Less,
    Abs,
    Sticky,
    Edge,
    Timer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCondition {
    pub switch: String,
    pub position: SwitchPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchPosition {
    Up,
    Mid,
    Down,
    Active,
    Inactive,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpecialFunction {
    pub switch: SwitchCondition,
    pub function: String,
    pub parameter: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timer {
    pub name: Option<String>,
    pub mode: TimerMode,
    pub start: Duration,
    pub countdown: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimerMode {
    Absolute,
    Running,
    ThrottleActive,
}

/// A named flight mode (base mode is index 0).
#[derive(Debug, Clone, PartialEq)]
pub struct FlightMode {
    pub name: String,
}

/// Expo and dual-rate settings for one axis in one flight mode.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpoSetting {
    /// Index into ModelIr::flight_modes.
    pub flight_mode_idx: usize,
    pub axis: StickAxis,
    /// Dual-rate: 100 = full deflection, 50 = half.
    pub dr: Percent,
    /// Optional expo curve (index into ModelIr::curves).
    pub curve: Option<CurveRef>,
}

/// Newtype for microsecond values (servo pulse widths, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Microseconds(pub i32);

/// Newtype for percentage values (-100.0 to 100.0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percent(pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvePoint {
    pub x: Percent,
    pub y: Percent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn microseconds_ordering() {
        assert!(Microseconds(1000) < Microseconds(2000));
        assert!(Microseconds(-100) < Microseconds(0));
    }

    #[test]
    fn percent_value() {
        let p = Percent(50.0);
        assert_eq!(p.0, 50.0);
    }
}
