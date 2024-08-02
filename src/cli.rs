use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "OSCAR2Parquet")]
#[command(author = "Pedro Ortiz Suarez <pedro@commoncrawl.org>")]
#[command(version = "0.1.0")]
#[command(about = "Converts OSCAR's jsonl files into parquet.", long_about = None)]
pub struct Cli {
    /// Folder containing the indices
    #[arg(value_name = "INPUT FOLDER")]
    pub src: PathBuf,

    /// Parquet file to write
    #[arg(value_name = "DESTINATION FOLDER")]
    pub dst: PathBuf,

    /// Number of threads to use
    #[arg(short, long, default_value = "10", value_name = "NUMBER OF THREADS")]
    pub threads: usize,
}
