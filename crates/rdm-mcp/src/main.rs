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
        CallToolResult, ContentBlock, Implementation, ExtensionCapabilities, ListResourcesResult, MetaObject,
        PaginatedRequestParams, ProtocolVersion, Resource, ReadResourceRequestParams,
        ReadResourceResponse, ReadResourceResult, ResourceContents, ServerCapabilities,
        ServerConfig,
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

/// Structured output of `roadmap_view`: the fixture plus two timing fields.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct RoadmapView {
    campaign: String,
    frozen_at: String,
    nodes: Vec<FixtureNode>,
    /// Milliseconds since the Unix epoch, taken just before the result is returned.
    t_server_ms: u64,
    /// Byte length of the serialised fixture.
    payload_bytes: u64,
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
    async fn roadmap_view(&self) -> Result<CallToolResult, ErrorData> {
        let fixture: Fixture = serde_json::from_str(ROADMAP_JSON).map_err(internal)?;
        let payload_bytes = serde_json::to_vec(&fixture).map_err(internal)?.len();
        let summary = format!(
            "Roadmap {} at {}: {} nodes.",
            fixture.campaign,
            fixture.frozen_at,
            fixture.nodes.len()
        );
        let view = RoadmapView {
            campaign: fixture.campaign,
            frozen_at: fixture.frozen_at,
            nodes: fixture.nodes,
            payload_bytes: u64::try_from(payload_bytes).map_err(internal)?,
            t_server_ms: now_ms(),
        };
        let mut result =
            CallToolResult::structured(serde_json::to_value(&view).map_err(internal)?);
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
        let (_, markdown) = LEAVES
            .iter()
            .find(|(p, _)| *p == args.path)
            .ok_or_else(|| ErrorData::invalid_params(format!("unknown leaf: {}", args.path), None))?;
        Ok(Json(LeafGetResult {
            path: args.path,
            markdown: (*markdown).to_owned(),
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
            .with_mime_type(APPS_MIME);
        Ok(ListResourcesResult::with_all_items(vec![page]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        if request.uri != UI_URI {
            return Err(ErrorData::invalid_params(
                format!("unknown resource: {}", request.uri),
                None,
            ));
        }
        let contents = ResourceContents::text(VIEW_HTML, UI_URI).with_mime_type(APPS_MIME);
        Ok(ReadResourceResult::new(vec![contents]).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = RdmSpike::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
