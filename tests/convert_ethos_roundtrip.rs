use modell_converter::formats::ethos::EthosFormat;
use modell_converter::formats::FormatParser;

#[test]
fn ethos_parse_arbitrary_bytes_does_not_panic() {
    let data: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02];
    let format = EthosFormat::default();
    let schema = format.parse(data).expect("parse failed");
    let ir = format.to_ir(schema).expect("to_ir failed");
    // Until RE is complete, IR should be a valid empty skeleton
    assert_eq!(ir.meta.firmware_origin, modell_converter::ir::model::FirmwareOrigin::Ethos);
    assert!(ir.channels.is_empty());
    assert!(ir.mixes.is_empty());
}
