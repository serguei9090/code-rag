use anyhow::Context;
use clap::{Parser, Subcommand};

use code_rag::commands::{index, search, serve, watch};
use code_rag::config::AppConfig;
use code_rag::telemetry::{init_telemetry, AppMode};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Index a codebase
    Index {
        /// Path to the codebase to index
        #[arg(short, long)]
        path: Option<String>,

        /// Update existing index instead of rebuilding
        #[arg(short, long)]
        update: bool,

        /// Force re-indexing (removes existing DB)
        #[arg(short, long)]
        force: bool,
    },
    /// Search the indexed codebase
    Search {
        /// The search query
        query: String,

        /// Limit the number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Generate HTML report
        #[arg(long)]
        html: bool,

        /// Filter by file extension
        #[arg(long)]
        ext: Option<String>,

        /// Filter by directory
        #[arg(long)]
        dir: Option<String>,

        /// Disable reranking (faster)
        #[arg(long)]
        no_rerank: bool,
    },
    /// Grep search (regex)
    Grep {
        /// The regex pattern
        pattern: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Start the search API server
    Serve {
        /// Port to listen on (default: 8000)
        #[arg(long)]
        port: Option<u16>,

        /// Host to bind to (default: 127.0.0.1)
        #[arg(long)]
        host: Option<String>,
    },
    /// Watch the codebase for changes and auto-index
    Watch {
        /// Path to watch
        #[arg(short, long)]
        path: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load Configuration
    // We strictly follow precedence config_rag.toml -> defaults.
    // NOTE: In a real app we might peek at args to find -c first, logic simplified here.
    let config = AppConfig::new().context("Failed to load configuration")?;

    // 2. Parse Args Early to determine Telemetry Mode
    let args = Args::parse();

    // 3. Setup Telemetry
    // If command is Serve, we use Server mode (OTLP), otherwise CLI mode (Chrome/Local)
    let app_mode = match args.command {
        Commands::Serve { .. } => AppMode::Server,
        _ => AppMode::Cli,
    };

    // Initialize telemetry. This guard must be held until the end of main.
    // Note: init_telemetry internally handles logging initialization for now,
    // replacing the old init_logging function.
    let _guard = init_telemetry(app_mode, &config).context("Failed to initialize telemetry")?;

    // 4. Execute Command
    match args.command {
        Commands::Index {
            path,
            update,
            force,
        } => {
            index::index_codebase(path, None, update, force, &config).await?;
        }
        Commands::Search {
            query,
            limit,
            json,
            html,
            ext,
            dir,
            no_rerank,
        } => {
            let options = search::SearchOptions {
                limit,
                db_path: None,
                html,
                json,
                ext,
                dir,
                no_rerank,
            };
            search::search_codebase(query, options, &config).await?;
        }
        Commands::Grep { pattern, json } => {
            search::grep_codebase(pattern, json, &config)?;
        }
        Commands::Serve { port, host } => {
            serve::serve_api(port, host, None, &config).await?;
        }
        Commands::Watch { path } => {
            watch::watch_codebase(path, None, &config).await?;
        }
    }

    Ok(())
}
