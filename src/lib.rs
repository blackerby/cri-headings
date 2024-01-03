mod api;
pub mod args;
mod constants;

use crate::api::Page;
use crate::args::Args;
use crate::constants::BASE_URL;
use anyhow::{bail, Result};
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest;
use reqwest::{blocking::Response as BlockingResponse, Response, StatusCode};
use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::Arc,
};
use time::OffsetDateTime;
use tokio::task::JoinHandle;

pub fn build_url(year: &String, args: Arc<Args>) -> (String, String) {
    (
        year.to_string(),
        format!(
            "{}{}/granules?offsetMark=*&pageSize={}&api_key={}",
            BASE_URL, year, args.page_size, args.api_key
        ),
    )
}

pub fn current_year() -> String {
    format!("{}", OffsetDateTime::now_utc().year())
}

pub fn is_rate_limited(response: &Response) -> bool {
    response.status() == StatusCode::TOO_MANY_REQUESTS
}

pub fn is_rate_limited_blocking(response: &BlockingResponse) -> bool {
    response.status() == StatusCode::TOO_MANY_REQUESTS
}

pub fn remaining_requests(response: &Response) -> Result<u16> {
    Ok(response
        .headers()
        .get("x-ratelimit-remaining")
        .expect("No matching header found")
        .to_str()?
        .parse::<u16>()?)
}

pub fn remaining_requests_blocking(response: &BlockingResponse) -> Result<u16> {
    Ok(response
        .headers()
        .get("x-ratelimit-remaining")
        .expect("No matching header found")
        .to_str()?
        .parse::<u16>()?)
}

pub fn requests_to_make(page: &Page) -> u16 {
    let quotient = page.count / page.page_size;
    match page.count % page.page_size {
        0 => quotient,
        _ => quotient + 1,
    }
}

pub fn blocking_run(args: Arc<Args>) -> Result<()> {
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

pub async fn async_run(args: Arc<Args>) -> Result<()> {
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
                let output_filename = format!("{}/CRI-{}_headings.txt", args.output_dir, year);
                let output_file = File::create(output_filename)?;
                let mut buf = BufWriter::new(output_file);
                let bar = ProgressBar::new(page.count as u64).with_message(format!("CRI-{}", year));
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

#[cfg(test)]
mod test {
    use crate::{current_year, requests_to_make, Page};

    #[test]
    pub fn test_requests_to_make_1000() {
        let page = Page {
            count: 14853,
            page_size: 1000,
            next_page: Some(String::new()),
            granules: Vec::new(),
        };
        let expected = 15;
        let result = requests_to_make(&page);
        assert_eq!(expected, result);
    }

    #[test]
    pub fn test_requests_to_make_100() {
        let page = Page {
            count: 14853,
            page_size: 100,
            next_page: Some(String::new()),
            granules: Vec::new(),
        };
        let expected = 149;
        let result = requests_to_make(&page);
        assert_eq!(expected, result);
    }

    #[test]
    pub fn test_requests_to_make_10() {
        let page = Page {
            count: 14853,
            page_size: 10,
            next_page: Some(String::new()),
            granules: Vec::new(),
        };
        let expected = 1486;
        let result = requests_to_make(&page);
        assert_eq!(expected, result);
    }

    #[test]
    pub fn test_current_year() {
        let expected = String::from("2024");
        let result = current_year();
        assert_eq!(expected, result)
    }
}
