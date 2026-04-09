use crate::error::ConversionError;
use crate::formats::FormatSerializer;
use crate::ir::model::*;
use super::JetiFormat;
use super::schema::{
    JetiFunction, JetiGlobal, JetiMixValue, JetiModel, JetiSection, JetiServo,
    JetiTelemSensor, JetiTimer,
};
use serde_json::{Map, Value};

impl FormatSerializer for JetiFormat {
    type Schema = JetiModel;

    fn from_ir(&self, ir: &ModelIr) -> Result<JetiModel, ConversionError> {
        let servos = JetiSection {
            type_name: "Servos".into(),
            data: ir.channels.iter().map(channel_to_servo).collect(),
        };

        // Assign a function ID per unique mix source
        let (functions_data, mixes_main_data, mixes_values_data) =
            build_mix_sections(&ir.mixes);

        let telem = JetiSection {
            type_name: "Telem-Detect".into(),
            data: ir.telemetry.iter().map(sensor_from_ir).collect(),
        };

        let timers = JetiSection {
            type_name: "Timers".into(),
            data: ir.timer.iter().map(timer_from_ir).collect(),
        };

        Ok(JetiModel {
            global: JetiGlobal {
                name: ir.meta.name.clone(),
                version: 680,
                tx_vers: "5.08".into(),
                model_type: 1,
                filename: String::new(),
                desc: String::new(),
                extra: Map::new(),
            },
            servos: Some(servos),
            functions: Some(JetiSection {
                type_name: "Functions".into(),
                data: functions_data,
            }),
            mixes_main: Some(JetiSection {
                type_name: "Mixes-Main".into(),
                data: mixes_main_data,
            }),
            mixes_values: Some(mixes_values_data),
            flight_modes: None,
            function_specs: None,
            timers: Some(timers),
            telem_detect: Some(telem),
            extra: Map::new(),
        })
    }

    fn serialize(&self, schema: &JetiModel) -> Result<Vec<u8>, ConversionError> {
        serde_json::to_vec_pretty(schema)
            .map_err(|e| ConversionError::JetiParse(e.to_string()))
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn channel_to_servo(ch: &Channel) -> JetiServo {
    let center_us = ch.center.0;
    let middle = (center_us - 1500) / 5;
    let max_positive = (ch.max.0 - center_us) / 5;
    let max_negative = (ch.min.0 - center_us) / 5;
    JetiServo {
        index: ch.index,
        middle,
        max_positive,
        max_negative,
        reversed: if ch.reversed { 1 } else { 0 },
        delay_positive: 0,
        delay_negative: 0,
        curve: vec![0; 13],
        extra: Map::new(),
    }
}

/// Returns (functions, mixes_main entries, mixes_values entries).
fn build_mix_sections(
    mixes: &[Mix],
) -> (Vec<JetiFunction>, Vec<Vec<Value>>, Vec<JetiMixValue>) {
    let mut functions: Vec<JetiFunction> = Vec::new();
    let mut main: Vec<Vec<Value>> = Vec::new();
    let mut values: Vec<JetiMixValue> = Vec::new();

    for (i, mix) in mixes.iter().enumerate() {
        let func_id = (i + 1) as u32;
        let control = mix_source_to_control(&mix.source);
        functions.push(JetiFunction {
            id: func_id,
            label: mix.name.clone().unwrap_or_else(|| format!("Mix{}", func_id)),
            control,
            extra: Map::new(),
        });

        main.push(vec![
            Value::from(mix.channel_out as u64),
            Value::from(func_id as u64),
            Value::from(1u64),
            Value::from(1u64),
        ]);

        let (intensity, direction) = if mix.weight.0 < 0.0 {
            ((-mix.weight.0) as i32, -1)
        } else {
            (mix.weight.0 as i32, 1)
        };

        values.push(JetiMixValue {
            flight_mode: 0,
            intensity,
            direction,
            switch: "0,0,0,0,1,4000,-1,0".into(),
            curve_type: 0,
            points_in: vec![-100, 0, 100],
            points_out: vec![-100, 0, 100],
            extra: Map::new(),
        });
    }

    (functions, main, values)
}

fn mix_source_to_control(source: &MixSource) -> String {
    let axis = match source {
        MixSource::Stick(StickAxis::Ail) => 1,
        MixSource::Stick(StickAxis::Thr) => 2,
        MixSource::Stick(StickAxis::Ele) => 3,
        MixSource::Stick(StickAxis::Rud) => 4,
        MixSource::Stick(StickAxis::S1)  => 5,
        MixSource::Stick(StickAxis::S2)  => 6,
        MixSource::Stick(StickAxis::LS)  => 7,
        MixSource::Stick(StickAxis::RS)  => 8,
        MixSource::Channel(n) => return format!("{},0,1,1,1,0,-1,0", 100 + n),
        MixSource::Switch(s)  => return s.clone(),
        MixSource::Trainer(_) | MixSource::Constant(_) => 0,
    };
    format!("{},0,1,1,1,0,-1,0", axis)
}

fn sensor_from_ir(s: &TelemetrySensor) -> JetiTelemSensor {
    let decimal = s.ratio.map(|r| {
        // ratio = 10^(-decimal) → decimal = -log10(ratio)
        (-r.log10()).round() as u32
    }).unwrap_or(0);
    JetiTelemSensor {
        label: s.name.clone(),
        unit: s.unit.clone().unwrap_or_default(),
        decimal,
        saving: 1,
        extra: Map::new(),
    }
}

fn timer_from_ir(t: &Timer) -> JetiTimer {
    JetiTimer {
        id: 1,
        label: t.name.clone().unwrap_or_else(|| "Timer1".into()),
        mode: if t.countdown { 1 } else { 0 },
        start: t.start.as_secs() as u32,
        extra: Map::new(),
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::{FormatParser, FormatSerializer};

    const SAMPLE: &[u8] = include_bytes!("testdata/0001Stre.jsn");

    #[test]
    fn roundtrip_name() {
        let fmt = JetiFormat::default();
        let schema = fmt.parse(SAMPLE).unwrap();
        let ir = fmt.to_ir(schema).unwrap();
        let out_schema = fmt.from_ir(&ir).unwrap();
        let bytes = fmt.serialize(&out_schema).unwrap();
        let schema2 = fmt.parse(&bytes).unwrap();
        assert_eq!(schema2.global.name, "Stream NXT");
    }

    #[test]
    fn from_ir_minimal() {
        let ir = ModelIr {
            meta: ModelMeta {
                name: "TestModel".into(),
                firmware_origin: FirmwareOrigin::JetiDuplex,
                notes: None,
            },
            channels: vec![Channel {
                index: 0,
                name: None,
                min: Microseconds(1000),
                max: Microseconds(2000),
                center: Microseconds(1500),
                reversed: false,
            }],
            mixes: vec![],
            curves: vec![],
            rf_modules: vec![],
            telemetry: vec![],
            logic_switches: vec![],
            special_functions: vec![],
            timer: None,
            flight_modes: vec![],
            expo_settings: vec![],
        };

        let fmt = JetiFormat::default();
        let schema = fmt.from_ir(&ir).unwrap();
        assert_eq!(schema.global.name, "TestModel");
        let servos = schema.servos.unwrap();
        assert_eq!(servos.data.len(), 1);
        assert_eq!(servos.data[0].middle, 0);
        assert_eq!(servos.data[0].max_positive, 100);
        assert_eq!(servos.data[0].max_negative, -100);
    }
}
