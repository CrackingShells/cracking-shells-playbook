import { connect } from './bridge';

const el = document.getElementById('app')!;
el.textContent = 'connecting...';
connect(
  (p) => {
    el.textContent = 'tool-result received: ' + JSON.stringify(p.structuredContent ?? null).slice(0, 80);
  },
  () => {},
).then(
  (h) => {
    el.textContent = 'connected, host protocol ' + h.protocolHost;
  },
  (e) => {
    el.textContent = 'connect failed: ' + String(e);
  },
);
