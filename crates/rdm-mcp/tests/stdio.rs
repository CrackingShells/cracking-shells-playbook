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

// ---- data channels ----

async fn view(client: &RunningService<RoleClient, TestClient>, mode: &str, scale: u32) -> Value {
    let args = json!({ "mode": mode, "scale": scale });
    let result = client
        .call_tool(
            CallToolRequestParams::new("roadmap_view")
                .with_arguments(args.as_object().expect("object").clone()),
        )
        .await
        .expect("call roadmap_view");
    result.structured_content.expect("structuredContent")
}

fn keys(v: &Value) -> Vec<&str> {
    let mut k: Vec<&str> = v.as_object().expect("object").keys().map(String::as_str).collect();
    k.sort_unstable();
    k
}

const COMMON: [&str; 7] = [
    "campaign", "frozen_at", "mode", "node_count", "payload_bytes", "scale", "t_server_ms",
];

#[tokio::test]
async fn each_mode_returns_documented_keys() {
    let client = connect(None).await;
    let result = view(&client, "result", 1).await;
    for k in ["nodes", "mode", "scale", "t_server_ms", "payload_bytes", "node_count"] {
        assert!(result.get(k).is_some(), "result lacks {k}");
    }
    assert!(result.get("stream_url").is_none() && result.get("chunk_count").is_none());
    let dir = result["nodes"].as_array().expect("nodes").iter().find(|n| n["kind"] == "dir").expect("a dir");
    assert!(dir.get("markdown").is_none(), "dir has no markdown");
    let leaf = result["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|n| n["kind"] == "leaf")
        .expect("a leaf");
    assert!(leaf["markdown"].as_str().expect("markdown").starts_with('#'));

    let chunks = view(&client, "chunks", 2).await;
    assert!(chunks.get("nodes").is_none());
    assert!(chunks["chunk_count"].as_u64().expect("chunk_count") >= 1);
    assert_eq!(chunks["chunk_uri_template"], "rdm://chunk/{n}?scale=2");

    let stream = view(&client, "stream", 3).await;
    assert!(stream.get("nodes").is_none());
    let url = stream["stream_url"].as_str().expect("stream_url");
    assert!(url.starts_with("http://127.0.0.1:") && url.ends_with("/stream?scale=3"), "{url}");
    for v in [&chunks, &stream] {
        for k in COMMON {
            assert!(v.get(k).is_some(), "missing {k}: {:?}", keys(v));
        }
    }
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn result_mode_refuses_scale() {
    let client = connect(None).await;
    let args = json!({ "mode": "result", "scale": 10 });
    let err = client
        .call_tool(
            CallToolRequestParams::new("roadmap_view")
                .with_arguments(args.as_object().expect("object").clone()),
        )
        .await
        .expect_err("scale 10 in result mode is refused");
    let text = err.to_string();
    assert!(text.contains("model context"), "{text}");
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn chunks_reassemble_to_scaled_nodes() {
    let client = connect(None).await;
    let v = view(&client, "chunks", 20).await;
    let count = v["chunk_count"].as_u64().expect("chunk_count");
    assert!(count > 1, "scale 20 spans several chunks");
    let mut nodes = Vec::new();
    for n in 0..count {
        let r = client
            .read_resource(ReadResourceRequestParams::new(format!("rdm://chunk/{n}?scale=20")))
            .await
            .expect("read chunk");
        let c = serde_json::to_value(&r.contents[0]).expect("serialises");
        assert_eq!(c["mimeType"], "application/json");
        let body: Value = serde_json::from_str(c["text"].as_str().expect("text")).expect("json");
        assert_eq!(body["n"], n);
        nodes.extend(body["nodes"].as_array().expect("nodes").clone());
    }
    assert_eq!(nodes.len() as u64, v["node_count"].as_u64().expect("node_count"));
    assert_eq!(nodes.len(), 16 * 20);
    assert_eq!(nodes[0]["path"], "m0_feasibility_spike.md");
    assert_eq!(nodes[16]["path"], "copy1/m0_feasibility_spike.md");
    assert_eq!(nodes[16]["title"], nodes[0]["title"]);
    assert_eq!(
        serde_json::to_vec(&nodes).expect("ser").len() as u64,
        v["payload_bytes"].as_u64().expect("payload_bytes")
    );
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn sse_streams_node_count_events_then_done() {
    use std::io::Read;
    let client = connect(None).await;
    let v = view(&client, "stream", 5).await;
    let url = v["stream_url"].as_str().expect("stream_url");
    let addr = url
        .strip_prefix("http://")
        .and_then(|r| r.split_once('/'))
        .expect("url")
        .0;
    let mut tcp = std::net::TcpStream::connect(addr).expect("connect");
    write!(tcp, "GET /stream?scale=5 HTTP/1.1\r\nHost: {addr}\r\n\r\n").expect("send");
    let mut raw = String::new();
    tcp.read_to_string(&mut raw).expect("read to close");
    let (head, body) = raw.split_once("\r\n\r\n").expect("headers");
    let head = head.to_ascii_lowercase();
    assert!(head.contains("text/event-stream"), "{head}");
    assert!(head.contains("access-control-allow-origin: *"), "{head}");
    assert!(head.contains("cache-control: no-store"), "{head}");
    let events: Vec<&str> = body.split("\n\n").filter(|e| !e.is_empty()).collect();
    let n = v["node_count"].as_u64().expect("node_count") as usize;
    assert_eq!(n, 80);
    assert_eq!(events.len(), n + 1);
    assert!(events[..n].iter().all(|e| e.starts_with("data: {")));
    assert!(events[n].starts_with("event: done"));

    let mut other = std::net::TcpStream::connect(addr).expect("connect");
    write!(other, "GET /nope HTTP/1.1\r\n\r\n").expect("send");
    let mut raw = String::new();
    other.read_to_string(&mut raw).expect("read");
    assert!(raw.starts_with("HTTP/1.1 404"), "{raw}");
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn csp_domain_carries_the_bound_port() {
    let client = connect(None).await;
    let v = view(&client, "stream", 1).await;
    let port = v["stream_url"]
        .as_str()
        .expect("stream_url")
        .split(':')
        .nth(2)
        .and_then(|r| r.split('/').next())
        .expect("port")
        .to_owned();
    let want = json!([format!("http://127.0.0.1:{port}")]);

    let listed = client.list_all_resources().await.expect("resources/list");
    let page = serde_json::to_value(&listed[0]).expect("ser");
    assert_eq!(page["_meta"]["ui"]["csp"]["connectDomains"], want);

    let read = client
        .read_resource(ReadResourceRequestParams::new("ui://rdm/graph"))
        .await
        .expect("read");
    let c = serde_json::to_value(&read.contents[0]).expect("ser");
    assert_eq!(c["_meta"]["ui"]["csp"]["connectDomains"], want);
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn leaf_template_serves_markdown_and_refuses_unknown() {
    let client = connect(None).await;
    let templates = client.list_all_resource_templates().await.expect("templates");
    let t = serde_json::to_value(&templates).expect("ser");
    assert!(t.to_string().contains("rdm://leaf/{path}"), "{t}");
    let r = client
        .read_resource(ReadResourceRequestParams::new("rdm://leaf/core/m1_core_foundation.md"))
        .await
        .expect("read leaf");
    let c = serde_json::to_value(&r.contents[0]).expect("ser");
    assert_eq!(c["mimeType"], "text/markdown");
    assert!(c["text"].as_str().expect("text").starts_with('#'));
    client
        .read_resource(ReadResourceRequestParams::new("rdm://leaf/nope.md"))
        .await
        .expect_err("unknown leaf refused");
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn payload_sizes_by_scale() {
    let client = connect(None).await;
    let mut sizes = Vec::new();
    for s in [1, 10, 100] {
        let v = view(&client, "chunks", s).await;
        sizes.push((s, v["payload_bytes"].as_u64().expect("b"), v["chunk_count"].as_u64().expect("c")));
    }
    eprintln!("SIZES (scale, payload_bytes, chunk_count): {sizes:?}");
    assert!(sizes[0].1 < sizes[1].1 && sizes[1].1 < sizes[2].1);
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn meta_mode_carries_nodes_in_result_meta() {
    let client = connect(None).await;
    let args = json!({ "mode": "meta", "scale": 3 });
    let result = client
        .call_tool(
            CallToolRequestParams::new("roadmap_view")
                .with_arguments(args.as_object().expect("object").clone()),
        )
        .await
        .expect("call roadmap_view");
    let sc = result.structured_content.expect("structuredContent");
    assert_eq!(
        keys(&sc),
        ["campaign", "frozen_at", "mode", "node_count", "payload_bytes", "scale", "t_server_ms"]
    );
    assert_eq!(sc["mode"], "meta");
    let meta = serde_json::to_value(result.meta.as_ref().expect("result _meta")).expect("ser");
    let nodes = meta["rdm"]["nodes"].as_array().expect("_meta.rdm.nodes");
    assert_eq!(nodes.len() as u64, sc["node_count"].as_u64().expect("node_count"));
    assert_eq!(nodes.len(), 48);
    assert!(nodes.iter().any(|n| n["markdown"].is_string()));
    assert_eq!(result.content.len(), 1);

    let args = json!({ "mode": "meta", "scale": 11 });
    client
        .call_tool(
            CallToolRequestParams::new("roadmap_view")
                .with_arguments(args.as_object().expect("object").clone()),
        )
        .await
        .expect_err("scale 11 refused in meta mode");
    client.cancel().await.expect("close");
}
