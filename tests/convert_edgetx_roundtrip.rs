use modell_converter::formats::edgetx::EdgeTxFormat;
use modell_converter::formats::{FormatParser, FormatSerializer};

#[test]
fn edgetx_roundtrip_preserves_model_name() {
    let input = include_bytes!("fixtures/sample_edgetx.yml");
    let format = EdgeTxFormat::default();

    let schema = format.parse(input).expect("parse failed");
    let ir = format.to_ir(schema).expect("to_ir failed");

    assert_eq!(ir.meta.name, "SampleModel");

    let out_schema = format.from_ir(&ir).expect("from_ir failed");
    let out_bytes = format.serialize(&out_schema).expect("serialize failed");

    let schema2 = format.parse(&out_bytes).expect("re-parse failed");
    let ir2 = format.to_ir(schema2).expect("re-to_ir failed");

    assert_eq!(ir.meta.name, ir2.meta.name);
    assert_eq!(ir.channels.len(), ir2.channels.len());
    assert_eq!(ir.mixes.len(), ir2.mixes.len());
}

#[test]
fn edgetx_roundtrip_preserves_channels() {
    let input = include_bytes!("fixtures/sample_edgetx.yml");
    let format = EdgeTxFormat::default();

    let schema = format.parse(input).expect("parse failed");
    let ir = format.to_ir(schema).expect("to_ir failed");

    assert_eq!(ir.channels.len(), 4);
    assert_eq!(ir.channels[0].name.as_deref(), Some("Ail"));
    assert!(!ir.channels[0].reversed);

    let out_schema = format.from_ir(&ir).expect("from_ir failed");
    let out_bytes = format.serialize(&out_schema).expect("serialize failed");
    let schema2 = format.parse(&out_bytes).expect("re-parse failed");
    let ir2 = format.to_ir(schema2).expect("re-to_ir failed");

    for (a, b) in ir.channels.iter().zip(ir2.channels.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.min, b.min);
        assert_eq!(a.max, b.max);
        assert_eq!(a.reversed, b.reversed);
    }
}
