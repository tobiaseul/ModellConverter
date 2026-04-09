use crate::error::ConversionError;
use crate::formats::FormatParser;
use crate::ir::model::*;
use super::JetiFormat;
use super::schema::{JetiModel, JetiMixValue, JetiServo, JetiTelemSensor, JetiTimer};
use serde_json::Value;
use std::time::Duration;

impl FormatParser for JetiFormat {
    type Schema = JetiModel;

    fn parse(&self, input: &[u8]) -> Result<JetiModel, ConversionError> {
        let utf8 = decode_latin1(input);
        let sanitized = fix_lone_surrogates(&utf8);
        serde_json::from_slice(&sanitized)
            .map_err(|e| ConversionError::JetiParse(e.to_string()))
    }

    fn to_ir(&self, schema: JetiModel) -> Result<ModelIr, ConversionError> {
        let functions: Vec<_> = schema
            .functions
            .map(|s| s.data)
            .unwrap_or_default();

        let mix_values: Vec<JetiMixValue> = schema
            .mixes_values
            .unwrap_or_default();

        let mixes_main: Vec<Vec<Value>> = schema
            .mixes_main
            .map(|s| s.data)
            .unwrap_or_default();

        let mixes = mixes_main
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| {
                // entry: [servo_idx, func_idx, sub, ...]
                let servo_idx = entry.first()?.as_u64()? as u8;
                let func_idx = entry.get(1)?.as_u64()? as usize;
                let func = functions.iter().find(|f| f.id as usize == func_idx)?;
                // use first mix value for this mix (FM0)
                let fm0 = mix_values.get(i)?;
                Some(mix_to_ir(servo_idx, func.label.clone(), &func.control, fm0))
            })
            .collect();

        let channels = schema
            .servos
            .map(|s| s.data)
            .unwrap_or_default()
            .into_iter()
            .map(servo_to_channel)
            .collect();

        let telemetry = schema
            .telem_detect
            .map(|s| s.data)
            .unwrap_or_default()
            .into_iter()
            .map(sensor_to_ir)
            .collect();

        let timer = schema
            .timers
            .and_then(|s| s.data.into_iter().next())
            .map(timer_to_ir);

        Ok(ModelIr {
            meta: ModelMeta {
                name: schema.global.name,
                firmware_origin: FirmwareOrigin::JetiDuplex,
                notes: None,
            },
            channels,
            mixes,
            curves: vec![],
            rf_modules: vec![],
            telemetry,
            logic_switches: vec![],
            special_functions: vec![],
            timer,
        })
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Jeti .jsn files use latin-1 encoding; convert to UTF-8 before JSON parse.
pub(crate) fn decode_latin1(bytes: &[u8]) -> Vec<u8> {
    let mut utf8 = Vec::with_capacity(bytes.len() + bytes.len() / 8);
    for &b in bytes {
        if b < 0x80 {
            utf8.push(b);
        } else {
            // latin-1 U+0080..U+00FF → two-byte UTF-8
            utf8.push(0xC0 | (b >> 6));
            utf8.push(0x80 | (b & 0x3F));
        }
    }
    utf8
}

/// Replace lone UTF-16 surrogates (`\uD800`–`\uDFFF`) in JSON bytes with `\uFFFD`.
///
/// Jeti firmware sometimes emits lone surrogate escape sequences which are
/// invalid Unicode and rejected by serde_json.
fn fix_lone_surrogates(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        // Look for \uXXXX where XXXX is a surrogate (D800–DFFF)
        if i + 5 < bytes.len()
            && bytes[i] == b'\\'
            && bytes[i + 1] == b'u'
            && matches!(bytes[i + 2], b'd' | b'D')
            && matches!(bytes[i + 3], b'8'..=b'9' | b'a'..=b'b' | b'A'..=b'B')
        {
            out.extend_from_slice(b"\\uFFFD");
            i += 6;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

/// Jeti percent values: ±100 unit ≈ ±500 µs from 1500 µs center.
fn jeti_pct_to_us(pct: i32) -> Microseconds {
    Microseconds(1500 + pct * 5)
}

fn servo_to_channel(s: JetiServo) -> Channel {
    let center = jeti_pct_to_us(s.middle);
    Channel {
        index: s.index,
        name: None,
        center,
        max: Microseconds(center.0 + s.max_positive * 5),
        min: Microseconds(center.0 + s.max_negative * 5),
        reversed: s.reversed != 0,
    }
}

fn mix_to_ir(channel_out: u8, name: String, control: &str, fm: &JetiMixValue) -> Mix {
    let weight = Percent((fm.intensity * fm.direction) as f32);
    Mix {
        channel_out,
        name: Some(name),
        source: parse_jeti_control(control),
        weight,
        offset: Percent(0.0),
        curve: None,
        switch: None,
        mode: MixMode::Add,
    }
}

/// Parse a Jeti control string "axis,sub,..." into a MixSource.
/// Axis IDs: 0=none, 1=Ail, 2=Thr, 3=Ele, 4=Rud, 5=S1, 6=S2
fn parse_jeti_control(s: &str) -> MixSource {
    let axis: u32 = s
        .split(',')
        .next()
        .and_then(|t| t.parse().ok())
        .unwrap_or(0);
    match axis {
        1 => MixSource::Stick(StickAxis::Ail),
        2 => MixSource::Stick(StickAxis::Thr),
        3 => MixSource::Stick(StickAxis::Ele),
        4 => MixSource::Stick(StickAxis::Rud),
        5 => MixSource::Stick(StickAxis::S1),
        6 => MixSource::Stick(StickAxis::S2),
        _ => MixSource::Constant(Percent(0.0)),
    }
}

fn sensor_to_ir(s: JetiTelemSensor) -> TelemetrySensor {
    let unit = s.unit.trim().to_string();
    let ratio = if s.decimal > 0 {
        Some(10f32.powi(-(s.decimal as i32)))
    } else {
        None
    };
    TelemetrySensor {
        name: s.label,
        unit: if unit.is_empty() { None } else { Some(unit) },
        ratio,
        source: TelemetrySource::Physical,
    }
}

fn timer_to_ir(t: JetiTimer) -> Timer {
    Timer {
        name: Some(t.label),
        mode: if t.mode == 0 { TimerMode::Absolute } else { TimerMode::Running },
        start: Duration::from_secs(t.start as u64),
        countdown: t.mode != 0,
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FormatParser;

    const SAMPLE: &[u8] = include_bytes!("testdata/0001Stre.jsn");

    #[test]
    fn decode_latin1_passthrough_ascii() {
        let input = b"hello world";
        assert_eq!(decode_latin1(input), input.to_vec());
    }

    #[test]
    fn decode_latin1_degree_sign() {
        // 0xB0 (°) in latin-1 → UTF-8: 0xC2, 0xB0
        let input = &[0xB0u8];
        let out = decode_latin1(input);
        assert_eq!(out, vec![0xC2, 0xB0]);
        assert_eq!(std::str::from_utf8(&out).unwrap(), "°");
    }

    #[test]
    fn fix_lone_surrogates_replaces_surrogate() {
        let input = br#"{"x":"\uD83D"}"#;
        let fixed = fix_lone_surrogates(input);
        assert!(std::str::from_utf8(&fixed).unwrap().contains("\\uFFFD"));
        // must parse without error
        let v: serde_json::Value = serde_json::from_slice(&fixed).unwrap();
        assert!(v["x"].as_str().unwrap().contains('\u{FFFD}'));
    }

    #[test]
    fn fix_lone_surrogates_leaves_normal_escapes() {
        let input = br#"{"x":"\u00B0"}"#;
        let fixed = fix_lone_surrogates(input);
        assert_eq!(input.to_vec(), fixed);
    }

    #[test]
    fn parse_file_with_lone_surrogate() {
        // Simulate a Jeti file that has a lone surrogate in a label field
        let json = br#"{"Global":{"Name":"Test","Version":680,"TxVers":"5.08","Model-Type":1,"Filename":"","Desc":""},"Labels":["\uD800bad"]}"#;
        let fmt = JetiFormat::default();
        let schema = fmt.parse(json).expect("should not fail on lone surrogate");
        assert_eq!(schema.global.name, "Test");
    }

    #[test]
    fn parse_real_file_name() {
        let fmt = JetiFormat::default();
        let schema = fmt.parse(SAMPLE).expect("parse failed");
        assert_eq!(schema.global.name, "Stream NXT");
    }

    #[test]
    fn parse_real_file_to_ir() {
        let fmt = JetiFormat::default();
        let schema = fmt.parse(SAMPLE).expect("parse failed");
        let ir = fmt.to_ir(schema).expect("to_ir failed");
        assert_eq!(ir.meta.name, "Stream NXT");
        assert_eq!(ir.meta.firmware_origin, FirmwareOrigin::JetiDuplex);
        assert!(!ir.channels.is_empty(), "channels should be populated");
        assert!(!ir.telemetry.is_empty(), "telemetry should be populated");
    }

    #[test]
    fn parse_jeti_control_axes() {
        assert_eq!(parse_jeti_control("1,0,1,1,1,0,-1,0"), MixSource::Stick(StickAxis::Ail));
        assert_eq!(parse_jeti_control("2,0,1,1,1,0,-1,0"), MixSource::Stick(StickAxis::Thr));
        assert_eq!(parse_jeti_control("3,0,1,1,1,0,-1,0"), MixSource::Stick(StickAxis::Ele));
        assert_eq!(parse_jeti_control("4,0,1,1,1,0,-1,0"), MixSource::Stick(StickAxis::Rud));
        assert_eq!(parse_jeti_control("0,0,0,0,1,4000,-1,0"), MixSource::Constant(Percent(0.0)));
    }

    #[test]
    fn servo_microsecond_conversion() {
        // middle=0, max_positive=100, max_negative=-100 → standard ±500 µs servo
        let servo = crate::formats::jeti::schema::JetiServo {
            index: 0,
            middle: 0,
            max_positive: 100,
            max_negative: -100,
            reversed: 0,
            delay_positive: 0,
            delay_negative: 0,
            curve: vec![],
            extra: Default::default(),
        };
        let ch = servo_to_channel(servo);
        assert_eq!(ch.center, Microseconds(1500));
        assert_eq!(ch.max, Microseconds(2000));
        assert_eq!(ch.min, Microseconds(1000));
    }
}
