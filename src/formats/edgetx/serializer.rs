use crate::error::ConversionError;
use crate::formats::FormatSerializer;
use crate::ir::model as ir;
use super::EdgeTxFormat;
use super::schema;

impl FormatSerializer for EdgeTxFormat {
    type Schema = schema::EdgeTxModel;

    fn from_ir(&self, model: &ir::ModelIr) -> Result<schema::EdgeTxModel, ConversionError> {
        Ok(schema::EdgeTxModel {
            header: schema::ModelHeader {
                name: model.meta.name.clone(),
                notes: model.meta.notes.clone(),
            },
            outputs: model
                .channels
                .iter()
                .map(|c| schema::OutputChannel {
                    name: c.name.clone(),
                    min: c.min.0,
                    max: c.max.0,
                    offset: c.center.0,
                    reverse: c.reversed,
                })
                .collect(),
            mixes: model.mixes.iter().map(mix_from_ir).collect(),
            curves: model.curves.iter().map(curve_from_ir).collect(),
            module_data: model.rf_modules.iter().map(module_from_ir).collect(),
            telemetry: model.telemetry.iter().map(sensor_from_ir).collect(),
            logical_switches: model.logic_switches.iter().map(ls_from_ir).collect(),
            special_functions: model.special_functions.iter().map(sf_from_ir).collect(),
            timers: model.timer.as_ref().map(|t| vec![timer_from_ir(t)]).unwrap_or_default(),
        })
    }

    fn serialize(&self, schema: &schema::EdgeTxModel) -> Result<Vec<u8>, ConversionError> {
        serde_yaml_ng::to_string(schema)
            .map(|s| s.into_bytes())
            .map_err(|e| ConversionError::EdgeTxParse(e.to_string()))
    }
}

fn mix_from_ir(m: &ir::Mix) -> schema::MixLine {
    schema::MixLine {
        channel: m.channel_out,
        name: m.name.clone(),
        source: source_to_str(&m.source),
        weight: m.weight.0 as i32,
        offset: m.offset.0 as i32,
        curve: m.curve.as_ref().map(|c| format!("cv{}", c.0)),
        switch: m.switch.as_ref().map(switch_condition_to_str),
        mode: match m.mode {
            ir::MixMode::Add => "add".to_string(),
            ir::MixMode::Replace => "replace".to_string(),
            ir::MixMode::Multiply => "multiply".to_string(),
        },
    }
}

fn source_to_str(s: &ir::MixSource) -> String {
    match s {
        ir::MixSource::Stick(axis) => match axis {
            ir::StickAxis::Ail => "Ail",
            ir::StickAxis::Ele => "Ele",
            ir::StickAxis::Thr => "Thr",
            ir::StickAxis::Rud => "Rud",
            ir::StickAxis::S1 => "S1",
            ir::StickAxis::S2 => "S2",
            ir::StickAxis::LS => "LS",
            ir::StickAxis::RS => "RS",
        }
        .to_string(),
        ir::MixSource::Channel(ch) => format!("ch{}", ch),
        ir::MixSource::Switch(sw) => sw.clone(),
        ir::MixSource::Constant(p) => format!("{}", p.0 as i32),
        ir::MixSource::Trainer(ch) => format!("tr{}", ch),
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

fn curve_from_ir(c: &ir::Curve) -> schema::CurveDef {
    match c {
        ir::Curve::Custom { name, points } => schema::CurveDef {
            name: name.clone(),
            curve_type: "custom".to_string(),
            points: points.iter().map(|p| [p.x.0 as i32, p.y.0 as i32]).collect(),
        },
        ir::Curve::Expo { name, expo, .. } => schema::CurveDef {
            name: name.clone(),
            curve_type: "expo".to_string(),
            points: vec![[0, expo.0 as i32]],
        },
    }
}

fn module_from_ir(m: &ir::RfModule) -> schema::ModuleData {
    schema::ModuleData {
        slot: match m.slot {
            ir::RfSlot::Internal => "internal".to_string(),
            ir::RfSlot::External => "external".to_string(),
        },
        protocol: m.protocol.clone(),
        sub_type: m.sub_type.clone(),
        channel_start: *m.channel_range.start(),
        channel_end: *m.channel_range.end(),
        options: m.options.clone(),
    }
}

fn sensor_from_ir(s: &ir::TelemetrySensor) -> schema::TelemetrySensor {
    schema::TelemetrySensor {
        name: s.name.clone(),
        unit: s.unit.clone(),
        ratio: s.ratio,
        sensor_type: match s.source {
            ir::TelemetrySource::Physical => "physical".to_string(),
            ir::TelemetrySource::Calculated => "calculated".to_string(),
            ir::TelemetrySource::Custom => "custom".to_string(),
        },
    }
}

fn ls_from_ir(ls: &ir::LogicSwitch) -> schema::LogicalSwitch {
    schema::LogicalSwitch {
        func: match ls.function {
            ir::LsFunction::And => "AND",
            ir::LsFunction::Or => "OR",
            ir::LsFunction::Xor => "XOR",
            ir::LsFunction::Equal => "=",
            ir::LsFunction::Greater => ">",
            ir::LsFunction::Less => "<",
            ir::LsFunction::Abs => "|d|>v",
            ir::LsFunction::Sticky => "sticky",
            ir::LsFunction::Edge => "edge",
            ir::LsFunction::Timer => "timer",
        }
        .to_string(),
        v1: ls.operand1.clone(),
        v2: ls.operand2.clone(),
        and_switch: ls.and_switch.as_ref().map(switch_condition_to_str),
    }
}

fn sf_from_ir(sf: &ir::SpecialFunction) -> schema::SpecialFunction {
    schema::SpecialFunction {
        switch: switch_condition_to_str(&sf.switch),
        func: sf.function.clone(),
        param: sf.parameter.clone(),
        enabled: sf.enabled,
    }
}

fn timer_from_ir(t: &ir::Timer) -> schema::TimerDef {
    schema::TimerDef {
        name: t.name.clone(),
        mode: match t.mode {
            ir::TimerMode::Absolute => "absolute".to_string(),
            ir::TimerMode::Running => "running".to_string(),
            ir::TimerMode::ThrottleActive => "throttleActive".to_string(),
        },
        start_seconds: t.start.as_secs() as u32,
        countdown: t.countdown,
    }
}
