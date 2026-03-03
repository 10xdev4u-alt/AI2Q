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
        /// OpenAI API Key
        #[arg(long, env = "OPENAI_API_KEY")]
        openai_key: Option<String>,
        /// OpenAI Model
        #[arg(long, default_value = "gpt-4-turbo-preview")]
        model: String,
        /// Ollama Host
        #[arg(long, default_value = "http://localhost")]
        ollama_host: String,
        /// Ollama Port
        #[arg(long, default_value = "11434")]
        ollama_port: u16,
        /// Use Ollama for local inference
        #[arg(long)]
        use_ollama: bool,
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
        Commands::Translate { prompt, url, openai_key, model, ollama_host, ollama_port, use_ollama } => {
            println!("{}", "Crawling schema for context...".cyan());
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(url)
                .await?;
            let crawler = PostgresSchemaCrawler::new(pool);
            let schema = crawler.crawl().await?;

            println!("{}", "Translating prompt...".cyan());
            let config = if *use_ollama {
                aiql_core::translator::TranslatorConfig::Ollama { host: ollama_host.clone(), port: *ollama_port, model: model.clone() }
            } else if let Some(key) = openai_key {
                aiql_core::translator::TranslatorConfig::OpenAI { api_key: key.clone(), model: model.clone(), temperature: None }
            } else {
                aiql_core::translator::TranslatorConfig::Mock
            };

            let translator = aiql_core::translator::create_translator(config);
            let plan = translator.translate(prompt, &schema).await?;

            println!("{}", "Translation generated:".green().bold());
            println!("{}: {}", "SQL".bold().blue(), plan.raw_query.yellow());
            println!("{}: {}", "Explanation".bold().blue(), plan.explanation.dimmed());
        }
    }

    Ok(())
}
