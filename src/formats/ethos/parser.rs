use binrw::BinReaderExt;
use std::io::Cursor;

use crate::error::ConversionError;
use crate::formats::FormatParser;
use crate::ir::model::*;
use super::EthosFormat;
use super::schema::EthosModel;

impl FormatParser for EthosFormat {
    type Schema = EthosModel;

    fn parse(&self, input: &[u8]) -> Result<EthosModel, ConversionError> {
        let mut cursor = Cursor::new(input);
        cursor.read_le::<EthosModel>().map_err(|e| ConversionError::EthosParse {
            offset: 0,
            message: e.to_string(),
        })
    }

    fn to_ir(&self, schema: EthosModel) -> Result<ModelIr, ConversionError> {
        // Format is not yet reverse engineered.
        // As offsets are confirmed, extract fields from schema.raw here.
        let _ = schema;
        Ok(ModelIr {
            meta: ModelMeta {
                name: String::new(), // TODO: extract from confirmed offset
                firmware_origin: FirmwareOrigin::Ethos,
                notes: None,
            },
            channels: vec![],          // TODO: pending RE
            mixes: vec![],             // TODO: pending RE
            curves: vec![],            // TODO: pending RE
            rf_modules: vec![],        // TODO: pending RE
            telemetry: vec![],         // TODO: pending RE
            logic_switches: vec![],    // TODO: pending RE
            special_functions: vec![], // TODO: pending RE
            timer: None,               // TODO: pending RE
        })
    }
}
