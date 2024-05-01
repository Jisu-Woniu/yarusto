mod cli;
mod converter;
mod model;

use clap::Parser;
use cli::Cli;
use converter::Converter;

pub fn main_impl() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let input_path = cli.input_path;
    let output_path = cli.output_path;

    let convertor = Converter::build(&input_path)?;

    Ok(())
}
