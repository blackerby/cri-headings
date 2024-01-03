use anyhow::Result;
use clap::Parser;
use cri_headings::args::Args;
use cri_headings::{async_run, blocking_run};
use std::fs;
use std::sync::Arc;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Arc::new(Args::parse());
    fs::create_dir_all(&args.output_dir)?;

    match args.years.len() {
        1 => blocking_run(args.clone()),
        _ => async_run(args.clone()).await,
    }
}
