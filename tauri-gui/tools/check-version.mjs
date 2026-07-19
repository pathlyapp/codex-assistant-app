import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const toolsDir = dirname(fileURLToPath(import.meta.url));
const appDir = resolve(toolsDir, "..");
const manifestPath = resolve(appDir, "src-tauri", "Cargo.toml");
const packageJson = JSON.parse(readFileSync(resolve(appDir, "package.json"), "utf8"));
const tauriConfig = JSON.parse(
  readFileSync(resolve(appDir, "src-tauri", "tauri.conf.json"), "utf8"),
);
const cargo = spawnSync(
  "cargo",
  ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", manifestPath],
  { encoding: "utf8" },
);

if (cargo.status !== 0) {
  process.stderr.write(cargo.stderr || "Unable to read Cargo metadata.\n");
  process.exit(cargo.status || 1);
}

const cargoMetadata = JSON.parse(cargo.stdout);
const cargoVersion = cargoMetadata.packages[0]?.version;
const expectedVersion = (process.argv[2] || packageJson.version).replace(/^v/, "");
const versions = {
  expected: expectedVersion,
  packageJson: packageJson.version,
  cargo: cargoVersion,
  tauri: tauriConfig.version,
};
const mismatches = Object.entries(versions).filter(([, version]) => version !== expectedVersion);

if (mismatches.length > 0) {
  console.error("Version mismatch:");
  for (const [source, version] of Object.entries(versions)) {
    console.error(`  ${source}: ${version}`);
  }
  process.exit(1);
}

console.log(`Version ${expectedVersion} is consistent.`);
