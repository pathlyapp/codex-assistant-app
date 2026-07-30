import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, test } from "node:test";
import { createUpdaterTestServer } from "./updater-test-server.mjs";

const services = [];
const roots = [];

afterEach(async () => {
  while (services.length) await services.pop().close();
  while (roots.length) rmSync(roots.pop(), { force: true, recursive: true });
});

function fixture(mode = "available") {
  const root = mkdtempSync(join(tmpdir(), "codex-updater-test-"));
  roots.push(root);
  const artifact = join(root, "CodexAssistant-0.9.1-windows-arm64-setup.exe");
  const signature = `${artifact}.sig`;
  writeFileSync(artifact, "signed-updater-fixture");
  writeFileSync(signature, "trusted-signature-fixture\n");
  const service = createUpdaterTestServer({
    artifact,
    signature,
    version: "0.9.1",
    target: "windows",
    arch: "aarch64",
    mode,
  });
  services.push(service);
  return { service, artifact };
}

test("returns a dynamic update manifest and serves the exact artifact", async () => {
  const { service } = fixture();
  const address = await service.listen();
  const response = await fetch(
    `http://${address.host}:${address.port}/updates/windows/aarch64/0.9.0`,
  );
  assert.equal(response.status, 200);
  const manifest = await response.json();
  assert.equal(manifest.version, "0.9.1");
  assert.equal(manifest.signature, "trusted-signature-fixture");

  const download = await fetch(manifest.url);
  assert.equal(download.status, 200);
  assert.equal(await download.text(), "signed-updater-fixture");
});

test("returns 204 when the client is current or has a different target", async () => {
  const { service } = fixture();
  const address = await service.listen();
  const current = await fetch(
    `http://${address.host}:${address.port}/updates/windows/aarch64/0.9.1`,
  );
  assert.equal(current.status, 204);

  const otherArchitecture = await fetch(
    `http://${address.host}:${address.port}/updates/windows/x86_64/0.9.0`,
  );
  assert.equal(otherArchitecture.status, 204);
});

test("can simulate an invalid detached signature", async () => {
  const { service } = fixture("invalid-signature");
  const address = await service.listen();
  const response = await fetch(
    `http://${address.host}:${address.port}/updates/windows/aarch64/0.9.0`,
  );
  assert.equal(response.status, 200);
  assert.equal((await response.json()).signature, "invalid-signature");
});

test("can simulate update service failure without exposing an artifact", async () => {
  const { service } = fixture("error");
  const address = await service.listen();
  const response = await fetch(
    `http://${address.host}:${address.port}/updates/windows/aarch64/0.9.0`,
  );
  assert.equal(response.status, 503);
  assert.equal((await response.json()).error, "simulated_update_service_failure");
});

test("refuses to bind the mock service to a LAN address", () => {
  assert.throws(
    () =>
      createUpdaterTestServer({
        artifact: "unused",
        signature: "unused",
        version: "0.9.1",
        target: "windows",
        arch: "aarch64",
        mode: "none",
        host: "0.0.0.0",
      }),
    /loopback/,
  );
});
