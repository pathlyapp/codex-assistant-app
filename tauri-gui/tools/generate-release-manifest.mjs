import { createHash } from "node:crypto";
import { createReadStream, promises as fs } from "node:fs";
import { basename, join, resolve } from "node:path";

function argument(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function expectedArtifacts(version) {
  return new Map([
    [
      `CodexAssistant-${version}-windows-x64-setup.exe`,
      { platform: "windows", arch: "x86_64", format: "nsis" },
    ],
    [
      `CodexAssistant-${version}-windows-arm64-setup.exe`,
      { platform: "windows", arch: "aarch64", format: "nsis" },
    ],
    [
      `CodexAssistant-${version}-macos-arm64.app.zip`,
      { platform: "macos", arch: "aarch64", format: "app.zip" },
    ],
  ]);
}

function sha256(path) {
  return new Promise((resolveHash, reject) => {
    const hash = createHash("sha256");
    const stream = createReadStream(path);
    stream.on("error", reject);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("end", () => resolveHash(hash.digest("hex")));
  });
}

const version = argument("--version");
const input = resolve(argument("--input") || "release-assets");
const channel = argument("--channel");

if (!version || !/^\d+\.\d+\.\d+([+-][0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error("--version must be a semantic version without a v prefix");
}
if (!["internal-test", "customer"].includes(channel)) {
  throw new Error("--channel must be internal-test or customer");
}
if (channel === "customer") {
  throw new Error(
    "Customer release is blocked until code-signing and signed-manifest verification are implemented",
  );
}

const entries = await fs.readdir(input, { withFileTypes: true });
const expected = expectedArtifacts(version);
const artifactNames = entries
  .filter((entry) => entry.isFile())
  .map((entry) => entry.name)
  .filter(
    (name) =>
      !["package-manifest.json", "SHA256SUMS.txt", "RELEASE-NOTES.md"].includes(name),
  )
  .sort();

if (artifactNames.length === 0) {
  throw new Error(`No release artifacts found in ${input}`);
}
const unexpected = artifactNames.filter((name) => !expected.has(name));
const missing = [...expected.keys()].filter((name) => !artifactNames.includes(name));
if (unexpected.length > 0 || missing.length > 0) {
  throw new Error(
    `Release artifact set mismatch; missing=[${missing.join(", ")}], unexpected=[${unexpected.join(", ")}]`,
  );
}

const artifacts = [];
for (const file of artifactNames) {
  const path = join(input, file);
  const stat = await fs.stat(path);
  artifacts.push({
    file: basename(file),
    ...expected.get(file),
    bytes: stat.size,
    sha256: await sha256(path),
    signing: {
      requiredForCustomer: true,
      status: "not_verified",
    },
  });
}

const manifest = {
  schemaVersion: 2,
  product: "codex-assistant",
  version,
  releasePolicy: {
    channel,
    customerReady: false,
    codeSigning: "not_verified",
    manifestSignature: "not_configured",
    blockingReason: "unsigned_internal_test_only",
  },
  generatedAt: new Date().toISOString(),
  artifacts,
};
const checksums = artifacts.map((artifact) => `${artifact.sha256}  ${artifact.file}`).join("\n");

await fs.writeFile(
  join(input, "package-manifest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
  "utf8",
);
await fs.writeFile(join(input, "SHA256SUMS.txt"), `${checksums}\n`, "utf8");

console.log(
  `Generated ${channel} release metadata for ${artifacts.length} artifacts.`,
);
