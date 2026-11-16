//! Contract test: spawns the built `rdm-mcp` binary and talks MCP 2026-07-28
//! to it over stdio, as a host would.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Stdio};

use rmcp::{
    ClientHandler, ClientLifecycleMode, ClientServiceExt, RoleClient,
    model::{CallToolRequestParams, ProtocolVersion, ReadResourceRequestParams, Tool},
    service::RunningService,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::{Value, json};
use tokio::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_rdm-mcp");
const APPS_MIME: &str = "text/html;profile=mcp-app";

#[derive(Clone, Default)]
struct TestClient;
impl ClientHandler for TestClient {}

async fn connect(probe_dir: Option<&Path>) -> RunningService<RoleClient, TestClient> {
    let transport = TokioChildProcess::new(Command::new(BIN).configure(|cmd| {
        if let Some(dir) = probe_dir {
            cmd.env("RDM_PROBE_DIR", dir);
        }
    }))
    .expect("spawn rdm-mcp");
    TestClient
        .serve_with_lifecycle(
            transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await
        .expect("connect at 2026-07-28")
}

fn tool<'a>(tools: &'a [Tool], name: &str) -> &'a Tool {
    tools
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("tool {name} is listed"))
}

fn meta_of(t: &Tool) -> Value {
    serde_json::to_value(t.meta.as_ref().expect("tool has _meta")).expect("meta serialises")
}

#[tokio::test]
async fn lists_three_tools_with_ui_meta() {
    let client = connect(None).await;
    let tools = client.list_all_tools().await.expect("tools/list");
    let mut names: Vec<_> = tools.iter().map(|t| t.name.to_string()).collect();
    names.sort();
    assert_eq!(names, ["leaf_get", "probe_record", "roadmap_view"]);

    assert_eq!(
        meta_of(tool(&tools, "roadmap_view"))["ui"]["resourceUri"],
        "ui://rdm/graph"
    );
    for app_only in ["leaf_get", "probe_record"] {
        assert_eq!(
            meta_of(tool(&tools, app_only))["ui"]["visibility"],
            json!(["app"]),
            "{app_only} is app-only"
        );
    }
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn roadmap_view_structured_content_matches_schema() {
    let client = connect(None).await;
    let tools = client.list_all_tools().await.expect("tools/list");
    let schema = Value::Object(
        tool(&tools, "roadmap_view")
            .output_schema
            .as_deref()
            .expect("roadmap_view has an outputSchema")
            .clone(),
    );
    let result = client
        .call_tool(CallToolRequestParams::new("roadmap_view"))
        .await
        .expect("call roadmap_view");
    assert_ne!(result.is_error, Some(true));
    let structured = result.structured_content.expect("structuredContent");

    let validator = jsonschema::validator_for(&schema).expect("outputSchema compiles");
    let errors: Vec<String> = validator
        .iter_errors(&structured)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "structuredContent violates outputSchema: {errors:?}");

    assert_eq!(structured["campaign"], "roadmap-mcp-rework");
    assert_eq!(structured["nodes"].as_array().expect("nodes").len(), 16);
    assert!(structured["t_server_ms"].as_u64().expect("t_server_ms") > 0);
    assert!(structured["payload_bytes"].as_u64().expect("payload_bytes") > 0);
    assert_eq!(result.content.len(), 1, "one text summary");
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn reads_ui_resource_with_apps_mime() {
    let client = connect(None).await;
    let result = client
        .read_resource(ReadResourceRequestParams::new("ui://rdm/graph"))
        .await
        .expect("resources/read");
    let contents = serde_json::to_value(&result.contents[0]).expect("serialises");
    assert_eq!(contents["mimeType"], APPS_MIME);
    assert!(contents["text"].as_str().expect("text").contains("<html"));
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn declares_apps_extension() {
    let client = connect(None).await;
    let info = client.peer_info().expect("server info after discover");
    let caps = serde_json::to_value(&info.capabilities).expect("serialises");
    assert_eq!(
        caps["extensions"]["io.modelcontextprotocol/ui"],
        json!({ "mimeTypes": [APPS_MIME] })
    );
    assert_eq!(info.protocol_version, ProtocolVersion::V_2026_07_28);
    client.cancel().await.expect("close");
}

/// Sends raw JSON-RPC lines to a fresh server and returns its first reply.
fn raw_exchange(lines: &[Value]) -> Value {
    let mut child = StdCommand::new(BIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn rdm-mcp");
    let mut stdin = child.stdin.take().expect("stdin");
    for l in lines {
        writeln!(stdin, "{l}").expect("write request");
    }
    let mut reply = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut reply)
        .expect("read reply");
    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
    serde_json::from_str(&reply).expect("reply is JSON")
}

#[test]
fn refuses_older_protocol() {
    // The initialize handshake at 2025-11-25 is refused.
    let reply = raw_exchange(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "t", "version": "0" }
        }
    })]);
    assert!(reply.get("result").is_none(), "initialize must not succeed: {reply}");
    assert!(reply["error"]["message"].is_string(), "protocol error expected: {reply}");

    // A request that names 2025-11-25 in its own _meta is refused too.
    let reply = raw_exchange(&[json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/list",
        "params": { "_meta": {
            "io.modelcontextprotocol/protocolVersion": "2025-11-25",
            "io.modelcontextprotocol/clientInfo": { "name": "t", "version": "0" },
            "io.modelcontextprotocol/clientCapabilities": {}
        } }
    })]);
    assert!(reply.get("result").is_none(), "tools/list must not succeed: {reply}");
    assert!(reply["error"]["message"].is_string(), "protocol error expected: {reply}");
    assert!(
        reply["error"].to_string().contains("2026-07-28"),
        "error names the supported version: {reply}"
    );
}

fn scratch_dir() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[tokio::test]
async fn probe_record_appends_one_line() {
    let dir = scratch_dir();
    let client = connect(Some(&dir)).await;
    let args = json!({
        "run_kind": "cold",
        "t_server_ms": 1,
        "t_view_ms": 2,
        "apps_protocol_announced": "2026-01-26",
        "apps_protocol_host": null,
        "host_capabilities": { "a": 1 },
        "host_context": {},
        "display_mode": "inline"
    });
    let result = client
        .call_tool(
            CallToolRequestParams::new("probe_record")
                .with_arguments(args.as_object().expect("object").clone()),
        )
        .await
        .expect("call probe_record");
    assert_ne!(result.is_error, Some(true), "{result:?}");
    client.cancel().await.expect("close");

    let text = std::fs::read_to_string(dir.join("probe.jsonl")).expect("probe.jsonl exists");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "exactly one line: {text:?}");
    let line: Value = serde_json::from_str(lines[0]).expect("valid JSON line");
    assert_eq!(line["mcp_protocol"], "2026-07-28");
    assert_eq!(line["run_kind"], "cold");
    assert!(line["t_recorded_ms"].as_u64().expect("t_recorded_ms") > 0);
    let _ = std::fs::remove_dir_all(&dir);
}
