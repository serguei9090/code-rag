Code-RAG CLI

code-rag is a cross-platform CLI tool for indexing and querying source code using embeddings and LanceDB.

It can be used:

As a native CLI binary on Windows and Linux

Or via Docker for fully reproducible and isolated execution

By agents / scripts / CI as a normal command-line tool

📦 What gets built?

Depending on the platform:

Linux:

target/release/code-rag


Windows:

target\release\code-rag.exe


These are native executables.

The target/ folder should NOT be committed to git.

🛠️ Build (Native, without Docker)
Requirements

Rust toolchain (https://rustup.rs
)

On Linux:

sudo apt install build-essential pkg-config libssl-dev protobuf-compiler


On Windows:

Install Rust via rustup

Install Visual Studio Build Tools (C++ workload)

Build (Linux or Windows)
cargo build --release


Output:

Linux:

./target/release/code-rag


Windows:

.\target\release\code-rag.exe

▶️ Run (Native)
Linux
./target/release/code-rag --help

Windows (PowerShell)
.\target\release\code-rag.exe --help

🐳 Build & Run with Docker (Recommended for reproducibility)

This project includes a multi-stage Docker build and docker-compose setup.

Requirements

Docker Desktop

Docker Compose

Build the image
docker compose build code-rag


The build uses BuildKit cache mounts so repeated builds are much faster.

Run the CLI
docker compose run --rm code-rag --help

Index a project (example)
docker compose run --rm code-rag index /workspace


Your current repo is mounted read-only into /workspace.

Where is the database stored?

LanceDB is persisted in a Docker volume:

/data/.lancedb


So your index survives container restarts.

⚡ Fast development rebuilds (optional)

There is a builder service for fast iteration without rebuilding the runtime image:

docker compose run --rm builder


This reuses:

Cargo registry cache

Cargo git cache

target/ build cache

🤖 How should agents use this?

Your agents can run code-rag in three ways:

1) Native binary (fastest)

Windows:

code-rag.exe ...


Linux:

./code-rag ...

2) Docker (most portable / safest)
docker compose run --rm code-rag ...

3) CI / Automation

In CI or scripts:

docker build -t code-rag .
docker run --rm -v $(pwd):/workspace code-rag --help

📁 Recommended project layout for releases

Do not commit target/.

Instead, when publishing:

dist/
  windows/code-rag.exe
  linux/code-rag


These can be attached to GitHub Releases.

🧠 Important notes

The Linux binary built in Docker is Linux-only

Windows must be built on Windows (or via cross-compilation setup)

Docker avoids all OS dependency issues and is recommended for automation

🔍 Debugging build issues

If Docker build fails:

docker build --no-cache --progress=plain .

📜 License

(Add your license here)