export type Status = 'planned' | 'inprogress' | 'done' | 'amendment' | 'blocked';

export interface RoadmapNode {
  path: string;
  parent: string | null;
  kind: 'leaf' | 'dir';
  title: string;
  status: Status;
  depth: number;
  /** Present only in result mode at scale 1. */
  markdown?: string;
}

export type Mode = 'result' | 'chunks' | 'stream' | 'meta';

/** structuredContent of roadmap_view (round 2 contract). nodes is present in result mode only. */
export interface RoadmapView {
  campaign?: string;
  mode: Mode;
  scale: number;
  t_server_ms: number;
  payload_bytes: number;
  node_count: number;
  nodes?: RoadmapNode[];
  chunk_count?: number;
  chunk_uri_template?: string;
  stream_url?: string;
}

export interface Stamps {
  t_result_ms: number;
  t_first_data_ms: number;
  t_first_paint_ms: number;
  t_complete_ms: number;
  bytes_received: number;
  nodes_received: number;
}

/** The five legacy classDef colours of the roadmap READMEs. */
export const STATUS_COLORS: Record<Status, { fill: string; color: string }> = {
  done: { fill: '#166534', color: '#bbf7d0' },
  inprogress: { fill: '#854d0e', color: '#fef08a' },
  planned: { fill: '#374151', color: '#e5e7eb' },
  amendment: { fill: '#1e3a5f', color: '#bfdbfe' },
  blocked: { fill: '#7f1d1d', color: '#fecaca' },
};
