use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "modell-converter",
    version,
    about = "Multi-format RC model converter: EdgeTX ↔ Ethos ↔ Jeti Duplex"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true, help = "Enable verbose logging")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert a model file between firmware formats
    Convert {
        #[arg(long, value_enum)]
        from: CliFormat,
        #[arg(long, value_enum)]
        to: CliFormat,
        /// Input file path
        input: PathBuf,
        /// Output file path (defaults to <input> with new extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Reverse engineering analysis tools for the Ethos binary format
    ReverseEng {
        #[command(subcommand)]
        tool: RevEngTool,
    },
}

#[derive(Subcommand)]
pub enum RevEngTool {
    /// Pretty hex dump of a binary file
    Hexdump {
        file: PathBuf,
        #[arg(long, default_value = "0")]
        offset: u64,
        #[arg(long)]
        len: Option<usize>,
        #[arg(long, default_value = "16")]
        width: usize,
    },
    /// Byte-level diff of two .bin files, highlighting differences
    Diff {
        file_a: PathBuf,
        file_b: PathBuf,
        /// Show N bytes of context around differing regions
        #[arg(long)]
        context: Option<usize>,
    },
}

/// CLI-facing format enum (implements ValueEnum for clap).
/// Converts to the library's `Format` type before passing to core logic.
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum CliFormat {
    #[value(alias = "edgetx")]
    Edgetx,
    #[value(alias = "ethos")]
    Ethos,
    #[value(alias = "jeti")]
    JetiDuplex,
}

impl From<CliFormat> for modell_converter::format::Format {
    fn from(f: CliFormat) -> Self {
        match f {
            CliFormat::Edgetx => modell_converter::format::Format::Edgetx,
            CliFormat::Ethos => modell_converter::format::Format::Ethos,
            CliFormat::JetiDuplex => modell_converter::format::Format::JetiDuplex,
        }
    }
}
