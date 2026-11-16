import { callAppTool } from './bridge';
import type { HostInfo } from './bridge';
import type { RoadmapView } from './types';

export type DisplayModeRan = 'fullscreen' | 'outline' | 'inline';

function runKind(): 'cold' | 'warm' | 'unknown' {
  const v = new URLSearchParams(window.location.search).get('run');
  return v === 'cold' || v === 'warm' ? v : 'unknown';
}

/** Takes t_view_ms in the first animation frame after the graph mounted. */
export function nextFrameMs(): Promise<number> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve(Date.now())));
}

// postMessage cannot clone Proxy objects; send plain JSON.
function plain(v: unknown): unknown {
  return v === undefined ? null : JSON.parse(JSON.stringify(v));
}

async function send(view: RoadmapView, host: HostInfo, tViewMs: number, displayMode: string) {
  // Field names are the probe_record contract of build/stub_server.md Step 2.
  return callAppTool('probe_record', {
    run_kind: runKind(),
    t_server_ms: view.t_server_ms,
    t_view_ms: tViewMs,
    apps_protocol_announced: host.protocolAnnounced,
    apps_protocol_host: host.protocolHost ?? 'unknown',
    host_capabilities: plain(host.hostCapabilities),
    host_context: plain(host.hostContext),
    display_mode: displayMode,
  });
}

/** First paint: one probe_record, display_mode taken from the host's current mode. */
export async function recordFirstPaint(view: RoadmapView, host: HostInfo, tViewMs: number): Promise<void> {
  const ctx = host.hostContext as { displayMode?: string } | undefined;
  await send(view, host, tViewMs, ctx?.displayMode ?? 'inline');
}

/** Full view opened: a second probe_record saying which path ran (fullscreen or outline). */
export async function recordFullView(
  view: RoadmapView,
  host: HostInfo,
  tViewMs: number,
  ran: DisplayModeRan,
): Promise<void> {
  await send(view, host, tViewMs, ran);
}
