import { createHash } from "node:crypto";
import { createReadStream, promises as fs } from "node:fs";
import { basename, join, resolve } from "node:path";

function argument(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function artifactMetadata(file) {
  const name = file.toLowerCase();
  if (name.includes("windows-x64")) {
    return { platform: "windows", arch: "x86_64", format: "nsis" };
  }
  if (name.includes("windows-arm64")) {
    return { platform: "windows", arch: "aarch64", format: "nsis" };
  }
  if (name.includes("macos-arm64")) {
    return { platform: "macos", arch: "aarch64", format: "app.zip" };
  }
  throw new Error(`Cannot infer artifact platform from ${file}`);
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

if (!version || !/^\d+\.\d+\.\d+([+-][0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error("--version must be a semantic version without a v prefix");
}

const entries = await fs.readdir(input, { withFileTypes: true });
const artifactNames = entries
  .filter((entry) => entry.isFile())
  .map((entry) => entry.name)
  .filter((name) => !["package-manifest.json", "SHA256SUMS.txt"].includes(name))
  .sort();

if (artifactNames.length === 0) {
  throw new Error(`No release artifacts found in ${input}`);
}

const artifacts = [];
for (const file of artifactNames) {
  const path = join(input, file);
  const stat = await fs.stat(path);
  artifacts.push({
    file: basename(file),
    ...artifactMetadata(file),
    bytes: stat.size,
    sha256: await sha256(path),
  });
}

const manifest = {
  schemaVersion: 1,
  product: "codex-assistant",
  version,
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

console.log(`Generated release metadata for ${artifacts.length} artifacts.`);
