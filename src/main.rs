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
        Commands::Convert { from, to, input, output } => {
            convert::run(from.into(), to.into(), &input, output.as_deref())?;
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
