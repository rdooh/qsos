#!/usr/bin/env python3
"""
QSOS log viewer server.
Serves utilities/log-viewer.html and exposes /api/logs returning all JSONL events.
Usage: python3 utilities/serve.py [port]
"""
import http.server, json, os, sys, glob

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=ROOT, **kwargs)

    def do_GET(self):
        if self.path == "/api/logs":
            self.serve_logs()
        else:
            super().do_GET()

    def serve_logs(self):
        pattern = os.path.join(ROOT, "work", "**", "logs", "qsos-run-*.jsonl")
        fallback = os.path.join(ROOT, ".qsos", "logs", "qsos-run-*.jsonl")

        events = []
        for path in sorted(glob.glob(pattern, recursive=True) + glob.glob(fallback)):
            try:
                with open(path) as f:
                    for line in f:
                        line = line.strip()
                        if line:
                            try:
                                events.append(json.loads(line))
                            except json.JSONDecodeError:
                                pass
            except OSError:
                pass

        body = json.dumps(events).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", len(body))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt, *args):
        pass  # suppress per-request noise


if __name__ == "__main__":
    os.chdir(ROOT)
    print(f"QSOS log viewer → http://localhost:{PORT}/utilities/log-viewer.html")
    http.server.HTTPServer(("", PORT), Handler).serve_forever()
