mod config;
mod hashtag_importer;
mod types;

use std::error::Error;

use clap::{Parser, Subcommand};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match &cli.command {
        // TODO: merge both first steps + toml-edit for init ?
        Commands::CreateApp => hashtag_importer::create_app()?,
        Commands::UserAuth => hashtag_importer::user_auth()?,
        Commands::Run => hashtag_importer::run()?,
    }
    Ok(())
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create app when run for the first time; registers this app on your mastodon instance
    CreateApp,
    /// Get permission to run as your user on server described on config.toml in order to get a token; needs the app
    /// to have been created already. Only read permission (scope) is required, in order search
    /// posts and read hashtag timelines.
    UserAuth,
    /// Run actual service based on config.toml
    Run,
}
