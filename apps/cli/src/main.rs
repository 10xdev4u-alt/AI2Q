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
    /// Translate natural language to SQL
    Translate {
        /// Natural language prompt
        #[arg(short, long)]
        prompt: String,
        /// Database URL for schema context
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
        Commands::Translate { prompt, url } => {
            println!("{}", "Crawling schema for context...".cyan());
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(url)
                .await?;
            let crawler = PostgresSchemaCrawler::new(pool);
            let schema = crawler.crawl().await?;

            println!("{}", "Translating prompt...".cyan());
            let translator = aiql_core::translator::MockTranslator;
            let plan = aiql_core::Translator::translate(&translator, prompt, &schema).await?;

            println!("{}", "Translation generated:".green().bold());
            println!("{}: {}", "SQL".bold().blue(), plan.raw_query.yellow());
            println!("{}: {}", "Explanation".bold().blue(), plan.explanation.dimmed());
        }
    }

    Ok(())
}
