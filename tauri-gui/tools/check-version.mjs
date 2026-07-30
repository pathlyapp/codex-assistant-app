import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const toolsDir = dirname(fileURLToPath(import.meta.url));
const appDir = resolve(toolsDir, "..");
const manifestPath = resolve(appDir, "src-tauri", "Cargo.toml");
const packageJson = JSON.parse(readFileSync(resolve(appDir, "package.json"), "utf8"));
const packageLock = JSON.parse(
  readFileSync(resolve(appDir, "package-lock.json"), "utf8"),
);
const tauriConfig = JSON.parse(
  readFileSync(resolve(appDir, "src-tauri", "tauri.conf.json"), "utf8"),
);
const frontend = readFileSync(resolve(appDir, "frontend", "index.html"), "utf8");
const rustEntry = readFileSync(resolve(appDir, "src-tauri", "src", "lib.rs"), "utf8");
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
const frontendLabel = frontend.match(/Codex 助手 (\d+\.\d+\.\d+)/)?.[1];
const frontendAssetVersions = [
  ...frontend.matchAll(/(?:styles\.css|main\.js)\?v=(\d+\.\d+\.\d+)/g),
].map((match) => match[1]);
const versions = {
  expected: expectedVersion,
  packageJson: packageJson.version,
  packageLock: packageLock.version,
  packageLockRoot: packageLock.packages?.[""]?.version,
  cargo: cargoVersion,
  tauri: tauriConfig.version,
  frontendLabel,
  frontendAssets:
    frontendAssetVersions.length > 0 && new Set(frontendAssetVersions).size === 1
      ? frontendAssetVersions[0]
      : undefined,
};
const mismatches = Object.entries(versions).filter(([, version]) => version !== expectedVersion);

if (!rustEntry.includes('const VERSION: &str = env!("CARGO_PKG_VERSION");')) {
  console.error('Rust runtime version must use env!("CARGO_PKG_VERSION").');
  process.exit(1);
}

if (tauriConfig.plugins?.updater) {
  console.error(
    "Base Tauri config must not contain a partial updater config; signed updater builds inject it.",
  );
  process.exit(1);
}

if (mismatches.length > 0) {
  console.error("Version mismatch:");
  for (const [source, version] of Object.entries(versions)) {
    console.error(`  ${source}: ${version}`);
  }
  process.exit(1);
}

console.log(`Version ${expectedVersion} is consistent.`);
