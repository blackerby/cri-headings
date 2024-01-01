mod api;
mod constants;
mod utils;

use crate::api::Page;
use crate::utils::*;
use anyhow::{bail, Result};
use clap::Parser;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar};
use reqwest;
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use tokio::task::JoinHandle;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Get Congressional Record Index headings from the GovInfo API
pub struct Args {
    /// CRI years to download. Default to current year.
    #[arg(default_values_t = [current_year()])]
    years: Vec<String>,

    /// API page size
    #[arg(default_value = "1000")]
    page_size: String,

    /// GovInfo API Key
    #[arg(default_value = "DEMO_KEY", last(true))]
    api_key: String,
}

#[tokio::main]
pub async fn run(args: Args) -> Result<()> {
    let years = &args.years;
    let api_key = &args.api_key;
    let page_size = &args.page_size;

    let urls = years
        .iter()
        .map(|year| build_url(year, page_size, api_key))
        .collect::<Vec<(String, String, String)>>();

    let mut tasks: Vec<JoinHandle<Result<()>>> = Vec::new();
    let mp = MultiProgress::new();

    for (api_key, year, url) in urls {
        let url = url.clone();
        let mp_clone = mp.clone();

        tasks.push(tokio::spawn(async move {
            let response = reqwest::get(url).await?;

            if is_rate_limited(&response) {
                bail!("Status Code 429: Too many requests. Wait one hour.")
            }

            let remaining_requests = remaining_requests(&response)?;
            let page = response.json::<Page>().await?;

            if remaining_requests < requests_to_make(&page) {
                bail!("Not enough requests remaining to complete task. Wait one hour.")
            }

            let output_filename = format!("CRI-{}_headings.txt", year);
            let output_file = File::create(output_filename)?;
            let mut buf = BufWriter::new(output_file);
            let bar = ProgressBar::new(page.count as u64);
            let pb = mp_clone.add(bar);

            for granule in page.granules {
                writeln!(buf, "{}", granule.title)?;
                pb.inc(1);
            }

            let mut next_page = page.next_page;
            while let Some(base_url) = next_page {
                let next_url = format!("{}&api_key={}", base_url, api_key);
                let page = reqwest::get(next_url).await?.json::<Page>().await?;
                for granule in page.granules {
                    writeln!(buf, "{}", granule.title)?;
                    pb.inc(1);
                }
                next_page = page.next_page;
            }

            buf.flush()?;
            pb.finish();
            Ok(())
        }))
    }

    join_all(tasks).await;

    Ok(())
}
