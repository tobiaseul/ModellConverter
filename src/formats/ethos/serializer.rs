use crate::error::ConversionError;
use crate::formats::FormatSerializer;
use crate::ir::model::ModelIr;
use super::EthosFormat;
use super::schema::EthosModel;

impl FormatSerializer for EthosFormat {
    type Schema = EthosModel;

    fn from_ir(&self, _ir: &ModelIr) -> Result<EthosModel, ConversionError> {
        // Cannot serialize to Ethos until the binary format is reverse engineered.
        Err(ConversionError::ConversionNotImplemented {
            from: "IR".to_string(),
            to: "Ethos".to_string(),
        })
    }

    fn serialize(&self, schema: &EthosModel) -> Result<Vec<u8>, ConversionError> {
        Ok(schema.raw.clone())
    }
}
