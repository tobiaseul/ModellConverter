use modell_converter::formats::edgetx::EdgeTxFormat;
use modell_converter::formats::{FormatParser, FormatSerializer};
use modell_converter::ir::model::{MixMode, MixSource, Microseconds, StickAxis};

fn parse_fixture() -> modell_converter::ir::model::ModelIr {
    let input = include_bytes!("fixtures/sample_edgetx.yml");
    let fmt = EdgeTxFormat::default();
    let schema = fmt.parse(input).expect("parse failed");
    fmt.to_ir(schema).expect("to_ir failed")
}

fn roundtrip(ir: &modell_converter::ir::model::ModelIr) -> modell_converter::ir::model::ModelIr {
    let fmt = EdgeTxFormat::default();
    let schema = fmt.from_ir(ir).expect("from_ir failed");
    let bytes = fmt.serialize(&schema).expect("serialize failed");
    let schema2 = fmt.parse(&bytes).expect("re-parse failed");
    fmt.to_ir(schema2).expect("re-to_ir failed")
}

#[test]
fn roundtrip_model_name() {
    let ir = parse_fixture();
    assert_eq!(ir.meta.name, "SampleModel");
    assert_eq!(roundtrip(&ir).meta.name, "SampleModel");
}

#[test]
fn roundtrip_channels() {
    let ir = parse_fixture();
    assert_eq!(ir.channels.len(), 4);

    let ch = &ir.channels[0];
    assert_eq!(ch.name.as_deref(), Some("Ail"));
    assert_eq!(ch.min, Microseconds(-1000));
    assert_eq!(ch.max, Microseconds(1000));
    assert_eq!(ch.center, Microseconds(0));
    assert!(!ch.reversed);

    let ir2 = roundtrip(&ir);
    for (a, b) in ir.channels.iter().zip(ir2.channels.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.min, b.min);
        assert_eq!(a.max, b.max);
        assert_eq!(a.center, b.center);
        assert_eq!(a.reversed, b.reversed);
    }
}

#[test]
fn roundtrip_mixes() {
    let ir = parse_fixture();
    assert_eq!(ir.mixes.len(), 4);

    // First mix: Ail stick → channel 0, weight 100, add mode
    let m = &ir.mixes[0];
    assert_eq!(m.channel_out, 0);
    assert_eq!(m.source, MixSource::Stick(StickAxis::Ail));
    assert_eq!(m.weight.0, 100.0);
    assert_eq!(m.offset.0, 0.0);
    assert_eq!(m.mode, MixMode::Add);
    assert!(m.curve.is_none());
    assert!(m.switch.is_none());

    let ir2 = roundtrip(&ir);
    for (a, b) in ir.mixes.iter().zip(ir2.mixes.iter()) {
        assert_eq!(a.channel_out, b.channel_out);
        assert_eq!(a.name, b.name);
        assert_eq!(a.source, b.source);
        assert_eq!(a.weight.0, b.weight.0);
        assert_eq!(a.offset.0, b.offset.0);
        assert_eq!(a.mode, b.mode);
        assert_eq!(a.curve, b.curve);
    }
}

#[test]
fn roundtrip_rf_module() {
    let ir = parse_fixture();
    assert_eq!(ir.rf_modules.len(), 1);

    let m = &ir.rf_modules[0];
    assert_eq!(m.protocol, "MULTI");
    assert_eq!(*m.channel_range.start(), 0);
    assert_eq!(*m.channel_range.end(), 7);

    let ir2 = roundtrip(&ir);
    assert_eq!(ir.rf_modules[0].protocol, ir2.rf_modules[0].protocol);
    assert_eq!(ir.rf_modules[0].channel_range, ir2.rf_modules[0].channel_range);
}

#[test]
fn invalid_mix_mode_returns_error() {
    let bad_yaml = b"header:\n  name: Bad\nmixes:\n  - ch: 0\n    source: Ail\n    weight: 100\n    offset: 0\n    mode: unknown_mode\n";
    let fmt = EdgeTxFormat::default();
    let schema = fmt.parse(bad_yaml).expect("parse failed");
    let result = fmt.to_ir(schema);
    assert!(result.is_err(), "expected error for unknown mix mode");
}

#[test]
fn invalid_curve_ref_returns_error() {
    let bad_yaml = b"header:\n  name: Bad\nmixes:\n  - ch: 0\n    source: Ail\n    weight: 100\n    offset: 0\n    mode: add\n    curve: cvXYZ\n";
    let fmt = EdgeTxFormat::default();
    let schema = fmt.parse(bad_yaml).expect("parse failed");
    let result = fmt.to_ir(schema);
    assert!(result.is_err(), "expected error for invalid curve reference");
}
