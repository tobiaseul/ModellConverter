use modell_converter::formats::jeti::JetiFormat;
use modell_converter::formats::{FormatParser, FormatSerializer};
use modell_converter::ir::model::FirmwareOrigin;

#[test]
fn jeti_parse_does_not_panic() {
    let input = include_bytes!("fixtures/sample_jeti.jsn");
    let fmt = JetiFormat::default();
    let schema = fmt.parse(input).expect("parse failed");
    assert_eq!(schema.name, "SampleJetiModel");
}

#[test]
fn jeti_roundtrip_preserves_name() {
    let input = include_bytes!("fixtures/sample_jeti.jsn");
    let fmt = JetiFormat::default();

    let schema = fmt.parse(input).expect("parse failed");
    let ir = fmt.to_ir(schema).expect("to_ir failed");
    assert_eq!(ir.meta.name, "SampleJetiModel");
    assert_eq!(ir.meta.firmware_origin, FirmwareOrigin::Unknown);

    let out_schema = fmt.from_ir(&ir).expect("from_ir failed");
    let out_bytes = fmt.serialize(&out_schema).expect("serialize failed");

    let schema2 = fmt.parse(&out_bytes).expect("re-parse failed");
    let ir2 = fmt.to_ir(schema2).expect("re-to_ir failed");
    assert_eq!(ir.meta.name, ir2.meta.name);
}

#[test]
fn jeti_unknown_fields_preserved_in_schema() {
    let input = include_bytes!("fixtures/sample_jeti.jsn");
    let fmt = JetiFormat::default();
    let schema = fmt.parse(input).expect("parse failed");
    assert!(schema.extra.contains_key("unknownFutureField"),
        "unknown fields should be preserved in extra");
}

#[test]
fn jeti_invalid_json_returns_error() {
    let bad = b"not valid json {{";
    let fmt = JetiFormat::default();
    assert!(fmt.parse(bad).is_err(), "expected error for invalid JSON");
}
