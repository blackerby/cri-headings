use clap::Parser;
use reqwest;
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
};

const BASE_URL: &str = "https://api.govinfo.gov/packages/CRI-";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page {
    count: u16,
    next_page: Option<String>,
    granules: Vec<Granule>,
}

#[derive(Deserialize)]
struct Granule {
    title: String,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Get Congressional Record Index headings from the GovInfo API
pub struct Args {
    /// CRI years to download. Default to current year.
    #[arg(default_value = "2023")] // how to make this current year?
    years: Vec<String>,

    /// API offset
    #[arg(default_value = "0")]
    offset: String,

    /// API page size
    #[arg(default_value = "1000")]
    page_size: String,

    /// GovInfo API Key
    #[arg(default_value = "DEMO_KEY", last(true))]
    api_key: String,
}

#[tokio::main]
pub async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    for year in args.years {
        let url = build_url(&year, &args.offset, &args.page_size, &args.api_key);
        let page = reqwest::get(url).await?.json::<Page>().await?;
        let granules = page.granules;

        let output_filename = format!("CRI-{}_headings.txt", &year);
        let output_file = File::create(output_filename)?;
        let mut buf = BufWriter::new(output_file);
        let mut count = 0;
        count += granules.len();

        for granule in granules {
            writeln!(buf, "{}", granule.title)?;
        }
        println!("{}/{} headings written", count, page.count);

        let mut next_page = page.next_page;
        while let Some(base_url) = next_page {
            let next_url = format!("{}&api_key={}", base_url, args.api_key);
            let page = reqwest::get(next_url).await?.json::<Page>().await?;
            let granules = page.granules;
            count += granules.len();
            for granule in granules {
                writeln!(buf, "{}", granule.title)?;
            }
            println!("{}/{} headings written", count, page.count);
            next_page = page.next_page;
        }
    }
    Ok(())
}

fn build_url(year: &String, offset: &String, page_size: &String, api_key: &String) -> String {
    format!(
        "{}{}/granules?offset={}&pageSize={}&api_key={}",
        BASE_URL, year, offset, page_size, api_key
    )
}
