pub mod edgetx;
pub mod ethos;

use crate::error::ConversionError;
use crate::ir::model::ModelIr;

pub trait FormatParser {
    type Schema;
    fn parse(&self, input: &[u8]) -> Result<Self::Schema, ConversionError>;
    fn to_ir(&self, schema: Self::Schema) -> Result<ModelIr, ConversionError>;
}

pub trait FormatSerializer {
    type Schema;
    fn from_ir(&self, ir: &ModelIr) -> Result<Self::Schema, ConversionError>;
    fn serialize(&self, schema: &Self::Schema) -> Result<Vec<u8>, ConversionError>;
}
