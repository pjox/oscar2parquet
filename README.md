# OSCAR2Parquet

This cli tool converts [OSCAR](https://oscar-project.org)'s jsonl files into parquet. It takes [Ungoliant](https://github.com/oscar-project/ungoliant)'s output as input and writes the parquet files to the destination folder. This tool intends to replace the splitting and compression steps of the [OSCAR generation](https://oscar-project.github.io/documentation/tools/generation-jeanzay/#preparing-for-release) previously performed by [oscar-tools](https://github.com/oscar-project/oscar-tools).

## Todo

- [ ] Add Python bindings
- [ ] Add tests
- [ ] Add option to control the maximum number of rows per parquet file

## Usage

```text
oscar2parquet -h
Converts OSCAR's jsonl files into parquet.

Usage: oscar2parquet [OPTIONS] <INPUT FOLDER> <DESTINATION FOLDER>

Arguments:
  <INPUT FOLDER>        Folder containing the indices
  <DESTINATION FOLDER>  Parquet file to write

Options:
  -t, --threads <NUMBER OF THREADS>  Number of threads to use [default: 10]
  -h, --help                         Print help
  -V, --version                      Print version
```
