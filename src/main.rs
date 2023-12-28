use clap::Parser;
use cri_headings::{run, Args};

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
