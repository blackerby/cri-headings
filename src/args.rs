use crate::utils::current_year;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Get Congressional Record Index headings from the GovInfo API
pub struct Args {
    /// CRI years to download. Default to current year.
    #[arg(default_values_t = [current_year()])]
    pub years: Vec<String>,

    /// API page size
    #[arg(default_value = "1000")]
    #[arg(long)]
    pub page_size: String,

    /// Output directory
    #[arg(default_value = ".")]
    #[arg(long)]
    pub output_dir: String,

    /// GovInfo API Key
    #[arg(default_value = "DEMO_KEY")]
    #[arg(long)]
    pub api_key: String,

    /// Write CSV
    #[arg(short('c'), long("csv"))]
    pub csv: bool,
}
