use serde_json::json;
use std::io::{BufRead, Write};
use std::process::Stdio;

#[test]
fn test_mcp_initialize() {
    let bin_path = env!("CARGO_BIN_EXE_code-rag");
    let mut cmd = std::process::Command::new(bin_path);
    let mut child = cmd
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Capture stderr to avoid polluting test output
        .spawn()
        .expect("Failed to spawn MCP process");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");

    // 1. Initialize
    let init_req = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 1
    });

    let req_str = serde_json::to_string(&init_req).unwrap();
    writeln!(stdin, "{}", req_str).unwrap();

    // Read response
    // We need to read line by line carefully
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut line = String::new();

    // Might read logs if they go to stdout, but we expect MCP to be clean on stdout?
    // The logs (tracing) default mainly to stderr or custom subscriber.
    // If we see non-JSON lines, we might skip them or fail.
    // But MCP implementation prints strictly JSON to stdout.

    // First response: InitializeResult
    reader.read_line(&mut line).expect("Failed to read line");
    let response: serde_json::Value =
        serde_json::from_str(&line).expect("Failed to parse JSON response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["serverInfo"]["name"] == "code-rag");

    // 2. Initialized Notification
    let notified_req = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    writeln!(stdin, "{}", serde_json::to_string(&notified_req).unwrap()).unwrap();

    // 3. List Tools
    let list_req = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 2
    });
    writeln!(stdin, "{}", serde_json::to_string(&list_req).unwrap()).unwrap();

    line.clear();
    reader.read_line(&mut line).expect("Failed to read line");
    let response: serde_json::Value =
        serde_json::from_str(&line).expect("Failed to parse JSON response");
    assert_eq!(response["id"], 2);
    let tools = response["result"]["tools"]
        .as_array()
        .expect("Tools list missing");
    assert!(tools.iter().any(|t| t["name"] == "search"));

    // Terminate
    child.kill().ok();
    child.wait().ok();
}
