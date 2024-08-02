use clap::Parser;

mod cli;
mod convert;
mod oscar;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    convert::convert_to_parquet(&cli.src, &cli.dst, cli.threads).await;
}
