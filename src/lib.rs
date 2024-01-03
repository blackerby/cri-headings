mod api;
pub mod args;
mod constants;
mod utils;

use crate::api::Page;
use crate::args::Args;
use crate::utils::*;
use anyhow::{bail, Result};
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    sync::Arc,
};
use tokio::task::JoinHandle;

#[tokio::main]
pub async fn run(args: Args) -> Result<()> {
    let args = Arc::new(args);
    fs::create_dir_all(&args.output_dir)?;

    match args.years.len() {
        1 => {
            let (year, url) = build_url(&args.years[0], args.clone());
            let response = reqwest::blocking::get(url)?;

            if is_rate_limited_blocking(&response) {
                bail!("Status Code 429: Too many requests. Wait one hour.")
            }

            let remaining_requests = remaining_requests_blocking(&response)?;
            let page = response.json::<Page>()?;

            if remaining_requests < requests_to_make(&page) {
                bail!("Not enough requests remaining to complete task. Wait one hour.")
            }

            if page.count > 0 {
                let output_filename = format!("{}/CRI-{}_headings.txt", args.output_dir, year);
                let output_file = File::create(output_filename)?;
                let mut buf = BufWriter::new(output_file);
                let bar = ProgressBar::new(page.count as u64).with_message(format!("CRI-{}", year));
                bar.set_style(ProgressStyle::with_template(
                    "{wide_bar} {msg} ({pos}/{len} headings)",
                )?);

                for granule in page.granules {
                    writeln!(buf, "{}", granule.title)?;
                    bar.inc(1);
                }

                let mut next_page = page.next_page;
                while let Some(base_url) = next_page {
                    let next_url = format!("{}&api_key={}", base_url, args.api_key);
                    let page = reqwest::blocking::get(next_url)?.json::<Page>()?;
                    for granule in page.granules {
                        writeln!(buf, "{}", granule.title)?;
                        bar.inc(1);
                    }
                    next_page = page.next_page;
                }
                buf.flush()?;
                bar.finish_and_clear();
            } else {
                println!("No CRI entries for {}", year);
            }
            Ok(())
        }
        _ => {
            let urls = args
                .years
                .iter()
                .map(|year| build_url(year, args.clone()))
                .collect::<Vec<(String, String)>>();

            let mut tasks: Vec<JoinHandle<Result<()>>> = Vec::new();
            let mp = MultiProgress::new();

            for (year, url) in urls {
                let url = url.clone();
                let mp_clone = mp.clone();
                let args = args.clone();

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

                    if page.count > 0 {
                        let output_filename =
                            format!("{}/CRI-{}_headings.txt", args.output_dir, year);
                        let output_file = File::create(output_filename)?;
                        let mut buf = BufWriter::new(output_file);
                        let bar = ProgressBar::new(page.count as u64)
                            .with_message(format!("CRI-{}", year));
                        bar.set_style(ProgressStyle::with_template(
                            "{wide_bar} {msg} ({pos}/{len} headings)",
                        )?);
                        let pb = mp_clone.add(bar);

                        for granule in page.granules {
                            writeln!(buf, "{}", granule.title)?;
                            pb.inc(1);
                        }

                        let mut next_page = page.next_page;
                        while let Some(base_url) = next_page {
                            let next_url = format!("{}&api_key={}", base_url, args.api_key);
                            let page = reqwest::get(next_url).await?.json::<Page>().await?;
                            for granule in page.granules {
                                writeln!(buf, "{}", granule.title)?;
                                pb.inc(1);
                            }
                            next_page = page.next_page;
                        }
                        buf.flush()?;
                        pb.finish_and_clear();
                    } else {
                        println!("No CRI entries for {}", year);
                    }
                    Ok(())
                }))
            }

            join_all(tasks).await;

            Ok(())
        }
    }
}
