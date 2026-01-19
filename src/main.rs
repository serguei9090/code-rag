use anyhow::Context;
use clap::{Parser, Subcommand};

use code_rag::commands::{index, search, serve, watch};
use code_rag::config::AppConfig;
use code_rag::telemetry::{init_telemetry, AppMode};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

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

        /// Workspace name (default: "default")
        #[arg(short, long, default_value = "default")]
        workspace: String,

        /// Device to use (auto, cpu, cuda, metal)
        #[arg(long)]
        device: Option<String>,

        /// Processing batch size (default: 256)
        #[arg(long)]
        batch_size: Option<usize>,

        /// Max number of threads
        #[arg(long)]
        threads: Option<usize>,

        /// Process priority (low, normal, high)
        #[arg(long)]
        priority: Option<String>,
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

        /// Workspace name (default: "default")
        #[arg(short, long, default_value = "default")]
        workspace: String,

        /// Optimize context to fit within N tokens (e.g. 8000)
        #[arg(long)]
        max_tokens: Option<usize>,

        /// Device to use (auto, cpu, cuda, metal)
        #[arg(long)]
        device: Option<String>,

        /// Expand query using local LLM
        #[arg(long)]
        expand: bool,
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

        /// Workspace name (default: "default")
        #[arg(short, long, default_value = "default")]
        workspace: String,
    },
    /// Start the Model Context Protocol (MCP) server
    Mcp,
    /// Start unified services (Serve, MCP, Watch) based on config
    Start,
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
            workspace,
            device,
            batch_size,
            threads,
            priority,
        } => {
            let mut config = config.clone();
            if let Some(d) = device {
                config.device = d;
            }
            if let Some(p) = priority {
                config.priority = p;
            }
            if let Some(t) = threads {
                tracing::warn!(
                    "Thread limit {} requested but not yet implemented - using default thread pool",
                    t
                );
                config.threads = Some(t);
            }
            if let Some(bs) = batch_size {
                config.batch_size = bs;
            }

            // Apply process priority
            // NOTE: `apply_process_priority` is not defined in the provided context.
            // Assuming it's a function that needs to be implemented or imported.
            // For now, it will cause a compilation error if not present.
            // Apply process priority
            apply_process_priority(&config.priority);

            index::index_codebase(
                index::IndexOptions {
                    path,
                    db_path: None,
                    update,
                    force,
                    workspace,
                    batch_size: Some(config.batch_size),
                    threads: config.threads,
                },
                &config,
            )
            .await?;
        }
        Commands::Search {
            query,
            limit,
            json,
            html,
            ext,
            dir,
            no_rerank,
            workspace,
            max_tokens,
            device,
            expand,
        } => {
            let mut config = config.clone();
            if let Some(d) = device {
                config.device = d;
            }
            let options = search::SearchOptions {
                limit,
                db_path: None,
                html,
                json,
                ext,
                dir,
                no_rerank,
                workspace: Some(workspace),

                max_tokens,
                expand,
            };
            search::search_codebase(query, options, &config).await?;
        }
        Commands::Grep { pattern, json } => {
            search::grep_codebase(pattern, json, &config)?;
        }
        Commands::Serve { port, host } => {
            serve::serve_api(port, host, None, &config).await?;
        }
        Commands::Watch { path, workspace } => {
            watch::watch_codebase(path, None, workspace, &config).await?;
        }
        Commands::Mcp => {
            code_rag::commands::mcp::run(&config).await?;
        }
        Commands::Start => {
            code_rag::commands::start::run(&config).await?;
        }
    }

    Ok(())
}

fn apply_process_priority(priority: &str) {
    let p_lower = priority.to_lowercase();
    match p_lower.as_str() {
        "normal" => {
            // Default, do nothing usually
        }
        "low" => {
            tracing::info!("Setting process priority to LOW");
            set_priority_low();
        }
        "high" => {
            tracing::info!("Setting process priority to HIGH");
            set_priority_high();
        }
        _ => {
            tracing::warn!(
                "Unknown priority '{}'. Use 'low', 'normal', or 'high'.",
                priority
            );
        }
    }
}

#[cfg(windows)]
fn set_priority_low() {
    // It's hard to change OWN priority without bindings.
    // Hack: We can't easily change it without `winapi` or `windows-sys` crate.
    // However, we can warn user:
    // tracing::warn!("Priority setting on Windows requires 'winapi' dependency. Skipping.");

    // Better: Use a simple Powershell command to set own priority?
    // powershell -Command "$process = Get-Process -Id $PID; $process.PriorityClass = 'BelowNormal'"
    let pid = std::process::id();
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$process = Get-Process -Id {}; $process.PriorityClass = 'BelowNormal'",
                pid
            ),
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
}

#[cfg(windows)]
fn set_priority_high() {
    let pid = std::process::id();
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$process = Get-Process -Id {}; $process.PriorityClass = 'AboveNormal'",
                pid
            ),
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
}

#[cfg(unix)]
fn set_priority_low() {
    // raw syscall or 'nice' command?
    // calling `nice` externally on self is tricky.
    // unsafe { libc::nice(10) };
    // Since we don't want to add libc dep just for this if we can avoid it...
    // But we probably don't have libc dep.
    tracing::warn!("Priority setting on Unix not fully implemented without libc.");
}

#[cfg(unix)]
fn set_priority_high() {
    tracing::warn!("Priority setting on Unix not fully implemented without libc.");
}

#[cfg(not(any(windows, unix)))]
fn set_priority_low() {}
#[cfg(not(any(windows, unix)))]
fn set_priority_high() {}
