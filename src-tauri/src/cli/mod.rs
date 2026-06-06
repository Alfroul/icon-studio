pub mod export;
pub mod analyze;
pub mod batch;
pub mod variation;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "iconstudio", version, about = "IconStudio CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Export an .iconproject.json to SVG/PNG/ICO
    Export(export::ExportArgs),
    /// Analyze color usage and design consistency
    Analyze(analyze::AnalyzeArgs),
    /// Batch export all .iconproject.json files in a directory
    Batch(batch::BatchArgs),
    /// Generate variations from an .iconproject.json with transforms
    Variations(variation::VariationArgs),
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Export(args)) => export::run(&args)?,
        Some(Commands::Analyze(args)) => analyze::run(&args)?,
        Some(Commands::Batch(args)) => batch::run(&args)?,
        Some(Commands::Variations(args)) => variation::run(&args)?,
        None => {
            Cli::parse_from(["iconstudio", "--help"]);
        }
    }
    Ok(())
}
