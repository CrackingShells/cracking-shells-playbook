import type { HostInfo } from './bridge';
import type { RoadmapView, Stamps } from './types';

export function runKind(): 'cold' | 'warm' | 'unknown' {
  const v = new URLSearchParams(window.location.search).get('run');
  return v === 'cold' || v === 'warm' ? v : 'unknown';
}

// postMessage cannot clone Proxy objects; send plain JSON.
function plain(v: unknown): unknown {
  return v === undefined ? null : JSON.parse(JSON.stringify(v));
}

/** The probe object. host_context is left out on purpose: it is large and recorded elsewhere. */
export function buildProbe(
  view: RoadmapView,
  stamps: Stamps,
  host: HostInfo,
  displayMode: string,
  leafRead: unknown,
) {
  return {
    run_kind: runKind(),
    mode: view.mode,
    scale: view.scale,
    node_count: view.node_count,
    payload_bytes: view.payload_bytes,
    t_server_ms: view.t_server_ms,
    ...plain(stamps) as object,
    apps_protocol_announced: host.protocolAnnounced,
    apps_protocol_host: host.protocolHost ?? 'unknown',
    display_mode: displayMode,
    leaf_read: leafRead,
    host_capabilities: plain(host.hostCapabilities),
  };
}

/** One single-line chat message: RDM_PROBE followed by compact JSON. */
export function probeLine(probe: object): string {
  return 'RDM_PROBE ' + JSON.stringify(probe);
}
