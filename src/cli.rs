use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Input directory for ZIP archives
    #[arg(default_value = ".")]
    pub input: PathBuf,
    /// Output directory for tarballs (.tar.zst)
    #[arg(short, long, default_value = "./out")]
    pub output: PathBuf,
}
