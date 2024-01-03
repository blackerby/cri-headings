# cri-headings

A command line tool for downloading Congressional Record Index headings from the GovInfo API.

## Installation

### Options

- Download and unzip the appropriate binary release for your system
- Clone and compile the source

## Usage

```bash
$ cri-headings --help
Get Congressional Record Index headings from the GovInfo API

Usage: cri-headings [OPTIONS] [YEARS]...

Arguments:
  [YEARS]...  CRI years to download. Default to current year [default: 2024]

Options:
      --page-size <PAGE_SIZE>    API page size [default: 1000]
      --output-dir <OUTPUT_DIR>  Output directory [default: .]
      --api-key <API_KEY>        GovInfo API Key [default: DEMO_KEY]
  -c, --csv                      Write CSV
  -h, --help                     Print help
  -V, --version                  Print version
```
