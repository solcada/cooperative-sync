#!/usr/bin/env python3
"""Minimal mock sync server for local client development."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import argparse


class SyncHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        if self.path == "/v1/health":
            self._send(200, b"ok")
            return

        self._send(404, b"not found")

    def do_POST(self) -> None:
        if self.path != "/v1/sync":
            self._send(404, b"not found")
            return

        content_length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(content_length).decode("utf-8", errors="replace")

        # This log helps verify what the Rust client sent.
        print("[mock-sync-server] received sync request:")
        print(body)

        if self.server.fail_sync:
            self._send(500, b"sync failed")
            return

        self._send(202, b"accepted")

    def log_message(self, format: str, *args) -> None:
        # Keep output focused on sync payloads.
        return

    def _send(self, status: int, body: bytes) -> None:
        self.send_response(status)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def main() -> None:
    parser = argparse.ArgumentParser(description="Run a local mock sync server.")
    parser.add_argument("--host", default="127.0.0.1", help="Bind host")
    parser.add_argument("--port", type=int, default=8080, help="Bind port")
    parser.add_argument(
        "--fail-sync",
        action="store_true",
        help="Return 500 for /v1/sync to test client recovery behavior",
    )
    args = parser.parse_args()

    server = ThreadingHTTPServer((args.host, args.port), SyncHandler)
    server.fail_sync = args.fail_sync

    mode = "fail" if args.fail_sync else "normal"
    print(f"[mock-sync-server] listening on http://{args.host}:{args.port} ({mode} mode)")
    print("[mock-sync-server] endpoints: GET /v1/health, POST /v1/sync")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
