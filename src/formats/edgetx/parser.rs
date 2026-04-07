use crate::error::ConversionError;
use crate::formats::FormatParser;
use crate::ir::model as ir;
use super::EdgeTxFormat;
use super::schema;
use std::collections::HashMap;
use std::time::Duration;

impl FormatParser for EdgeTxFormat {
    type Schema = schema::EdgeTxModel;

    fn parse(&self, input: &[u8]) -> Result<schema::EdgeTxModel, ConversionError> {
        serde_yaml_ng::from_slice(input)
            .map_err(|e| ConversionError::EdgeTxParse(e.to_string()))
    }

    fn to_ir(&self, m: schema::EdgeTxModel) -> Result<ir::ModelIr, ConversionError> {
        // Build input-channel → stick mapping from expoData: chn → StickAxis
        let input_map = build_input_map(&m.expo_data);

        Ok(ir::ModelIr {
            meta: ir::ModelMeta {
                name: m.header.name,
                firmware_origin: ir::FirmwareOrigin::EdgeTx,
                notes: m.header.notes,
            },
            channels: m
                .output_channels
                .into_iter()
                .enumerate()
                .map(|(i, o)| ir::Channel {
                    index: i as u8,
                    name: if o.name.is_empty() { None } else { Some(o.name) },
                    min: ir::Microseconds(o.min),
                    max: ir::Microseconds(o.max),
                    center: ir::Microseconds(o.offset),
                    reversed: o.revert,
                })
                .collect(),
            mixes: m
                .mix_data
                .into_iter()
                .map(|line| mix_to_ir(line, &input_map))
                .collect::<Result<_, _>>()?,
            curves: m.curves.into_iter().map(curve_to_ir).collect(),
            rf_modules: m.module_data.into_iter().map(module_to_ir).collect(),
            telemetry: m.telemetry.into_iter().map(sensor_to_ir).collect(),
            logic_switches: m
                .logical_switches
                .into_iter()
                .enumerate()
                .map(|(i, ls)| ls_to_ir(i as u8, ls))
                .collect::<Result<_, _>>()?,
            special_functions: m
                .special_functions
                .into_iter()
                .map(sf_to_ir)
                .collect::<Result<_, _>>()?,
            timer: m.timers.into_iter().next().map(timer_to_ir),
        })
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a map of input-channel index → StickAxis from the expoData section.
fn build_input_map(expo: &[schema::ExpoLine]) -> HashMap<u8, ir::StickAxis> {
    expo.iter()
        .filter_map(|e| parse_stick_raw(&e.src_raw).map(|axis| (e.chn, axis)))
        .collect()
}

/// Parse a raw stick name ("Ail", "Ele", "Thr", "Rud", "S1", "S2", "LS", "RS").
fn parse_stick_raw(s: &str) -> Option<ir::StickAxis> {
    match s {
        "Ail" | "ail" => Some(ir::StickAxis::Ail),
        "Ele" | "ele" => Some(ir::StickAxis::Ele),
        "Thr" | "thr" => Some(ir::StickAxis::Thr),
        "Rud" | "rud" => Some(ir::StickAxis::Rud),
        "S1"  | "s1"  => Some(ir::StickAxis::S1),
        "S2"  | "s2"  => Some(ir::StickAxis::S2),
        "LS"  | "ls"  => Some(ir::StickAxis::LS),
        "RS"  | "rs"  => Some(ir::StickAxis::RS),
        _ => None,
    }
}

fn parse_src_raw(s: &str, input_map: &HashMap<u8, ir::StickAxis>) -> ir::MixSource {
    // "I0"–"I15" → look up in input_map
    if let Some(rest) = s.strip_prefix('I').or_else(|| s.strip_prefix('i')) {
        if let Ok(idx) = rest.parse::<u8>() {
            if let Some(axis) = input_map.get(&idx) {
                return ir::MixSource::Stick(axis.clone());
            }
        }
    }
    // Direct stick names
    if let Some(axis) = parse_stick_raw(s) {
        return ir::MixSource::Stick(axis);
    }
    // "CH1"–"CH32" (1-indexed in EdgeTX)
    if let Some(rest) = s.strip_prefix("CH").or_else(|| s.strip_prefix("ch")) {
        if let Ok(n) = rest.parse::<u8>() {
            return ir::MixSource::Channel(n.saturating_sub(1));
        }
    }
    ir::MixSource::Constant(ir::Percent(0.0))
}

fn mix_to_ir(
    m: schema::MixLine,
    input_map: &HashMap<u8, ir::StickAxis>,
) -> Result<ir::Mix, ConversionError> {
    let curve = if m.curve.curve_type == 0 {
        None
    } else {
        Some(ir::CurveRef(m.curve.value as u8))
    };

    let mode = match m.mltpx.as_str() {
        "ADD" | "add" | "" => ir::MixMode::Add,
        "MULTIPLY" | "multiply" => ir::MixMode::Multiply,
        "REPLACE" | "replace" => ir::MixMode::Replace,
        other => {
            return Err(ConversionError::EdgeTxParse(format!(
                "unknown mix mode: {other}"
            )))
        }
    };

    let switch = if m.swtch == "NONE" || m.swtch.is_empty() {
        None
    } else {
        Some(parse_switch_condition(&m.swtch))
    };

    Ok(ir::Mix {
        channel_out: m.dest_ch,
        name: if m.name.is_empty() { None } else { Some(m.name) },
        source: parse_src_raw(&m.src_raw, input_map),
        weight: ir::Percent(m.weight as f32),
        offset: ir::Percent(m.offset as f32),
        curve,
        switch,
        mode,
    })
}

fn parse_switch_condition(s: &str) -> ir::SwitchCondition {
    let (sw, pos) = if let Some(rest) = s.strip_suffix('↑') {
        (rest, ir::SwitchPosition::Up)
    } else if let Some(rest) = s.strip_suffix('↓') {
        (rest, ir::SwitchPosition::Down)
    } else if let Some(rest) = s.strip_suffix('-') {
        (rest, ir::SwitchPosition::Mid)
    } else if let Some(rest) = s.strip_prefix('!') {
        (rest, ir::SwitchPosition::Inactive)
    } else {
        (s, ir::SwitchPosition::Active)
    };
    ir::SwitchCondition { switch: sw.to_string(), position: pos }
}

fn curve_to_ir(c: schema::CurveDef) -> ir::Curve {
    if c.curve_type == "expo" {
        ir::Curve::Expo {
            name: c.name,
            expo: ir::Percent(c.points.first().map(|p| p[1] as f32).unwrap_or(0.0)),
            differential: ir::Percent(0.0),
        }
    } else {
        ir::Curve::Custom {
            name: c.name,
            points: c
                .points
                .into_iter()
                .map(|p| ir::CurvePoint { x: ir::Percent(p[0] as f32), y: ir::Percent(p[1] as f32) })
                .collect(),
        }
    }
}

fn module_to_ir(m: schema::ModuleData) -> ir::RfModule {
    ir::RfModule {
        slot: if m.slot == "internal" { ir::RfSlot::Internal } else { ir::RfSlot::External },
        protocol: m.protocol,
        sub_type: m.sub_type,
        channel_range: m.channel_start..=m.channel_end,
        options: m.options,
    }
}

fn sensor_to_ir(s: schema::TelemetrySensor) -> ir::TelemetrySensor {
    ir::TelemetrySensor {
        name: s.name,
        unit: s.unit,
        ratio: s.ratio,
        source: match s.sensor_type.as_str() {
            "calculated" => ir::TelemetrySource::Calculated,
            "custom" => ir::TelemetrySource::Custom,
            _ => ir::TelemetrySource::Physical,
        },
    }
}

fn ls_to_ir(index: u8, ls: schema::LogicalSwitch) -> Result<ir::LogicSwitch, ConversionError> {
    let function = match ls.func.as_str() {
        "AND" | "and" => ir::LsFunction::And,
        "OR" | "or" => ir::LsFunction::Or,
        "XOR" | "xor" => ir::LsFunction::Xor,
        "=" | "eq" => ir::LsFunction::Equal,
        ">" | "gt" => ir::LsFunction::Greater,
        "<" | "lt" => ir::LsFunction::Less,
        "|d|>v" | "abs" => ir::LsFunction::Abs,
        "sticky" => ir::LsFunction::Sticky,
        "edge" => ir::LsFunction::Edge,
        "timer" => ir::LsFunction::Timer,
        _ => ir::LsFunction::And,
    };
    Ok(ir::LogicSwitch {
        index,
        function,
        operand1: ls.v1,
        operand2: ls.v2,
        and_switch: ls.and_switch.as_deref().map(parse_switch_condition),
    })
}

fn sf_to_ir(sf: schema::SpecialFunction) -> Result<ir::SpecialFunction, ConversionError> {
    Ok(ir::SpecialFunction {
        switch: parse_switch_condition(&sf.switch),
        function: sf.func,
        parameter: sf.param,
        enabled: sf.enabled,
    })
}

fn timer_to_ir(t: schema::TimerDef) -> ir::Timer {
    ir::Timer {
        name: t.name,
        mode: match t.mode.as_str() {
            "running" => ir::TimerMode::Running,
            "throttleActive" => ir::TimerMode::ThrottleActive,
            _ => ir::TimerMode::Absolute,
        },
        start: Duration::from_secs(t.start_seconds as u64),
        countdown: t.countdown,
    }
}
