mod cli;
mod commands;
mod config;
mod error;
mod output;
mod redmine_client;

use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    let exit_code = commands::run(cli).await;
    std::process::exit(exit_code);
}
