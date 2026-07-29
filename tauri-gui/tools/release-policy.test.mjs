import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const toolsDir = dirname(fileURLToPath(import.meta.url));
const manifestScript = join(toolsDir, "generate-release-manifest.mjs");
const notesScript = join(toolsDir, "generate-release-notes.mjs");

async function releaseDirectory(t) {
  const root = await mkdtemp(join(tmpdir(), "codex-assistant-release-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(
    join(root, "CodexAssistant-1.2.3-windows-x64-setup.exe"),
    "windows-x64",
  );
  await writeFile(
    join(root, "CodexAssistant-1.2.3-windows-arm64-setup.exe"),
    "windows-arm64",
  );
  await writeFile(
    join(root, "CodexAssistant-1.2.3-macos-arm64.app.zip"),
    "macos-arm64",
  );
  return root;
}

function runManifest(root, channel) {
  return execFileSync(
    process.execPath,
    [
      manifestScript,
      "--version",
      "1.2.3",
      "--input",
      root,
      "--channel",
      channel,
    ],
    { encoding: "utf8" },
  );
}

test("internal test manifest is explicitly not customer ready", async (t) => {
  const root = await releaseDirectory(t);
  runManifest(root, "internal-test");

  const manifest = JSON.parse(
    await readFile(join(root, "package-manifest.json"), "utf8"),
  );
  assert.equal(manifest.schemaVersion, 2);
  assert.equal(manifest.releasePolicy.channel, "internal-test");
  assert.equal(manifest.releasePolicy.customerReady, false);
  assert.equal(manifest.releasePolicy.codeSigning, "not_verified");
  assert.equal(manifest.releasePolicy.manifestSignature, "not_configured");
  assert.equal(manifest.artifacts.length, 3);
  for (const artifact of manifest.artifacts) {
    assert.equal(artifact.signing.requiredForCustomer, true);
    assert.equal(artifact.signing.status, "not_verified");
    assert.match(artifact.sha256, /^[a-f0-9]{64}$/);
  }

  const expected = createHash("sha256").update("windows-x64").digest("hex");
  const x64 = manifest.artifacts.find((artifact) => artifact.arch === "x86_64");
  assert.equal(x64.sha256, expected);
});

test("customer channel is blocked without signing verification", async (t) => {
  const root = await releaseDirectory(t);
  const result = spawnSync(
    process.execPath,
    [
      manifestScript,
      "--version",
      "1.2.3",
      "--input",
      root,
      "--channel",
      "customer",
    ],
    { encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Customer release is blocked/);
});

test("release channel is mandatory", async (t) => {
  const root = await releaseDirectory(t);
  const result = spawnSync(
    process.execPath,
    [manifestScript, "--version", "1.2.3", "--input", root],
    { encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /--channel must be/);
});

test("incomplete artifact set is rejected", async (t) => {
  const root = await releaseDirectory(t);
  await rm(join(root, "CodexAssistant-1.2.3-macos-arm64.app.zip"));
  const result = spawnSync(
    process.execPath,
    [
      manifestScript,
      "--version",
      "1.2.3",
      "--input",
      root,
      "--channel",
      "internal-test",
    ],
    { encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Release artifact set mismatch/);
  assert.match(result.stderr, /macos-arm64\.app\.zip/);
});

test("unexpected release attachment is rejected", async (t) => {
  const root = await releaseDirectory(t);
  await writeFile(join(root, "CodexAssistant-1.2.2-windows-x64-setup.exe"), "old");
  const result = spawnSync(
    process.execPath,
    [
      manifestScript,
      "--version",
      "1.2.3",
      "--input",
      root,
      "--channel",
      "internal-test",
    ],
    { encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Release artifact set mismatch/);
  assert.match(result.stderr, /1\.2\.2-windows-x64/);
});

test("generated notes disclose limitations and recovery", async (t) => {
  const root = await releaseDirectory(t);
  runManifest(root, "internal-test");
  execFileSync(
    process.execPath,
    [
      notesScript,
      "--manifest",
      join(root, "package-manifest.json"),
      "--output",
      join(root, "RELEASE-NOTES.md"),
    ],
    { encoding: "utf8" },
  );

  const notes = await readFile(join(root, "RELEASE-NOTES.md"), "utf8");
  assert.match(notes, /Internal Test Build/);
  assert.match(notes, /Known Limitations/);
  assert.match(notes, /Upgrade/);
  assert.match(notes, /Recovery And Uninstall/);
  assert.match(notes, /SHA256 verifies file integrity, not publisher identity/);
});
