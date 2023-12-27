use anyhow::{bail, Result};
use scraper::{Html, Selector};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

fn main() -> Result<()> {
    // parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <file name>"),
        _ => {}
    }

    // read input
    let file_name = &args[1];
    let file = File::open(file_name)?;
    let mut buf_reader = BufReader::new(file);
    let mut html = String::new();
    buf_reader.read_to_string(&mut html)?;
    let document = Html::parse_document(&html);
    let selector = Selector::parse("p.heading").unwrap();

    // setup output file
    let file_stem = Path::new(file_name).file_stem().unwrap();
    let output_filename = format!("{}_headings.txt", file_stem.to_str().unwrap());
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
    Ok(())
}
