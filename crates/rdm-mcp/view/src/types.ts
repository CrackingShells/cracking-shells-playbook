export type Status = 'planned' | 'inprogress' | 'done' | 'amendment' | 'blocked';

export interface RoadmapNode {
  path: string;
  parent: string | null;
  kind: 'leaf' | 'dir';
  title: string;
  status: Status;
  depth: number;
}

/** structuredContent of roadmap_view: the fixture plus the server's own numbers. */
export interface RoadmapView {
  campaign: string;
  frozen_at: string;
  nodes: RoadmapNode[];
  t_server_ms: number;
  payload_bytes: number;
}

/** The five legacy classDef colours of the roadmap READMEs. */
export const STATUS_COLORS: Record<Status, { fill: string; color: string }> = {
  done: { fill: '#166534', color: '#bbf7d0' },
  inprogress: { fill: '#854d0e', color: '#fef08a' },
  planned: { fill: '#374151', color: '#e5e7eb' },
  amendment: { fill: '#1e3a5f', color: '#bfdbfe' },
  blocked: { fill: '#7f1d1d', color: '#fecaca' },
};
