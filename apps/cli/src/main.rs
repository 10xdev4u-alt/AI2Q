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
    /// Generate and execute a natural language migration
    Migrate {
        /// Migration prompt
        #[arg(short, long)]
        prompt: String,
        /// Database URL
        #[arg(short, long)]
        url: String,
        /// OpenAI API Key
        #[arg(long, env = "OPENAI_API_KEY")]
        openai_key: Option<String>,
        /// OpenAI Model
        #[arg(long, default_value = "gpt-4-turbo-preview")]
        model: String,
        /// Use Ollama for local inference
        #[arg(long)]
        use_ollama: bool,
    },
    /// Generate synthetic data for testing
    Mock {
        /// Mock data prompt
        #[arg(short, long)]
        prompt: String,
        /// Database URL
        #[arg(short, long)]
        url: String,
        /// OpenAI API Key
        #[arg(long, env = "OPENAI_API_KEY")]
        openai_key: Option<String>,
        /// OpenAI Model
        #[arg(long, default_value = "gpt-4-turbo-preview")]
        model: String,
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
        Commands::Migrate { prompt, url, openai_key, model, use_ollama } => {
            println!("{}", "Crawling schema for context...".cyan());
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(url)
                .await?;
            let crawler = PostgresSchemaCrawler::new(pool.clone());
            let schema = crawler.crawl().await?;

            println!("{}", "Generating migration...".cyan());
            let config = if *use_ollama {
                 aiql_core::translator::TranslatorConfig::Ollama { host: "http://localhost".into(), port: 11434, model: model.clone() }
            } else if let Some(key) = openai_key {
                aiql_core::translator::TranslatorConfig::OpenAI { api_key: key.clone(), model: model.clone(), temperature: Some(0.0) }
            } else {
                aiql_core::translator::TranslatorConfig::Mock
            };

            let translator = aiql_core::translator::create_translator(config);
            let plan = translator.translate_migration(prompt, &schema, aiql_core::DatabaseDialect::Postgres).await?;

            println!("{}", "Generated Migration:".green().bold());
            println!("{}: {}", "SQL".bold().blue(), plan.raw_sql.yellow());
            println!("{}: {}", "Explanation".bold().blue(), plan.explanation.dimmed());

            use dialoguer::Confirm;
            if Confirm::new().with_prompt("Do you want to execute this migration?").interact()? {
                println!("{}", "Executing migration...".cyan());
                let engine = aiql_core::migration::PostgresMigrationEngine::new(pool);
                aiql_core::MigrationEngine::migrate(&engine, &plan).await?;
                println!("{}", "Migration executed successfully!".green().bold());
            } else {
                println!("{}", "Migration cancelled.".yellow());
            }
        }
        Commands::Mock { prompt, url, openai_key, model } => {
            println!("{}", "Crawling schema for context...".cyan());
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(url)
                .await?;
            let crawler = PostgresSchemaCrawler::new(pool.clone());
            let schema = crawler.crawl().await?;

            println!("{}", "Generating mock data...".cyan());
            let translator = aiql_core::translator::OpenAITranslator::new(
                openai_key.clone().unwrap_or_default(),
                model.clone()
            );
            
            let queries = aiql_core::MockDataGenerator::generate_mock_data(&translator, prompt, &schema, aiql_core::DatabaseDialect::Postgres).await?;

            println!("{}", "Generated Mock Queries:".green().bold());
            for q in &queries {
                println!("{}", q.yellow());
            }

            use dialoguer::Confirm;
            if Confirm::new().with_prompt("Do you want to execute these mock queries?").interact()? {
                println!("{}", "Executing mock data insertion...".cyan());
                for q in queries {
                    sqlx::query(&q).execute(&pool).await?;
                }
                println!("{}", "Mock data inserted successfully!".green().bold());
            }
        }
    }

    Ok(())
}
