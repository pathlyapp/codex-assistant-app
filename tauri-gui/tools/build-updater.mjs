import { spawnSync } from "node:child_process";
import { promises as fs } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const GENERATED_CONFIG = join(
  ROOT,
  ".local-update",
  "tauri.updater.generated.conf.json",
);
const CHANNELS = new Set(["internal-test", "beta", "stable"]);

function argument(argv, name) {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : undefined;
}

export function updaterBuildSettings(argv, env) {
  const platform = argument(argv, "--platform");
  const target = argument(argv, "--target");
  const mock = argv.includes("--mock");
  if (!["mac", "windows"].includes(platform)) {
    throw new Error("--platform must be mac or windows");
  }
  if (
    target &&
    ![
      "aarch64-pc-windows-msvc",
      "x86_64-pc-windows-msvc",
      "aarch64-apple-darwin",
      "x86_64-apple-darwin",
    ].includes(target)
  ) {
    throw new Error("--target is not a supported desktop Rust target");
  }
  if (
    target &&
    ((platform === "windows" && !target.endsWith("-windows-msvc")) ||
      (platform === "mac" && !target.endsWith("-apple-darwin")))
  ) {
    throw new Error("--target does not match --platform");
  }

  const endpoint = env.CODEX_ASSISTANT_UPDATE_ENDPOINT?.trim();
  const pubkey = env.CODEX_ASSISTANT_UPDATE_PUBKEY?.trim();
  const channel = env.CODEX_ASSISTANT_UPDATE_CHANNEL?.trim() || "internal-test";
  if (!endpoint || !pubkey) {
    throw new Error(
      "CODEX_ASSISTANT_UPDATE_ENDPOINT and CODEX_ASSISTANT_UPDATE_PUBKEY are required",
    );
  }
  if (
    !env.TAURI_SIGNING_PRIVATE_KEY?.trim() &&
    !env.TAURI_SIGNING_PRIVATE_KEY_PATH?.trim()
  ) {
    throw new Error(
      "TAURI_SIGNING_PRIVATE_KEY or TAURI_SIGNING_PRIVATE_KEY_PATH is required",
    );
  }
  if (!CHANNELS.has(channel)) {
    throw new Error("update channel must be internal-test, beta, or stable");
  }

  let parsed;
  try {
    parsed = new URL(endpoint);
  } catch {
    throw new Error("CODEX_ASSISTANT_UPDATE_ENDPOINT must be a valid URL");
  }
  if (!parsed.hostname || parsed.username || parsed.password) {
    throw new Error(
      "CODEX_ASSISTANT_UPDATE_ENDPOINT must not contain embedded credentials",
    );
  }
  const loopback = ["127.0.0.1", "localhost", "[::1]"].includes(parsed.hostname);
  if (mock) {
    if (parsed.protocol !== "http:" || !loopback) {
      throw new Error("mock updater endpoint must use HTTP on a loopback host");
    }
  } else if (parsed.protocol !== "https:") {
    throw new Error("production updater endpoint must use HTTPS");
  }

  return {
    platform,
    target,
    mock,
    config: {
      bundle: {
        createUpdaterArtifacts: true,
      },
      plugins: {
        updater: {
          pubkey,
          endpoints: [endpoint],
          windows: {
            installMode: "passive",
          },
          ...(mock ? { dangerousInsecureTransportProtocol: true } : {}),
        },
      },
    },
  };
}

export function tauriBuildArguments(settings, configPath) {
  const args = [
    "build",
    "--bundles",
    settings.platform === "mac" ? "app" : "nsis",
    "--config",
    configPath,
  ];
  if (settings.mock) {
    args.push("--features", "updater-mock");
  }
  if (settings.target) {
    args.push("--target", settings.target);
  }
  return args;
}

async function main() {
  const settings = updaterBuildSettings(process.argv.slice(2), process.env);
  const buildEnvironment = { ...process.env };
  if (
    !buildEnvironment.TAURI_SIGNING_PRIVATE_KEY?.trim() &&
    buildEnvironment.TAURI_SIGNING_PRIVATE_KEY_PATH?.trim()
  ) {
    const privateKeyPath = resolve(
      ROOT,
      buildEnvironment.TAURI_SIGNING_PRIVATE_KEY_PATH,
    );
    buildEnvironment.TAURI_SIGNING_PRIVATE_KEY = await fs.readFile(
      privateKeyPath,
      "utf8",
    );
  }
  buildEnvironment.TAURI_SIGNING_PRIVATE_KEY_PASSWORD ??= "";
  await fs.mkdir(dirname(GENERATED_CONFIG), { recursive: true });
  await fs.writeFile(
    GENERATED_CONFIG,
    `${JSON.stringify(settings.config, null, 2)}\n`,
    { encoding: "utf8", mode: 0o600 },
  );

  const require = createRequire(import.meta.url);
  const tauriCli = require.resolve("@tauri-apps/cli/tauri.js");
  try {
    const result = spawnSync(
      process.execPath,
      [tauriCli, ...tauriBuildArguments(settings, GENERATED_CONFIG)],
      {
        cwd: ROOT,
        env: buildEnvironment,
        stdio: "inherit",
      },
    );
    if (result.error) {
      throw result.error;
    }
    process.exitCode = result.status ?? 1;
  } finally {
    await fs.rm(GENERATED_CONFIG, { force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
