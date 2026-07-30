import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

function argument(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

const version = argument("--version");
if (!version || !/^\d+\.\d+\.\d+([+-][0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error("--version must be a semantic version without a v prefix");
}

const toolsDirectory = dirname(fileURLToPath(import.meta.url));
const root = resolve(argument("--root") || resolve(toolsDirectory, ".."));

function updateJson(relativePath, mutate) {
  const path = resolve(root, relativePath);
  const value = JSON.parse(readFileSync(path, "utf8"));
  mutate(value);
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

updateJson("package.json", (value) => {
  value.version = version;
});
updateJson("package-lock.json", (value) => {
  value.version = version;
  value.packages[""].version = version;
});
updateJson("src-tauri/tauri.conf.json", (value) => {
  value.version = version;
});

const cargoPath = resolve(root, "src-tauri", "Cargo.toml");
const cargo = readFileSync(cargoPath, "utf8");
const updatedCargo = cargo.replace(
  /(^\[package\][\s\S]*?^version = ")[^"]+(")/m,
  `$1${version}$2`,
);
if (updatedCargo === cargo) {
  throw new Error("Cargo package version was not found");
}
writeFileSync(cargoPath, updatedCargo, "utf8");

const frontendPath = resolve(root, "frontend", "index.html");
const frontend = readFileSync(frontendPath, "utf8");
const updatedFrontend = frontend
  .replace(/(styles\.css\?v=)\d+\.\d+\.\d+/g, `$1${version}`)
  .replace(/(main\.js\?v=)\d+\.\d+\.\d+/g, `$1${version}`)
  .replace(/(Codex 助手 )\d+\.\d+\.\d+/g, `$1${version}`);
if (updatedFrontend === frontend) {
  throw new Error("Frontend version fields were not found");
}
writeFileSync(frontendPath, updatedFrontend, "utf8");

console.log(`Set Codex Assistant version to ${version} in ${root}`);
