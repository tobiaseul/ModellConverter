use crate::error::ConversionError;
use crate::formats::FormatParser;
use crate::ir::model::*;
use super::JetiFormat;
use super::schema::JetiModel;

impl FormatParser for JetiFormat {
    type Schema = JetiModel;

    fn parse(&self, input: &[u8]) -> Result<JetiModel, ConversionError> {
        serde_json::from_slice(input)
            .map_err(|e| ConversionError::JetiParse(e.to_string()))
    }

    fn to_ir(&self, schema: JetiModel) -> Result<ModelIr, ConversionError> {
        // Fields are mapped here incrementally as .jsn files are analyzed.
        Ok(ModelIr {
            meta: ModelMeta {
                name: schema.name,
                firmware_origin: FirmwareOrigin::Unknown,
                notes: None,
            },
            channels: vec![],          // TODO: pending .jsn analysis
            mixes: vec![],             // TODO: pending .jsn analysis
            curves: vec![],            // TODO: pending .jsn analysis
            rf_modules: vec![],        // TODO: pending .jsn analysis
            telemetry: vec![],         // TODO: pending .jsn analysis
            logic_switches: vec![],    // TODO: pending .jsn analysis
            special_functions: vec![], // TODO: pending .jsn analysis
            timer: None,               // TODO: pending .jsn analysis
        })
    }
}
