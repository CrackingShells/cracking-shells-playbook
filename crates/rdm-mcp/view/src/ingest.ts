import { tick } from 'svelte';
import { readResourceText } from './bridge';
import type { RoadmapNode, RoadmapView, Stamps } from './types';

const nextFrame = () => new Promise<void>((r) => requestAnimationFrame(() => r()));
const utf8 = (s: string) => new TextEncoder().encode(s).length;

/** Appends nodes and paints in batches: one DOM update per animation frame. */
class Painter {
  all: RoadmapNode[] = [];
  pending: RoadmapNode[] = [];
  scheduled = false;
  firstPainted = false;
  constructor(
    private setNodes: (n: RoadmapNode[]) => void,
    private stamps: Stamps,
  ) {}
  push(batch: RoadmapNode[]) {
    if (!batch.length) return;
    if (!this.stamps.t_first_data_ms) this.stamps.t_first_data_ms = Date.now();
    this.pending.push(...batch);
    this.stamps.nodes_received += batch.length;
    if (!this.scheduled) {
      this.scheduled = true;
      requestAnimationFrame(() => {
        this.scheduled = false;
        this.all = this.all.concat(this.pending);
        this.pending = [];
        this.setNodes(this.all);
        if (!this.firstPainted) {
          this.firstPainted = true;
          // The rAF after Svelte has rendered the first nodes.
          void tick().then(() => requestAnimationFrame(() => (this.stamps.t_first_paint_ms = Date.now())));
        }
      });
    }
  }
  /** Resolves after the rAF that follows the render of the last node. */
  async finish() {
    while (this.pending.length || this.scheduled) await nextFrame();
    await tick();
    await nextFrame();
    this.stamps.t_complete_ms = Date.now();
  }
}

/**
 * Ingest by mode. Stamps are written into `stamps` as they happen.
 * Throws with the verbatim error text on a read or network failure.
 */
export async function ingest(
  sc: RoadmapView,
  stamps: Stamps,
  setNodes: (n: RoadmapNode[]) => void,
): Promise<void> {
  const painter = new Painter(setNodes, stamps);
  if (sc.mode === 'result' || sc.mode === 'meta') {
    stamps.bytes_received = utf8(JSON.stringify(sc.nodes ?? []));
    painter.push(sc.nodes ?? []);
  } else if (sc.mode === 'chunks') {
    const tpl = sc.chunk_uri_template ?? '';
    // Chunks are numbered from 0 (the contract does not say; see report).
    for (let n = 0; n < (sc.chunk_count ?? 0); n++) {
      const uri = tpl.replace('{n}', String(n)).replace(/scale=S\b/, `scale=${sc.scale}`);
      const text = await readResourceText(uri);
      stamps.bytes_received += utf8(text);
      painter.push((JSON.parse(text) as { nodes: RoadmapNode[] }).nodes);
    }
  } else if (sc.mode === 'stream') {
    await new Promise<void>((resolve, reject) => {
      let violation = '';
      const onViolation = (e: SecurityPolicyViolationEvent) => {
        violation = `CSP violation: ${e.violatedDirective} blocked ${e.blockedURI}`;
      };
      document.addEventListener('securitypolicyviolation', onViolation);
      const es = new EventSource(sc.stream_url ?? '');
      let done = false;
      es.onmessage = (e) => {
        stamps.bytes_received += utf8(e.data);
        painter.push([JSON.parse(e.data) as RoadmapNode]);
      };
      es.addEventListener('done', () => {
        done = true;
        es.close();
        document.removeEventListener('securitypolicyviolation', onViolation);
        resolve();
      });
      es.onerror = () => {
        if (done) return;
        es.close();
        document.removeEventListener('securitypolicyviolation', onViolation);
        // Verbatim: EventSource exposes no text, so give everything the page knows.
        reject(new Error(`EventSource error on ${sc.stream_url} (readyState ${es.readyState})${violation ? '; ' + violation : ''}`));
      };
    });
  } else {
    throw new Error('unknown mode: ' + String((sc as { mode?: unknown }).mode));
  }
  await painter.finish();
}
