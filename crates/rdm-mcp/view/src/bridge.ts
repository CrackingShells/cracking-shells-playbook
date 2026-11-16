// MCP Apps bridge, built on the App class of @modelcontextprotocol/ext-apps.
import { App, PostMessageTransport, LATEST_PROTOCOL_VERSION } from '@modelcontextprotocol/ext-apps';

export type ToolResultParams = Parameters<NonNullable<App['ontoolresult']>>[0];

export interface HostInfo {
  /** Apps protocolVersion this view announced in ui/initialize. */
  protocolAnnounced: string;
  /** Apps protocolVersion the host answered with. Not exposed by App, so read from the host reply when available. */
  protocolHost: string | null;
  hostCapabilities: unknown;
  hostContext: unknown;
  hostInfo: unknown;
}

export const app = new App({ name: 'rdm-graph-view', version: '0.0.0' }, {}, { autoResize: true });

/**
 * Handshake: ui/initialize, then ui/notifications/initialized (both done by app.connect).
 * onToolResult is registered BEFORE connect, as the library requires.
 */
export async function connect(
  onToolResult: (p: ToolResultParams) => void,
  onHostContext: (ctx: unknown) => void,
): Promise<HostInfo> {
  app.ontoolresult = onToolResult;
  app.onhostcontextchanged = () => onHostContext(app.getHostContext());
  let protocolHost: string | null = null;
  const transport = new PostMessageTransport(window.parent, window.parent);
  // Observe the host's ui/initialize reply to keep its protocolVersion (App discards it).
  window.addEventListener('message', (e) => {
    const d = e.data as { result?: { protocolVersion?: string; hostCapabilities?: unknown } } | null;
    if (d && d.result && typeof d.result.protocolVersion === 'string' && d.result.hostCapabilities) {
      protocolHost = d.result.protocolVersion;
    }
  });
  await app.connect(transport);
  return {
    protocolAnnounced: LATEST_PROTOCOL_VERSION,
    protocolHost,
    hostCapabilities: app.getHostCapabilities(),
    hostContext: app.getHostContext(),
    hostInfo: app.getHostVersion(),
  };
}

/** Call a server tool (tools/call) and return its structuredContent, or null. */
export async function callAppTool<T = unknown>(name: string, args: Record<string, unknown> = {}): Promise<T | null> {
  try {
    const res = await app.callServerTool({ name, arguments: args });
    return (res.structuredContent as T | undefined) ?? null;
  } catch (e) {
    // Surface, never hide: every failed tool call is kept for the diagnostics panel.
    toolErrorListener?.(name, String(e));
    throw e;
  }
}

let toolErrorListener: ((tool: string, text: string) => void) | null = null;
/** The diagnostics panel registers here to see the last error of every failed tool call. */
export function onToolError(fn: (tool: string, text: string) => void): void {
  toolErrorListener = fn;
}

/** ui/update-model-context, only if the host advertises the updateModelContext capability. */
export async function sendModelContext(structured: Record<string, unknown>): Promise<string> {
  const caps = app.getHostCapabilities() as { updateModelContext?: unknown } | undefined;
  if (!caps?.updateModelContext) return 'skipped: host does not advertise updateModelContext';
  try {
    await app.updateModelContext({
      content: [{ type: 'text', text: JSON.stringify(structured) }],
      structuredContent: structured,
    });
    return 'ok';
  } catch (e) {
    const t = String(e);
    toolErrorListener?.('ui/update-model-context', t);
    return 'failed: ' + t;
  }
}

/** Ask the host for a display mode (ui/request-display-mode). */
export async function requestDisplayMode(mode: 'inline' | 'fullscreen' | 'pip'): Promise<string> {
  const r = await app.requestDisplayMode({ mode });
  return r.mode;
}
