use aiql_core::crawlers::PostgresSchemaCrawler;
use aiql_core::SchemaCrawler;
use clap::{Parser, Subcommand};
use colored::*;
use sqlx::postgres::PgPoolOptions;

#[derive(Parser)]
#[command(name = "aiql")]
#[command(about = "AI Query Layer CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crawl a database schema
    Crawl {
        /// Database URL
        #[arg(short, long)]
        url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Crawl { url } => {
            println!("{}", "Crawling database schema...".cyan());
            
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(url)
                .await?;

            let crawler = PostgresSchemaCrawler::new(pool);
            let schema = crawler.crawl().await?;

            println!("{}", "Schema crawled successfully!".green().bold());
            println!("{:#?}", schema);
        }
    }

    Ok(())
}
