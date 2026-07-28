#!/usr/bin/env python3
"""Controlled OpenAI-compatible Router used by installer end-to-end tests."""

import argparse
import json
import threading
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class RouterHandler(BaseHTTPRequestHandler):
    server_version = "CodexAssistantTestRouter/1"

    def do_GET(self):
        if self.path == "/health":
            self.send_json(200, {"status": "ok"})
            return
        if self.path != "/v1/models":
            self.send_json(404, {"error": {"message": "not found"}})
            return
        if not self.authorized():
            return
        if (
            self.server.mode == "verify-models-fail"
            and self.server.completed_response_count() >= 1
        ):
            self.send_json(503, {"error": {"message": "injected verify failure"}})
            return
        self.send_json(
            200,
            {
                "object": "list",
                "data": [
                    {
                        "id": self.server.model,
                        "object": "model",
                        "owned_by": "codex-assistant-test",
                    }
                ],
            },
        )

    def do_POST(self):
        if self.path != "/v1/responses":
            self.send_json(404, {"error": {"message": "not found"}})
            return
        if not self.authorized():
            return
        payload = self.read_json()
        if payload is None:
            return
        if payload.get("model") != self.server.model:
            self.send_json(400, {"error": {"message": "model unavailable"}})
            return

        mode = self.server.mode
        if mode == "responses-404":
            self.send_json(404, {"error": {"message": "responses unsupported"}})
        elif mode == "disconnect":
            self.send_sse(
                [
                    {
                        "type": "response.output_text.delta",
                        "delta": "not retained",
                    }
                ]
            )
        elif mode == "failed":
            self.send_sse([{"type": "response.failed", "response": {"status": "failed"}}])
        elif mode == "wrong-model":
            self.send_sse([self.completed_event(f"{self.server.model}-mapped")])
        elif mode == "json":
            self.send_json(200, self.completed_response(self.server.model))
        else:
            if mode == "verify-models-fail":
                self.server.record_completed_response()
            self.send_sse(
                [
                    {
                        "type": "response.created",
                        "response": {
                            "id": self.response_id(),
                            "object": "response",
                            "status": "in_progress",
                            "model": self.server.model,
                            "output": [],
                        },
                    },
                    self.completed_event(self.server.model),
                ]
            )

    def authorized(self):
        expected = self.server.key
        if not expected:
            return True
        if self.headers.get("Authorization") == f"Bearer {expected}":
            return True
        self.send_json(401, {"error": {"message": "unauthorized"}})
        return False

    def read_json(self):
        try:
            length = int(self.headers.get("Content-Length", "0"))
            if length <= 0 or length > 64 * 1024:
                raise ValueError("invalid body size")
            return json.loads(self.rfile.read(length))
        except (ValueError, json.JSONDecodeError):
            self.send_json(400, {"error": {"message": "invalid request"}})
            return None

    def send_json(self, status, payload):
        body = json.dumps(payload, separators=(",", ":")).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("x-request-id", self.request_id())
        self.end_headers()
        self.wfile.write(body)

    def send_sse(self, events):
        body = b"".join(
            (
                f"event: {event['type']}\n"
                f"data: {json.dumps(event, separators=(',', ':'))}\n\n"
            ).encode()
            for event in events
        )
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("x-request-id", self.request_id())
        self.end_headers()
        self.wfile.write(body)

    def completed_event(self, model):
        return {
            "type": "response.completed",
            "response": self.completed_response(model),
        }

    def completed_response(self, model):
        return {
            "id": self.response_id(),
            "object": "response",
            "status": "completed",
            "model": model,
            "output": [
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "output_text", "text": "OK"}],
                }
            ],
        }

    @staticmethod
    def response_id():
        return f"resp_{uuid.uuid4().hex[:16]}"

    @staticmethod
    def request_id():
        return f"req_{uuid.uuid4().hex[:16]}"

    def log_message(self, fmt, *args):
        print(f"{self.client_address[0]} {self.command} {self.path} {fmt % args}")


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=11435)
    parser.add_argument("--model", default="codex-assistant-test")
    parser.add_argument("--key", default="")
    parser.add_argument(
        "--mode",
        choices=(
            "normal",
            "json",
            "responses-404",
            "disconnect",
            "failed",
            "wrong-model",
            "verify-models-fail",
        ),
        default="normal",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    server = ThreadingHTTPServer((args.host, args.port), RouterHandler)
    server.model = args.model
    server.key = args.key
    server.mode = args.mode
    server.response_count = 0
    server.response_count_lock = threading.Lock()

    def completed_response_count():
        with server.response_count_lock:
            return server.response_count

    def record_completed_response():
        with server.response_count_lock:
            server.response_count += 1

    server.completed_response_count = completed_response_count
    server.record_completed_response = record_completed_response
    print(
        f"Test Router listening at http://{args.host}:{args.port}/v1 "
        f"(model={args.model}, mode={args.mode})",
        flush=True,
    )
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
