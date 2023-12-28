use anyhow::Result;
use clap::Parser;
use reqwest;
use scraper::{Html, Selector};
use std::{
    fs::File,
    io::{BufWriter, Write},
};

const BASE_URL: &str = "https://api.govinfo.gov/packages/CRI-";

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Get Congressional Record Index headings from the GovInfo API
pub struct Args {
    /// CRI years to download. Default to current year.
    #[arg(default_value = "2023")] // how to make this current year?
    years: Vec<String>,

    /// GovInfo API Key
    #[arg(default_value = "DEMO_KEY")]
    api_key: String,

    /// Data format
    #[arg(default_value = "htm")]
    format: String,
}

#[tokio::main]
pub async fn run(args: Args) -> Result<()> {
    for year in args.years {
        let url = build_url(&year, &args.format, &args.api_key);
        let mut response = reqwest::get(url).await?;
        let mut buf = Vec::new();

        while let Some(chunk) = response.chunk().await? {
            buf.write(&chunk)?;
        }

        let html = String::from_utf8(buf)?;
        let document = Html::parse_document(&html);
        // TODO: do this without `unwrap`
        let selector = Selector::parse("p.heading").unwrap();

        let output_filename = format!("CRI-{}_headings.txt", &year);
        let output_file = File::create(output_filename)?;
        let mut buf_writer = BufWriter::new(output_file);

        // write output
        for p_tag in document.select(&selector) {
            let raw_heading = p_tag.inner_html();
            let heading = match raw_heading.find('<') {
                None => raw_heading,
                Some(i) => raw_heading[..i].to_owned(),
            };
            writeln!(&mut buf_writer, "{}", heading)?;
        }
    }
    Ok(())
}

fn build_url(year: &String, format: &String, api_key: &String) -> String {
    format!("{}{}/{}?api_key={}", BASE_URL, year, format, api_key)
}
