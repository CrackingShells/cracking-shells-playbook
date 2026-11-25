//! Throwaway MCP Apps feasibility stub for rdm: serves a frozen roadmap
//! fixture, a `ui://` page and two app-only tools over stdio, on MCP
//! 2026-07-28 only.

use std::borrow::Cow;
use std::io::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    handler::server::{
        router::tool::ToolRouter,
        tool::schema_for_output,
        wrapper::{Json, Parameters},
    },
    model::{
        CallToolResult, ContentBlock, ExtensionCapabilities, Implementation,
        ListResourceTemplatesResult, ListResourcesResult, MetaObject, PaginatedRequestParams,
        ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult,
        Resource, ResourceContents, ResourceTemplate, ServerCapabilities, ServerConfig,
    },
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// URI of the graph page, referenced by `roadmap_view`'s `_meta.ui.resourceUri`.
const UI_URI: &str = "ui://rdm/graph";
/// MIME type that marks an MCP App page.
const APPS_MIME: &str = "text/html;profile=mcp-app";
/// Extension identifier of MCP Apps in `capabilities.extensions`.
const APPS_EXTENSION: &str = "io.modelcontextprotocol/ui";

const VIEW_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/view.html"));
const ROADMAP_JSON: &str = include_str!("../fixtures/roadmap.json");
include!(concat!(env!("OUT_DIR"), "/leaves.rs"));

/// One node of the frozen roadmap.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct FixtureNode {
    path: String,
    parent: Option<String>,
    kind: String,
    title: String,
    status: String,
    depth: u32,
}

/// The frozen roadmap as stored in `fixtures/roadmap.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Fixture {
    campaign: String,
    frozen_at: String,
    nodes: Vec<FixtureNode>,
}

/// A node as served to the page: the fixture node plus a leaf's markdown.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
struct ViewNode {
    #[serde(flatten)]
    node: FixtureNode,
    #[serde(skip_serializing_if = "Option::is_none")]
    markdown: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Result,
    Chunks,
    Stream,
    Meta,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Result => "result",
            Self::Chunks => "chunks",
            Self::Stream => "stream",
            Self::Meta => "meta",
        }
    }
}

/// Arguments of `roadmap_view`; all optional.
#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct RoadmapViewArgs {
    /// How the nodes reach the page: `result` (default), `chunks` or `stream`.
    #[serde(default)]
    mode: Option<Mode>,
    /// Repeat the fixture this many times, 1 to 100 (default 1).
    #[serde(default)]
    scale: Option<u32>,
}

/// Structured output of `roadmap_view`. Which optional fields are present
/// depends on `mode`.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct RoadmapView {
    campaign: String,
    frozen_at: String,
    mode: String,
    scale: u32,
    /// Milliseconds since the Unix epoch, taken just before the result is returned.
    t_server_ms: u64,
    /// Total serialised bytes of the scaled node list, markdown included.
    payload_bytes: u64,
    node_count: u64,
    /// Present in `result` mode only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nodes: Option<Vec<Value>>,
    /// Present in `chunks` mode only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chunk_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chunk_uri_template: Option<String>,
    /// Present in `stream` mode only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stream_url: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LeafGetArgs {
    /// Leaf path as listed in the fixture, for example `core/m1_core_foundation.md`.
    path: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct LeafGetResult {
    path: String,
    markdown: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
enum RunKind {
    Cold,
    Warm,
    Unknown,
}

/// One probe measurement sent by the view.
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
struct ProbeRecord {
    run_kind: RunKind,
    t_server_ms: u64,
    t_view_ms: u64,
    apps_protocol_announced: String,
    #[serde(default)]
    apps_protocol_host: Option<String>,
    #[serde(default)]
    host_capabilities: Value,
    #[serde(default)]
    host_context: Value,
    #[serde(default)]
    display_mode: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProbeAck {
    recorded: bool,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

fn meta_from(value: Value) -> MetaObject {
    match value {
        Value::Object(map) => MetaObject(map),
        _ => MetaObject::new(),
    }
}

/// `_meta` of the model-visible tool: points the host at the page.
fn ui_resource_meta() -> MetaObject {
    meta_from(json!({ "ui": { "resourceUri": UI_URI } }))
}

/// `_meta` of the app-only tools: hidden from the model.
fn ui_app_only_meta() -> MetaObject {
    meta_from(json!({ "ui": { "visibility": ["app"] } }))
}

fn internal(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

/// Largest accepted `scale`.
const MAX_SCALE: u32 = 100;
/// Largest `scale` in `meta` mode.
const META_MAX_SCALE: u32 = 10;
/// Target size of one chunk, in serialised bytes.
const CHUNK_BYTES: usize = 64 * 1024;
/// Port of the localhost SSE listener, set once in `main`.
static STREAM_PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();

/// Markdown of a leaf; a `copy<k>/` prefix is stripped first.
fn leaf_markdown(path: &str) -> Option<&'static str> {
    let base = match path.strip_prefix("copy") {
        Some(rest) => match rest.split_once('/') {
            Some((k, tail)) if !k.is_empty() && k.bytes().all(|b| b.is_ascii_digit()) => tail,
            _ => path,
        },
        None => path,
    };
    LEAVES.iter().find(|(p, _)| *p == base).map(|(_, m)| *m)
}

/// The fixture repeated `scale` times, breadth-first within each copy. Copy
/// k > 0 prefixes every path and parent with `copy<k>/`; titles, statuses and
/// depths are unchanged and each leaf carries its markdown.
fn scaled_nodes(scale: u32) -> Result<Vec<ViewNode>, ErrorData> {
    let fixture: Fixture = serde_json::from_str(ROADMAP_JSON).map_err(internal)?;
    let mut out = Vec::with_capacity(fixture.nodes.len() * scale as usize);
    for k in 0..scale {
        for node in &fixture.nodes {
            let mut node = node.clone();
            let markdown = (node.kind == "leaf")
                .then(|| leaf_markdown(&node.path).map(str::to_owned))
                .flatten();
            if k > 0 {
                node.path = format!("copy{k}/{}", node.path);
                node.parent = node.parent.map(|p| format!("copy{k}/{p}"));
            }
            out.push(ViewNode { node, markdown });
        }
    }
    Ok(out)
}

/// Splits nodes into chunks of about `CHUNK_BYTES` serialised bytes.
fn chunk_nodes(nodes: &[ViewNode]) -> Vec<&[ViewNode]> {
    let mut chunks = Vec::new();
    let (mut start, mut size) = (0, 0);
    for (i, n) in nodes.iter().enumerate() {
        size += serde_json::to_vec(n).map_or(0, |v| v.len() + 1);
        if size >= CHUNK_BYTES {
            chunks.push(&nodes[start..=i]);
            start = i + 1;
            size = 0;
        }
    }
    if start < nodes.len() {
        chunks.push(&nodes[start..]);
    }
    chunks
}

fn query_scale(query: &str) -> Option<u32> {
    let v = query.split('&').find_map(|kv| kv.strip_prefix("scale="))?;
    v.parse().ok().filter(|s| (1..=MAX_SCALE).contains(s))
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let Some(h) = s.get(i + 1..i + 3)
            && let Ok(v) = u8::from_str_radix(h, 16)
        {
            out.push(v);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Spike-only SSE listener on loopback: one event per node, then `done`.
fn serve_stream(mut conn: std::net::TcpStream) -> std::io::Result<()> {
    use std::io::{BufRead, BufReader};
    let mut request_line = String::new();
    {
        let mut reader = BufReader::new(&conn);
        reader.read_line(&mut request_line)?;
        let mut line = String::new();
        while reader.read_line(&mut line)? > 2 {
            line.clear();
        }
    }
    let mut parts = request_line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if method != "GET" || path != "/stream" {
        return conn.write_all(
            b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
    }
    let Some(scale) = query_scale(query) else {
        return conn.write_all(
            b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
    };
    conn.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\
          Access-Control-Allow-Origin: *\r\nCache-Control: no-store\r\n\
          Connection: close\r\n\r\n",
    )?;
    conn.flush()?;
    let nodes = scaled_nodes(scale).map_err(|e| std::io::Error::other(e.message.to_string()))?;
    for n in &nodes {
        let json = serde_json::to_string(n).map_err(std::io::Error::other)?;
        write!(conn, "data: {json}\n\n")?;
        conn.flush()?;
    }
    conn.write_all(b"event: done\ndata: {}\n\n")?;
    conn.flush()
}

/// Binds `127.0.0.1:0` and serves the stream on background threads.
fn start_stream_listener() -> std::io::Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    std::thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            std::thread::spawn(move || {
                let _ = serve_stream(conn);
            });
        }
    });
    Ok(port)
}

/// Page-level CSP: lets the page connect to the stream listener.
fn ui_page_meta() -> MetaObject {
    let origin = STREAM_PORT
        .get()
        .map_or_else(Vec::new, |p| vec![format!("http://127.0.0.1:{p}")]);
    meta_from(json!({ "ui": { "csp": { "connectDomains": origin } } }))
}

/// The spike server. It keeps no state between calls.
#[derive(Clone)]
struct RdmSpike {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RdmSpike {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Show the frozen roadmap as an interactive graph.",
        output_schema = schema_for_output::<RoadmapView>(),
        meta = ui_resource_meta()
    )]
    async fn roadmap_view(
        &self,
        Parameters(args): Parameters<RoadmapViewArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let mode = args.mode.unwrap_or(Mode::Result);
        let scale = args.scale.unwrap_or(1);
        if !(1..=MAX_SCALE).contains(&scale) {
            return Err(ErrorData::invalid_params(
                format!("scale must be an integer from 1 to {MAX_SCALE}"),
                None,
            ));
        }
        if matches!(mode, Mode::Meta) && scale > META_MAX_SCALE {
            return Err(ErrorData::invalid_params(
                format!("mode \"meta\" takes scale 1 to {META_MAX_SCALE}"),
                None,
            ));
        }
        if matches!(mode, Mode::Result) && scale > 1 {
            return Err(ErrorData::invalid_params(
                "mode \"result\" cannot take scale > 1: the result enters the model context; \
                 use mode \"chunks\" or \"stream\""
                    .to_owned(),
                None,
            ));
        }
        let fixture: Fixture = serde_json::from_str(ROADMAP_JSON).map_err(internal)?;
        let nodes = scaled_nodes(scale)?;
        let payload_bytes = serde_json::to_vec(&nodes).map_err(internal)?.len();
        let mut view = RoadmapView {
            campaign: fixture.campaign,
            frozen_at: fixture.frozen_at,
            mode: mode.as_str().to_owned(),
            scale,
            t_server_ms: 0,
            payload_bytes: u64::try_from(payload_bytes).map_err(internal)?,
            node_count: u64::try_from(nodes.len()).map_err(internal)?,
            nodes: None,
            chunk_count: None,
            chunk_uri_template: None,
            stream_url: None,
        };
        match mode {
            Mode::Result => {
                view.nodes = Some(
                    nodes
                        .iter()
                        .map(serde_json::to_value)
                        .collect::<Result<_, _>>()
                        .map_err(internal)?,
                );
            }
            Mode::Chunks => {
                view.chunk_count =
                    Some(u64::try_from(chunk_nodes(&nodes).len()).map_err(internal)?);
                view.chunk_uri_template = Some(format!("rdm://chunk/{{n}}?scale={scale}"));
            }
            Mode::Stream => {
                let port = STREAM_PORT
                    .get()
                    .ok_or_else(|| internal("stream listener is not bound"))?;
                view.stream_url = Some(format!("http://127.0.0.1:{port}/stream?scale={scale}"));
            }
            Mode::Meta => {}
        }
        let summary = format!(
            "Roadmap {} at {}: {} nodes, mode {}, scale {}.",
            view.campaign, view.frozen_at, view.node_count, view.mode, scale
        );
        view.t_server_ms = now_ms();
        let mut value = serde_json::to_value(&view).map_err(internal)?;
        let mut result_meta = None;
        if matches!(mode, Mode::Meta) {
            // structuredContent keeps only the small summary; the nodes ride
            // in the result-level `_meta`.
            if let Value::Object(map) = &mut value {
                for k in ["nodes", "chunk_count", "chunk_uri_template", "stream_url"] {
                    map.remove(k);
                }
            }
            let nodes = serde_json::to_value(&nodes).map_err(internal)?;
            result_meta = Some(meta_from(json!({ "rdm": { "nodes": nodes } })));
        }
        let mut result = CallToolResult::structured(value).with_meta(result_meta);
        result.content = vec![ContentBlock::text(summary)];
        Ok(result)
    }

    #[tool(
        description = "Return the markdown of one frozen leaf.",
        meta = ui_app_only_meta()
    )]
    async fn leaf_get(
        &self,
        Parameters(args): Parameters<LeafGetArgs>,
    ) -> Result<Json<LeafGetResult>, ErrorData> {
        let markdown = leaf_markdown(&args.path).ok_or_else(|| {
            ErrorData::invalid_params(format!("unknown leaf: {}", args.path), None)
        })?;
        Ok(Json(LeafGetResult {
            path: args.path,
            markdown: markdown.to_owned(),
        }))
    }

    #[tool(
        description = "Append one probe measurement to $RDM_PROBE_DIR/probe.jsonl.",
        meta = ui_app_only_meta()
    )]
    async fn probe_record(
        &self,
        Parameters(record): Parameters<ProbeRecord>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ProbeAck>, ErrorData> {
        let Some(dir) = std::env::var_os("RDM_PROBE_DIR") else {
            eprintln!("rdm-mcp: RDM_PROBE_DIR is unset; probe record dropped");
            return Ok(Json(ProbeAck { recorded: false }));
        };
        let mut line = serde_json::to_value(&record).map_err(internal)?;
        if let Value::Object(map) = &mut line {
            map.insert(
                "mcp_protocol".into(),
                context
                    .protocol_version()
                    .map_or(Value::Null, |v| Value::String(v.as_str().to_owned())),
            );
            map.insert("t_recorded_ms".into(), json!(now_ms()));
            // Tags the server process, so the first first-paint probe of each
            // process can be told apart as a cold run when the host gives the
            // page no `?run=` flag.
            map.insert("server_pid".into(), json!(std::process::id()));
            map.insert(
                "server_started_ms".into(),
                json!(SERVER_STARTED_MS.get().copied()),
            );
        }
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).map_err(internal)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("probe.jsonl"))
            .map_err(internal)?;
        writeln!(file, "{line}").map_err(internal)?;
        Ok(Json(ProbeAck { recorded: true }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for RdmSpike {
    fn get_info(&self) -> ServerConfig {
        let mut capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();
        // rmcp has no typed helper for the MCP Apps extension.
        let mut extensions = ExtensionCapabilities::new();
        extensions.insert(
            APPS_EXTENSION.to_owned(),
            json!({ "mimeTypes": [APPS_MIME] })
                .as_object()
                .cloned()
                .unwrap_or_default(),
        );
        capabilities.extensions = Some(extensions);
        ServerConfig::new(capabilities)
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(Implementation::new("rdm-mcp", env!("CARGO_PKG_VERSION")))
    }

    /// Accept 2026-07-28 only: any other version, on `initialize` or in a
    /// request's `_meta`, is refused with an unsupported-protocol-version error.
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(&[ProtocolVersion::V_2026_07_28])
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let page = Resource::new(UI_URI, "rdm graph")
            .with_description("Interactive roadmap graph")
            .with_mime_type(APPS_MIME)
            .with_meta(ui_page_meta());
        Ok(ListResourcesResult::with_all_items(vec![page]))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            ResourceTemplate::new("rdm://leaf/{path}", "leaf")
                .with_description("Markdown of one frozen leaf")
                .with_mime_type("text/markdown"),
            ResourceTemplate::new("rdm://chunk/{n}?scale={scale}", "chunk")
                .with_description("One chunk of the scaled node list")
                .with_mime_type("application/json"),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let uri = request.uri.as_str();
        let bad = |m: String| ErrorData::invalid_params(m, None);
        let contents = if uri == UI_URI {
            ResourceContents::text(VIEW_HTML, UI_URI)
                .with_mime_type(APPS_MIME)
                .with_meta(ui_page_meta())
        } else if let Some(path) = uri.strip_prefix("rdm://leaf/") {
            let path = percent_decode(path);
            let md = leaf_markdown(&path).ok_or_else(|| bad(format!("unknown leaf: {path}")))?;
            ResourceContents::text(md, uri).with_mime_type("text/markdown")
        } else if let Some(rest) = uri.strip_prefix("rdm://chunk/") {
            let (n, query) = rest.split_once('?').unwrap_or((rest, ""));
            let n: usize = n
                .parse()
                .map_err(|_| bad(format!("bad chunk index: {n}")))?;
            let scale =
                query_scale(query).ok_or_else(|| bad("scale must be 1 to 100".to_owned()))?;
            let nodes = scaled_nodes(scale)?;
            let chunks = chunk_nodes(&nodes);
            let chunk = chunks
                .get(n)
                .ok_or_else(|| bad(format!("chunk {n} out of range (0..{})", chunks.len())))?;
            let body =
                serde_json::to_string(&json!({ "n": n, "nodes": chunk })).map_err(internal)?;
            ResourceContents::text(body, uri).with_mime_type("application/json")
        } else {
            return Err(bad(format!("unknown resource: {uri}")));
        };
        Ok(ReadResourceResult::new(vec![contents]).into())
    }
}

/// When this server process started, in epoch milliseconds (set once in `main`).
static SERVER_STARTED_MS: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = SERVER_STARTED_MS.set(now_ms());
    let _ = STREAM_PORT.set(start_stream_listener()?);
    let service = RdmSpike::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
