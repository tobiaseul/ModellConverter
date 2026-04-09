use anyhow::Result;
use clap::Parser;

mod cli;
use cli::{Cli, Commands, RevEngTool};

use modell_converter::convert;
use modell_converter::reveng;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    }

    match cli.command {
        Commands::Convert { from, input, output } => {
            let to = modell_converter::format::Format::Edgetx;
            let is_batch = input.is_dir()
                || input.extension().and_then(|e| e.to_str()) == Some("zip");
            if is_batch {
                let out_dir = match output {
                    Some(p) => p,
                    None => {
                        let stem = input.file_stem().unwrap_or_default();
                        let mut name = stem.to_os_string();
                        name.push("_converted");
                        input.parent().unwrap_or(std::path::Path::new(".")).join(name)
                    }
                };
                convert::run_batch(from.into(), to, &input, &out_dir)?;
            } else {
                convert::run(from.into(), to, &input, output.as_deref())?;
            }
        }
        Commands::ReverseEng { tool } => match tool {
            RevEngTool::Hexdump { file, offset, len, width } => {
                reveng::hexdump::run(&file, offset, len, width)?;
            }
            RevEngTool::Diff { file_a, file_b, context } => {
                reveng::diff::run(&file_a, &file_b, context)?;
            }
        },
    }

    Ok(())
}
