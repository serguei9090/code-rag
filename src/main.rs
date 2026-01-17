use anyhow::Context;
use clap::{Parser, Subcommand};
use time::macros::format_description;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use code_rag::commands::{index, search, serve, watch};
use code_rag::config::AppConfig;

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

    // 2. Setup Logging
    let _guard = init_logging(&config);

    // 3. Parse Args
    let args = Args::parse();

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
            search::search_codebase(query, limit, None, html, json, ext, dir, no_rerank, &config)
                .await?;
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

fn init_logging(config: &AppConfig) -> Option<WorkerGuard> {
    let log_level = config.log_level.parse::<Level>().unwrap_or(Level::INFO);
    let log_format = &config.log_format;

    // Setup Local Timer
    let timer = tracing_subscriber::fmt::time::LocalTime::new(format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6][offset_hour sign:mandatory]:[offset_minute]"
    ));

    // Stderr Layer (User Feedback)
    let stderr_layer = if log_format.eq_ignore_ascii_case("json") {
        tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_timer(timer.clone())
            .json()
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_timer(timer.clone())
            .boxed()
    };

    // Filter stderr: Warnings/Errors always shown, Info shown if level >= Info
    let stderr_filter =
        tracing_subscriber::filter::Targets::new().with_target("code_rag", log_level);

    let registry = tracing_subscriber::registry().with(stderr_layer.with_filter(stderr_filter));

    // File Layer (Optional)
    if config.log_to_file {
        let file_appender = tracing_appender::rolling::daily(&config.log_dir, "code-rag.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_timer(timer)
            .boxed();

        let file_filter =
            tracing_subscriber::filter::Targets::new().with_target("code_rag", log_level);

        registry.with(file_layer.with_filter(file_filter)).init();

        Some(guard)
    } else {
        registry.init();
        None
    }
}
