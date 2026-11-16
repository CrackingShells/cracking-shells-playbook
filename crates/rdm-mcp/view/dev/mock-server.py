#!/usr/bin/env python3
"""Dev only. Static files for the mock host plus an SSE endpoint /stream?scale=S.
Run from crates/rdm-mcp/view:  python3 dev/mock-server.py 8765"""
import json, sys, time
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs

BASE = json.load(open("dev/sample-roadmap.json"))["nodes"]


def scaled(scale):
    for i in range(scale):
        for n in BASE:
            if scale == 1:
                yield n
            else:
                yield {**n, "path": f"s{i}/{n['path']}", "parent": f"s{i}/{n['parent']}" if n["parent"] else f"s{i}"}


class H(SimpleHTTPRequestHandler):
    def do_GET(self):
        u = urlparse(self.path)
        if u.path != "/stream":
            return super().do_GET()
        scale = int(parse_qs(u.query).get("scale", ["1"])[0])
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        try:
            for n in scaled(scale):
                self.wfile.write(f"data: {json.dumps(n)}\n\n".encode())
                self.wfile.flush()
                time.sleep(0.005)
            self.wfile.write(b"event: done\ndata: {}\n\n")
            self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            pass

    def log_message(self, *a):
        pass


if __name__ == "__main__":
    ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
