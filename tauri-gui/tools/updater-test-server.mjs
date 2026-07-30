import { createReadStream, readFileSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { basename, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const MODES = new Set(["available", "none", "invalid-signature", "error"]);
const TARGETS = new Set(["windows", "darwin"]);
const ARCHITECTURES = new Set(["x86_64", "aarch64"]);

function compareVersions(left, right) {
  const parse = (value) =>
    String(value)
      .replace(/^v/, "")
      .split(/[+-]/, 1)[0]
      .split(".")
      .map((part) => Number.parseInt(part, 10) || 0);
  const a = parse(left);
  const b = parse(right);
  for (let index = 0; index < Math.max(a.length, b.length); index += 1) {
    const difference = (a[index] || 0) - (b[index] || 0);
    if (difference !== 0) return Math.sign(difference);
  }
  return 0;
}

function json(response, status, payload) {
  const body = `${JSON.stringify(payload)}\n`;
  response.writeHead(status, {
    "Cache-Control": "no-store",
    "Content-Length": Buffer.byteLength(body),
    "Content-Type": "application/json; charset=utf-8",
  });
  response.end(body);
}

function noContent(response) {
  response.writeHead(204, { "Cache-Control": "no-store" });
  response.end();
}

export function createUpdaterTestServer({
  artifact,
  signature,
  version,
  target,
  arch,
  mode = "available",
  notes = "WP-604A local updater test release",
  host = "127.0.0.1",
  port = 0,
  publishedAt = "2026-07-30T00:00:00Z",
  onRequest = () => {},
}) {
  if (!MODES.has(mode)) throw new Error(`Unsupported mode: ${mode}`);
  if (!TARGETS.has(target)) throw new Error(`Unsupported target: ${target}`);
  if (!ARCHITECTURES.has(arch)) throw new Error(`Unsupported architecture: ${arch}`);
  if (!/^\d+\.\d+\.\d+([+-][0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error("version must be a semantic version without a v prefix");
  }
  if (!["127.0.0.1", "localhost", "::1"].includes(host)) {
    throw new Error("The updater test server may only bind to a loopback address");
  }

  const artifactPath = artifact ? resolve(artifact) : "";
  const signatureValue = signature ? readFileSync(resolve(signature), "utf8").trim() : "";
  const artifactName = artifactPath ? basename(artifactPath) : "missing-update-artifact";
  if (mode === "available" || mode === "invalid-signature") {
    if (!artifactPath || !statSync(artifactPath).isFile()) {
      throw new Error("artifact must point to an existing updater package");
    }
    if (!signatureValue) throw new Error("signature must point to a non-empty .sig file");
  }

  const server = createServer((request, response) => {
    const requestUrl = new URL(request.url || "/", `http://${host}`);
    onRequest(request.method || "UNKNOWN", requestUrl.pathname);
    if (request.method !== "GET") {
      json(response, 405, { error: "method_not_allowed" });
      return;
    }
    if (requestUrl.pathname === "/health") {
      json(response, 200, {
        service: "codex-assistant-updater-test",
        mode,
        version,
        target,
        arch,
      });
      return;
    }
    if (requestUrl.pathname === `/downloads/${encodeURIComponent(artifactName)}`) {
      if (!artifactPath) {
        json(response, 404, { error: "artifact_not_configured" });
        return;
      }
      const stat = statSync(artifactPath);
      response.writeHead(200, {
        "Cache-Control": "no-store",
        "Content-Disposition": `attachment; filename="${artifactName}"`,
        "Content-Length": stat.size,
        "Content-Type": "application/octet-stream",
      });
      createReadStream(artifactPath).pipe(response);
      return;
    }

    const match = requestUrl.pathname.match(
      /^\/updates\/(windows|darwin)\/(x86_64|aarch64)\/([^/]+)$/,
    );
    if (!match) {
      json(response, 404, { error: "not_found" });
      return;
    }
    const [, requestedTarget, requestedArch, encodedCurrentVersion] = match;
    const currentVersion = decodeURIComponent(encodedCurrentVersion);
    if (mode === "error") {
      json(response, 503, { error: "simulated_update_service_failure" });
      return;
    }
    if (
      mode === "none"
      || requestedTarget !== target
      || requestedArch !== arch
      || compareVersions(currentVersion, version) >= 0
    ) {
      noContent(response);
      return;
    }

    const address = server.address();
    const downloadPort = typeof address === "object" && address ? address.port : port;
    json(response, 200, {
      version,
      pub_date: publishedAt,
      notes,
      url: `http://${host}:${downloadPort}/downloads/${encodeURIComponent(artifactName)}`,
      signature: mode === "invalid-signature" ? "invalid-signature" : signatureValue,
    });
  });

  return {
    server,
    async listen() {
      await new Promise((resolveListen, reject) => {
        server.once("error", reject);
        server.listen(port, host, resolveListen);
      });
      const address = server.address();
      if (!address || typeof address === "string") {
        throw new Error("Unable to determine updater test server address");
      }
      return {
        host,
        port: address.port,
        endpoint: `http://${host}:${address.port}/updates/{{target}}/{{arch}}/{{current_version}}`,
      };
    },
    async close() {
      if (!server.listening) return;
      await new Promise((resolveClose, reject) => {
        server.close((error) => (error ? reject(error) : resolveClose()));
      });
    },
  };
}

function argument(name, fallback) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : fallback;
}

async function main() {
  const service = createUpdaterTestServer({
    artifact: argument("--artifact"),
    signature: argument("--signature"),
    version: argument("--version"),
    target: argument("--target"),
    arch: argument("--arch"),
    mode: argument("--mode", "available"),
    notes: argument("--notes", "WP-604A local updater test release"),
    host: argument("--host", "127.0.0.1"),
    port: Number.parseInt(argument("--port", "4317"), 10),
    onRequest(method, path) {
      process.stdout.write(`${method} ${path}\n`);
    },
  });
  const address = await service.listen();
  process.stdout.write(
    [
      "Codex Assistant updater test server is ready.",
      `Endpoint: ${address.endpoint}`,
      "Press Ctrl+C to stop.",
      "",
    ].join("\n"),
  );
  const stop = async () => {
    await service.close();
    process.exit(0);
  };
  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    process.stderr.write(`${error.stack || error.message}\n`);
    process.exit(1);
  });
}
