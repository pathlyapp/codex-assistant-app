#!/usr/bin/env python3
import argparse
import asyncio
import os
import plistlib
import re
import shutil
import socket
import subprocess
import sys
import time
from pathlib import Path


LABEL = "com.company.codex-assistant.ollama-proxy"
APP_DIR = Path.home() / "Library" / "Application Support" / "CodexAssistant"
INSTALLED_SCRIPT = APP_DIR / "parallels-ollama-proxy.py"
LAUNCH_AGENT = Path.home() / "Library" / "LaunchAgents" / f"{LABEL}.plist"
LOG_DIR = Path.home() / "Library" / "Logs" / "CodexAssistant"


def bridge_address(interface):
    try:
        result = subprocess.run(
            ["/sbin/ifconfig", interface],
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise RuntimeError(f"Cannot inspect Parallels interface {interface}: {error}")
    match = re.search(r"^\s*inet\s+(\d+\.\d+\.\d+\.\d+)\s", result.stdout, re.MULTILINE)
    if not match:
        raise RuntimeError(f"Parallels interface {interface} has no IPv4 address")
    return match.group(1)


def resolved_listen_host(args):
    return args.listen_host or bridge_address(args.interface)


def endpoint(host, port):
    return f"http://{host}:{port}/v1"


def can_connect(host, port, timeout=1.5):
    try:
        with socket.create_connection((host, port), timeout=timeout):
            return True
    except OSError:
        return False


async def pipe(reader, writer):
    try:
        while data := await reader.read(65536):
            writer.write(data)
            await writer.drain()
    finally:
        writer.close()
        try:
            await writer.wait_closed()
        except (BrokenPipeError, ConnectionResetError):
            pass


async def handle(client_reader, client_writer, upstream_host, upstream_port):
    try:
        upstream_reader, upstream_writer = await asyncio.open_connection(
            upstream_host, upstream_port
        )
    except OSError:
        client_writer.close()
        await client_writer.wait_closed()
        return
    await asyncio.gather(
        pipe(client_reader, upstream_writer),
        pipe(upstream_reader, client_writer),
        return_exceptions=True,
    )


async def run_proxy(args):
    listen_host = resolved_listen_host(args)
    if not can_connect(args.upstream_host, args.upstream_port):
        raise RuntimeError(
            f"Ollama is not listening at {args.upstream_host}:{args.upstream_port}"
        )
    server = await asyncio.start_server(
        lambda reader, writer: handle(
            reader, writer, args.upstream_host, args.upstream_port
        ),
        listen_host,
        args.listen_port,
    )
    print(
        f"Forwarding {listen_host}:{args.listen_port} "
        f"to {args.upstream_host}:{args.upstream_port}",
        flush=True,
    )
    print(f"Windows Router URL: {endpoint(listen_host, args.listen_port)}", flush=True)
    async with server:
        await server.serve_forever()


def launchctl(*arguments, check=True, capture_output=False):
    return subprocess.run(
        ["/bin/launchctl", *arguments],
        check=check,
        capture_output=capture_output,
        text=True,
    )


def launch_domain():
    return f"gui/{os.getuid()}"


def install_service(args):
    if sys.platform != "darwin":
        raise RuntimeError("The Parallels Ollama bridge can only be installed on macOS")
    listen_host = resolved_listen_host(args)
    if not can_connect(args.upstream_host, args.upstream_port):
        raise RuntimeError(
            f"Start Ollama first; nothing is listening at "
            f"{args.upstream_host}:{args.upstream_port}"
        )

    APP_DIR.mkdir(parents=True, exist_ok=True)
    LAUNCH_AGENT.parent.mkdir(parents=True, exist_ok=True)
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    shutil.copy2(Path(__file__).resolve(), INSTALLED_SCRIPT)
    INSTALLED_SCRIPT.chmod(0o755)

    program_arguments = [
        sys.executable,
        str(INSTALLED_SCRIPT),
        "run",
        "--listen-host",
        listen_host,
        "--listen-port",
        str(args.listen_port),
        "--upstream-host",
        args.upstream_host,
        "--upstream-port",
        str(args.upstream_port),
    ]
    payload = {
        "Label": LABEL,
        "ProgramArguments": program_arguments,
        "RunAtLoad": True,
        "KeepAlive": True,
        "ProcessType": "Background",
        "StandardOutPath": str(LOG_DIR / "ollama-proxy.log"),
        "StandardErrorPath": str(LOG_DIR / "ollama-proxy.error.log"),
    }
    with LAUNCH_AGENT.open("wb") as stream:
        plistlib.dump(payload, stream, sort_keys=False)

    domain = launch_domain()
    launchctl("bootout", domain, str(LAUNCH_AGENT), check=False, capture_output=True)
    launchctl("bootstrap", domain, str(LAUNCH_AGENT))
    launchctl("enable", f"{domain}/{LABEL}")
    launchctl("kickstart", "-k", f"{domain}/{LABEL}")

    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        if can_connect(listen_host, args.listen_port):
            break
        time.sleep(0.25)
    else:
        raise RuntimeError(
            f"Bridge service started but {listen_host}:{args.listen_port} is not reachable. "
            f"Check {LOG_DIR / 'ollama-proxy.error.log'}"
        )
    print("Parallels Ollama bridge installed and running.")
    print(f"Windows Router URL: {endpoint(listen_host, args.listen_port)}")


def uninstall_service(_args):
    domain = launch_domain()
    launchctl("bootout", domain, str(LAUNCH_AGENT), check=False, capture_output=True)
    LAUNCH_AGENT.unlink(missing_ok=True)
    INSTALLED_SCRIPT.unlink(missing_ok=True)
    print("Parallels Ollama bridge removed.")


def service_status(args):
    listen_host = resolved_listen_host(args)
    result = launchctl(
        "print", f"{launch_domain()}/{LABEL}", check=False, capture_output=True
    )
    loaded = result.returncode == 0
    reachable = can_connect(listen_host, args.listen_port)
    print(f"LaunchAgent: {'loaded' if loaded else 'not installed'}")
    print(f"Bridge: {'reachable' if reachable else 'not reachable'}")
    print(f"Windows Router URL: {endpoint(listen_host, args.listen_port)}")
    if not loaded or not reachable:
        raise RuntimeError("Parallels Ollama bridge is not ready")


def build_parser():
    parser = argparse.ArgumentParser(
        description="Expose macOS Ollama only to the Parallels virtual network."
    )
    parser.add_argument(
        "action", nargs="?", choices=("run", "install", "uninstall", "status"), default="run"
    )
    parser.add_argument("--interface", default="bridge100")
    parser.add_argument("--listen-host")
    parser.add_argument("--listen-port", type=int, default=11434)
    parser.add_argument("--upstream-host", default="127.0.0.1")
    parser.add_argument("--upstream-port", type=int, default=11434)
    return parser


def main():
    args = build_parser().parse_args()
    try:
        if args.action == "run":
            asyncio.run(run_proxy(args))
        elif args.action == "install":
            install_service(args)
        elif args.action == "uninstall":
            uninstall_service(args)
        else:
            service_status(args)
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
