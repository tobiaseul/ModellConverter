use crate::error::ConversionError;
use crate::formats::FormatSerializer;
use crate::ir::model as ir;
use super::EdgeTxFormat;
use super::schema::{self, InputName, MixCurve};
use std::collections::HashMap;

/// EdgeTX firmware version emitted in generated files.
const SEMVER: &str = "2.11.4";

/// Standard RETA input order used by EdgeTX (Rud=I0, Ele=I1, Thr=I2, Ail=I3).
/// Extra axes (S1, S2, LS, RS) get indices 4–7.
const RETA_ORDER: &[ir::StickAxis] = &[
    ir::StickAxis::Rud,
    ir::StickAxis::Ele,
    ir::StickAxis::Thr,
    ir::StickAxis::Ail,
    ir::StickAxis::S1,
    ir::StickAxis::S2,
    ir::StickAxis::LS,
    ir::StickAxis::RS,
];

impl FormatSerializer for EdgeTxFormat {
    type Schema = schema::EdgeTxModel;

    fn from_ir(&self, model: &ir::ModelIr) -> Result<schema::EdgeTxModel, ConversionError> {
        // Collect stick axes used in mixes OR expo settings, preserving RETA order.
        let used_axes: Vec<ir::StickAxis> = RETA_ORDER
            .iter()
            .filter(|ax| {
                model.mixes.iter().any(|m| matches!(&m.source, ir::MixSource::Stick(a) if a == *ax))
                || model.expo_settings.iter().any(|s| &s.axis == *ax)
            })
            .cloned()
            .collect();

        // Map StickAxis → input channel index (I0, I1, …)
        let axis_to_input: HashMap<ir::StickAxis, u8> = used_axes
            .iter()
            .enumerate()
            .map(|(i, ax)| (ax.clone(), i as u8))
            .collect();

        let expo_data = build_expo_data(&used_axes, &axis_to_input, &model.expo_settings, model.flight_modes.len());
        let input_names = build_input_names(&used_axes);
        let mix_data = model.mixes.iter().map(|m| mix_from_ir(m, &axis_to_input)).collect();
        let output_channels = model.channels.iter().map(channel_from_ir).collect();
        let flight_modes = model.flight_modes.iter().map(fm_from_ir).collect();

        Ok(schema::EdgeTxModel {
            semver: SEMVER.into(),
            header: schema::ModelHeader {
                name: model.meta.name.clone(),
                notes: model.meta.notes.clone(),
                ..Default::default()
            },
            mix_data,
            expo_data,
            input_names,
            output_channels,
            flight_modes,
            curves: model.curves.iter().map(curve_from_ir).collect(),
            module_data: model.rf_modules.iter().map(module_from_ir).collect(),
            telemetry: model.telemetry.iter().map(sensor_from_ir).collect(),
            logical_switches: model.logic_switches.iter().map(ls_from_ir).collect(),
            special_functions: model.special_functions.iter().map(sf_from_ir).collect(),
            timers: model.timer.as_ref().map(|t| vec![timer_from_ir(t)]).unwrap_or_default(),
            ..Default::default()
        })
    }

    fn serialize(&self, schema: &schema::EdgeTxModel) -> Result<Vec<u8>, ConversionError> {
        serde_yaml_ng::to_string(schema)
            .map(|s| s.into_bytes())
            .map_err(|e| ConversionError::EdgeTxParse(e.to_string()))
    }
}

// ── expo / input helpers ──────────────────────────────────────────────────────

fn stick_to_raw_name(ax: &ir::StickAxis) -> &'static str {
    match ax {
        ir::StickAxis::Rud => "Rud",
        ir::StickAxis::Ele => "Ele",
        ir::StickAxis::Thr => "Thr",
        ir::StickAxis::Ail => "Ail",
        ir::StickAxis::S1  => "S1",
        ir::StickAxis::S2  => "S2",
        ir::StickAxis::LS  => "LS",
        ir::StickAxis::RS  => "RS",
    }
}

fn build_expo_data(
    axes: &[ir::StickAxis],
    axis_to_input: &HashMap<ir::StickAxis, u8>,
    expo_settings: &[ir::ExpoSetting],
    num_fms: usize,
) -> Vec<schema::ExpoLine> {
    let mut lines = vec![];
    for ax in axes {
        let chn = *axis_to_input.get(ax).unwrap_or(&0);
        let ax_specs: Vec<&ir::ExpoSetting> = expo_settings.iter().filter(|s| &s.axis == ax).collect();

        if ax_specs.is_empty() {
            // No expo configured for this axis — emit a plain passthrough line.
            lines.push(schema::ExpoLine {
                src_raw: stick_to_raw_name(ax).into(),
                mode: 3,
                chn,
                weight: 100,
                ..Default::default()
            });
            continue;
        }

        // Check if all FMs share the same DR and curve.
        let all_same = ax_specs.windows(2).all(|w| w[0].dr == w[1].dr && w[0].curve == w[1].curve);

        if all_same || num_fms <= 1 {
            // Single line active in all FMs.
            let s = &ax_specs[0];
            lines.push(expo_line(ax, chn, s, schema::default_flight_modes()));
        } else {
            // One line per FM, each active only in its flight mode.
            for s in &ax_specs {
                let fm_mask = fm_mask(s.flight_mode_idx, num_fms);
                lines.push(expo_line(ax, chn, s, fm_mask));
            }
        }
    }
    lines
}

/// Build a 9-character flight-mode mask string where position `active_idx` is '0'
/// (active) and all other positions up to `num_fms` are '1' (inactive).
fn fm_mask(active_idx: usize, num_fms: usize) -> String {
    (0..9)
        .map(|i| if i == active_idx && i < num_fms { '0' } else { '1' })
        .collect()
}

fn expo_line(ax: &ir::StickAxis, chn: u8, s: &ir::ExpoSetting, flight_modes: String) -> schema::ExpoLine {
    let curve = match &s.curve {
        None => MixCurve::default(),
        Some(ir::CurveRef(idx)) => MixCurve { curve_type: 6, value: *idx as i32 },
    };
    schema::ExpoLine {
        src_raw: stick_to_raw_name(ax).into(),
        mode: 3,
        chn,
        weight: s.dr.0 as i32,
        curve,
        flight_modes,
        ..Default::default()
    }
}

fn build_input_names(axes: &[ir::StickAxis]) -> HashMap<String, InputName> {
    axes.iter()
        .enumerate()
        .map(|(i, ax)| (i.to_string(), InputName { val: stick_to_raw_name(ax).into() }))
        .collect()
}

// ── mix ───────────────────────────────────────────────────────────────────────

fn src_raw_str(source: &ir::MixSource, axis_to_input: &HashMap<ir::StickAxis, u8>) -> String {
    match source {
        ir::MixSource::Stick(ax) => {
            if let Some(&idx) = axis_to_input.get(ax) {
                format!("I{}", idx)
            } else {
                stick_to_raw_name(ax).into()
            }
        }
        ir::MixSource::Channel(n) => format!("CH{}", n + 1),
        ir::MixSource::Switch(sw) => sw.clone(),
        ir::MixSource::Constant(p) => format!("{}", p.0 as i32),
        ir::MixSource::Trainer(ch) => format!("TR{}", ch),
    }
}

fn mix_from_ir(m: &ir::Mix, axis_to_input: &HashMap<ir::StickAxis, u8>) -> schema::MixLine {
    let curve = match &m.curve {
        None => MixCurve::default(),
        Some(ir::CurveRef(idx)) => MixCurve { curve_type: 6, value: *idx as i32 },
    };
    let swtch = m.switch.as_ref().map(switch_condition_to_str).unwrap_or_else(|| "NONE".into());

    schema::MixLine {
        dest_ch: m.channel_out,
        src_raw: src_raw_str(&m.source, axis_to_input),
        weight: m.weight.0 as i32,
        swtch,
        curve,
        mltpx: match m.mode {
            ir::MixMode::Add => "ADD".into(),
            ir::MixMode::Multiply => "MULTIPLY".into(),
            ir::MixMode::Replace => "REPLACE".into(),
        },
        offset: m.offset.0 as i32,
        name: m.name.clone().unwrap_or_default(),
        ..Default::default()
    }
}

fn switch_condition_to_str(sc: &ir::SwitchCondition) -> String {
    let suffix = match sc.position {
        ir::SwitchPosition::Up => "↑",
        ir::SwitchPosition::Mid => "-",
        ir::SwitchPosition::Down => "↓",
        ir::SwitchPosition::Active => "",
        ir::SwitchPosition::Inactive => "!",
    };
    format!("{}{}", sc.switch, suffix)
}

// ── flight modes ─────────────────────────────────────────────────────────────

fn fm_from_ir(fm: &ir::FlightMode) -> schema::FlightModeDef {
    schema::FlightModeDef {
        name: fm.name.clone(),
        switch: "NONE".into(),
        fade_in: 0,
        fade_out: 0,
    }
}

// ── output channels ───────────────────────────────────────────────────────────

fn channel_from_ir(ch: &ir::Channel) -> schema::OutputChannel {
    schema::OutputChannel {
        name: ch.name.clone().unwrap_or_default(),
        min: ch.min.0,
        max: ch.max.0,
        offset: ch.center.0,
        revert: ch.reversed,
        ..Default::default()
    }
}

// ── curves ───────────────────────────────────────────────────────────────────

fn curve_from_ir(c: &ir::Curve) -> schema::CurveDef {
    match c {
        ir::Curve::Custom { name, points } => schema::CurveDef {
            name: name.clone(),
            curve_type: "custom".into(),
            points: points.iter().map(|p| [p.x.0 as i32, p.y.0 as i32]).collect(),
        },
        ir::Curve::Expo { name, expo, .. } => schema::CurveDef {
            name: name.clone(),
            curve_type: "expo".into(),
            points: vec![[0, expo.0 as i32]],
        },
    }
}

// ── modules ───────────────────────────────────────────────────────────────────

fn module_from_ir(m: &ir::RfModule) -> schema::ModuleData {
    schema::ModuleData {
        slot: match m.slot { ir::RfSlot::Internal => "internal".into(), ir::RfSlot::External => "external".into() },
        protocol: m.protocol.clone(),
        sub_type: m.sub_type.clone(),
        channel_start: *m.channel_range.start(),
        channel_end: *m.channel_range.end(),
        options: m.options.clone(),
    }
}

// ── sensors ───────────────────────────────────────────────────────────────────

fn sensor_from_ir(s: &ir::TelemetrySensor) -> schema::TelemetrySensor {
    schema::TelemetrySensor {
        name: s.name.clone(),
        unit: s.unit.clone(),
        ratio: s.ratio,
        sensor_type: match s.source {
            ir::TelemetrySource::Physical => "physical".into(),
            ir::TelemetrySource::Calculated => "calculated".into(),
            ir::TelemetrySource::Custom => "custom".into(),
        },
    }
}

// ── logical switches ──────────────────────────────────────────────────────────

fn ls_from_ir(ls: &ir::LogicSwitch) -> schema::LogicalSwitch {
    schema::LogicalSwitch {
        func: match ls.function {
            ir::LsFunction::And => "AND", ir::LsFunction::Or => "OR",
            ir::LsFunction::Xor => "XOR", ir::LsFunction::Equal => "=",
            ir::LsFunction::Greater => ">", ir::LsFunction::Less => "<",
            ir::LsFunction::Abs => "|d|>v", ir::LsFunction::Sticky => "sticky",
            ir::LsFunction::Edge => "edge", ir::LsFunction::Timer => "timer",
        }.into(),
        v1: ls.operand1.clone(),
        v2: ls.operand2.clone(),
        and_switch: ls.and_switch.as_ref().map(switch_condition_to_str),
    }
}

// ── special functions ─────────────────────────────────────────────────────────

fn sf_from_ir(sf: &ir::SpecialFunction) -> schema::SpecialFunction {
    schema::SpecialFunction {
        switch: switch_condition_to_str(&sf.switch),
        func: sf.function.clone(),
        param: sf.parameter.clone(),
        enabled: sf.enabled,
    }
}

// ── timers ────────────────────────────────────────────────────────────────────

fn timer_from_ir(t: &ir::Timer) -> schema::TimerDef {
    schema::TimerDef {
        name: t.name.clone(),
        mode: match t.mode {
            ir::TimerMode::Absolute => "absolute".into(),
            ir::TimerMode::Running => "running".into(),
            ir::TimerMode::ThrottleActive => "throttleActive".into(),
        },
        start_seconds: t.start.as_secs() as u32,
        countdown: t.countdown,
    }
}
