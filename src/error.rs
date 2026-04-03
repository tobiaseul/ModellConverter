use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("EdgeTX YAML parse error: {0}")]
    EdgeTxParse(String),

    #[error("Ethos binary parse error at offset {offset:#x}: {message}")]
    EthosParse { offset: u64, message: String },

    #[error("Field '{field}' not yet supported for Ethos format (pending reverse engineering)")]
    EthosFieldNotSupported { field: &'static str },

    #[error("Jeti JSON parse error: {0}")]
    JetiParse(String),

    #[error("IR validation failed: {0}")]
    IrValidation(String),

    #[error("Conversion from {from} to {to} is not yet implemented")]
    ConversionNotImplemented { from: String, to: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
