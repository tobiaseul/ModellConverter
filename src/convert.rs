use std::io::Read as _;
use std::path::Path;
use anyhow::{Context, Result};

use crate::format::Format;
use crate::formats::edgetx::EdgeTxFormat;
use crate::formats::ethos::EthosFormat;
use crate::formats::jeti::JetiFormat;
use crate::formats::{FormatParser, FormatSerializer};

fn format_extension(fmt: &Format) -> &'static str {
    match fmt {
        Format::Edgetx => "yml",
        Format::Ethos => "bin",
        Format::JetiDuplex => "jsn",
    }
}

pub fn run(from: Format, to: Format, input: &Path, output: Option<&Path>) -> Result<()> {
    let input_bytes = std::fs::read(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    let output_bytes = convert(&input_bytes, &from, &to)?;

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => input.with_extension(format_extension(&to)),
    };

    std::fs::write(&out_path, &output_bytes)
        .with_context(|| format!("Failed to write output file: {}", out_path.display()))?;

    println!("Converted: {} → {}", input.display(), out_path.display());
    println!("\n⚠️  WARNING: Always verify the converted file on your transmitter before flying!");
    println!("The author is NOT responsible for any errors, data loss, or damage caused by conversion.\n");
    Ok(())
}

/// Run batch conversion and print a summary. Used by the CLI.
pub fn run_batch(from: Format, to: Format, input: &Path, output: &Path) -> Result<()> {
    let (converted, errors) = batch(from, to, input, output)?;
    println!("\nBatch complete: {} converted, {} errors", converted, errors);
    if converted > 0 {
        println!("⚠️  WARNING: Always verify converted files on your transmitter before flying!");
    }
    Ok(())
}

/// Run batch conversion and return (converted, errors). Used by the GUI.
pub fn batch(from: Format, to: Format, input: &Path, output: &Path) -> Result<(usize, usize)> {
    std::fs::create_dir_all(output)
        .with_context(|| format!("Failed to create output directory: {}", output.display()))?;

    Ok(if input.is_dir() {
        batch_from_dir(&from, &to, input, output)
    } else {
        batch_from_zip(&from, &to, input, output)
    })
}

fn batch_from_dir(from: &Format, to: &Format, input: &Path, output: &Path) -> (usize, usize) {
    let ext = format_extension(from);
    let mut converted = 0;
    let mut errors = 0;

    for entry in walkdir::WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some(ext) {
            continue;
        }
        let out_path = output.join(path.file_name().unwrap()).with_extension(format_extension(to));
        match convert_file(from, to, path, &out_path) {
            Ok(()) => { converted += 1; println!("  {} → {}", path.display(), out_path.display()); }
            Err(e) => { errors += 1; eprintln!("  ERROR {}: {}", path.display(), e); }
        }
    }
    (converted, errors)
}

fn batch_from_zip(from: &Format, to: &Format, input: &Path, output: &Path) -> (usize, usize) {
    let ext = format_extension(from);
    let mut converted = 0;
    let mut errors = 0;

    let file = match std::fs::File::open(input) {
        Ok(f) => f,
        Err(e) => { eprintln!("Failed to open zip: {}", e); return (0, 1); }
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => { eprintln!("Failed to read zip: {}", e); return (0, 1); }
    };

    for i in 0..archive.len() {
        let mut zip_file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => { errors += 1; eprintln!("  ERROR reading zip entry {}: {}", i, e); continue; }
        };

        let name = zip_file.name().to_string();
        let entry_path = Path::new(&name);
        if entry_path.extension().and_then(|e| e.to_str()) != Some(ext) {
            continue;
        }

        let mut bytes = Vec::new();
        if let Err(e) = zip_file.read_to_end(&mut bytes) {
            errors += 1;
            eprintln!("  ERROR reading {}: {}", name, e);
            continue;
        }

        let stem = entry_path.file_stem().unwrap_or_default();
        let out_path = output.join(stem).with_extension(format_extension(to));

        match convert(&bytes, from, to) {
            Ok(out_bytes) => {
                match std::fs::write(&out_path, &out_bytes) {
                    Ok(()) => { converted += 1; println!("  {} → {}", name, out_path.display()); }
                    Err(e) => { errors += 1; eprintln!("  ERROR writing {}: {}", out_path.display(), e); }
                }
            }
            Err(e) => { errors += 1; eprintln!("  ERROR converting {}: {}", name, e); }
        }
    }
    (converted, errors)
}

fn convert_file(from: &Format, to: &Format, input: &Path, output: &Path) -> Result<()> {
    let bytes = std::fs::read(input)
        .with_context(|| format!("Failed to read {}", input.display()))?;
    let out_bytes = convert(&bytes, from, to)?;
    std::fs::write(output, &out_bytes)
        .with_context(|| format!("Failed to write {}", output.display()))
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
        Format::JetiDuplex => {
            let parser = JetiFormat::default();
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
        Format::JetiDuplex => {
            let ser = JetiFormat::default();
            let schema = ser.from_ir(&ir)?;
            ser.serialize(&schema)?
        }
    };

    Ok(output)
}
