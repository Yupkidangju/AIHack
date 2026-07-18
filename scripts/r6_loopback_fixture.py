#!/usr/bin/env python3
"""R6 PTY 감사를 위한 결정적 OpenAI-compatible loopback fixture."""

from __future__ import annotations

import argparse
import json
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path


def response_payload(kind: str) -> dict[str, object]:
    if kind == "DECISION":
        return {
            "kind": "DECISION",
            "action": {"type": "WAIT"},
            "rationale": "Deterministic fixture suggestion.",
            "confidence": 0.75,
        }
    if kind == "SOFT_ADJUDICATION":
        return {
            "kind": "SOFT_ADJUDICATION",
            "verdict": "NEUTRAL",
            "reasonCode": "FIXTURE_NEUTRAL",
            "message": "Deterministic fixture judgment.",
        }
    return {"kind": "NARRATIVE", "text": "Deterministic fixture narrative."}


class FixtureHandler(BaseHTTPRequestHandler):
    server_version = "AIHackR6Fixture/1"

    def do_POST(self) -> None:  # noqa: N802 - BaseHTTPRequestHandler contract
        if self.path != "/v1/chat/completions":
            self.send_error(404)
            return

        length = int(self.headers.get("Content-Length", "0"))
        request = json.loads(self.rfile.read(length))
        canonical = json.loads(request["messages"][1]["content"])
        payload = response_payload(canonical.get("kind", "NARRATIVE"))
        content = json.dumps(payload, separators=(",", ":"))
        body = json.dumps(
            {"choices": [{"message": {"content": content}}]},
            separators=(",", ":"),
        ).encode("utf-8")

        time.sleep(self.server.delay_ms / 1000)  # type: ignore[attr-defined]
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        try:
            self.wfile.write(body)
        except BrokenPipeError:
            pass

    def log_message(self, _format: str, *_args: object) -> None:
        return


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=0)
    parser.add_argument("--delay-ms", type=int, default=0)
    parser.add_argument("--max-requests", type=int, default=1)
    parser.add_argument("--ready-file", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if not 0 <= args.port <= 65535 or args.delay_ms < 0 or args.max_requests < 1:
        raise SystemExit("invalid fixture arguments")

    server = HTTPServer(("127.0.0.1", args.port), FixtureHandler)
    server.delay_ms = args.delay_ms  # type: ignore[attr-defined]
    args.ready_file.write_text(str(server.server_port), encoding="utf-8")
    for _ in range(args.max_requests):
        server.handle_request()
    server.server_close()


if __name__ == "__main__":
    main()
