use std::path::Path;
use anyhow::{Context, Result};

use crate::format::Format;
use crate::formats::edgetx::EdgeTxFormat;
use crate::formats::ethos::EthosFormat;
use crate::formats::{FormatParser, FormatSerializer};

pub fn run(from: Format, to: Format, input: &Path, output: Option<&Path>) -> Result<()> {
    let input_bytes = std::fs::read(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    let output_bytes = convert(&input_bytes, &from, &to)?;

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let ext = match to {
                Format::Edgetx => "yml",
                Format::Ethos => "bin",
            };
            input.with_extension(ext)
        }
    };

    std::fs::write(&out_path, &output_bytes)
        .with_context(|| format!("Failed to write output file: {}", out_path.display()))?;

    println!("Converted: {} → {}", input.display(), out_path.display());
    Ok(())
}

pub fn convert(input_bytes: &[u8], from: &Format, to: &Format) -> Result<Vec<u8>> {
    let ir = match from {
        Format::Edgetx => {
            let parser = EdgeTxFormat::default();
            let schema = parser.parse(input_bytes)?;
            parser.to_ir(schema)?
        }
        Format::Ethos => {
            let parser = EthosFormat::default();
            let schema = parser.parse(input_bytes)?;
            parser.to_ir(schema)?
        }
    };

    let output = match to {
        Format::Edgetx => {
            let ser = EdgeTxFormat::default();
            let schema = ser.from_ir(&ir)?;
            ser.serialize(&schema)?
        }
        Format::Ethos => {
            let ser = EthosFormat::default();
            let schema = ser.from_ir(&ir)?;
            ser.serialize(&schema)?
        }
    };

    Ok(output)
}
