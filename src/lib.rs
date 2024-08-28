mod cli;
mod converter;
mod error;
mod model;

use clap::Parser;
use cli::Cli;
use converter::Converter;

pub async fn main_impl() -> anyhow::Result<()> {
    let Cli {
        input: input_path,
        output: output_path,
    } = Cli::parse();

    let converter = Converter::with_input_path(&input_path).await?;

    converter
        .rename()
        .await?
        .convert()
        .await?
        .tar(&output_path)
        .await?;

    Ok(())
}
