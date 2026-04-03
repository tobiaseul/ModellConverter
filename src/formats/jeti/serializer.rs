use crate::error::ConversionError;
use crate::formats::FormatSerializer;
use crate::ir::model::ModelIr;
use super::JetiFormat;
use super::schema::JetiModel;

impl FormatSerializer for JetiFormat {
    type Schema = JetiModel;

    fn from_ir(&self, ir: &ModelIr) -> Result<JetiModel, ConversionError> {
        // Fields are mapped here incrementally as .jsn files are analyzed.
        Ok(JetiModel {
            name: ir.meta.name.clone(),
            extra: Default::default(),
        })
    }

    fn serialize(&self, schema: &JetiModel) -> Result<Vec<u8>, ConversionError> {
        serde_json::to_vec_pretty(schema)
            .map_err(|e| ConversionError::JetiParse(e.to_string()))
    }
}
