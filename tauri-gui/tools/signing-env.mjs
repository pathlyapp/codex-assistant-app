import { spawnSync } from "node:child_process";
import { promises as fs } from "node:fs";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

export const SIGNING_ENV_PATH = join(ROOT, ".env.signing");

export function parseEnvFile(text) {
  const entries = {};
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }
    const separator = line.indexOf("=");
    if (separator <= 0) {
      continue;
    }
    const key = line.slice(0, separator).trim();
    let value = line.slice(separator + 1).trim();
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }
    if (key) {
      entries[key] = value;
    }
  }
  return entries;
}

export async function loadSigningEnv(env, filePath = SIGNING_ENV_PATH) {
  let text;
  try {
    text = await fs.readFile(filePath, "utf8");
  } catch {
    return false;
  }
  for (const [key, value] of Object.entries(parseEnvFile(text))) {
    if (!env[key]?.trim()) {
      env[key] = value;
    }
  }
  return true;
}

export function detectDeveloperIdIdentity() {
  const result = spawnSync(
    "security",
    ["find-identity", "-v", "-p", "codesigning"],
    { encoding: "utf8" },
  );
  if (result.error || result.status !== 0) {
    return null;
  }
  for (const line of result.stdout.split("\n")) {
    const match = line.match(
      /^\s*\d+\)\s+([0-9A-Fa-f]{40})\s+"(Developer ID Application: [^"]+)"/,
    );
    if (match) {
      return { hash: match[1].toUpperCase(), name: match[2] };
    }
  }
  return null;
}

function defaultApiKeySearchPaths(keyId) {
  return [
    join(ROOT, "private_keys"),
    join(homedir(), "private_keys"),
    join(homedir(), ".private_keys"),
    join(homedir(), ".appstoreconnect", "private_keys"),
  ].map((directory) => join(directory, `AuthKey_${keyId}.p8`));
}

export function resolveApiKeyFile(keyId, configuredPath) {
  if (configuredPath) {
    const path = isAbsolute(configuredPath)
      ? configuredPath
      : resolve(ROOT, configuredPath);
    return existsSync(path) ? path : null;
  }
  for (const candidate of defaultApiKeySearchPaths(keyId)) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

export function notarizationCredentials(env) {
  const apiKey = env.APPLE_API_KEY?.trim();
  const apiIssuer = env.APPLE_API_ISSUER?.trim();
  const apiKeyPath = env.APPLE_API_KEY_PATH?.trim();
  if (apiKey || apiIssuer || apiKeyPath) {
    const missing = [];
    if (!apiKey) {
      missing.push("APPLE_API_KEY");
    }
    if (!apiIssuer) {
      missing.push("APPLE_API_ISSUER");
    }
    if (missing.length > 0) {
      return {
        mode: null,
        error: `App Store Connect API 公证配置不完整，缺少: ${missing.join(", ")}`,
      };
    }
    const keyFile = resolveApiKeyFile(apiKey, apiKeyPath);
    if (!keyFile) {
      return {
        mode: null,
        error: apiKeyPath
          ? `APPLE_API_KEY_PATH 指向的 .p8 文件不存在: ${apiKeyPath}`
          : `未找到 AuthKey_${apiKey}.p8，请设置 APPLE_API_KEY_PATH 或将密钥放入 ~/private_keys 等默认目录`,
      };
    }
    return { mode: "api-key", keyFile };
  }

  const appleId = env.APPLE_ID?.trim();
  const applePassword = env.APPLE_PASSWORD?.trim();
  const teamId = env.APPLE_TEAM_ID?.trim();
  if (appleId || applePassword || teamId) {
    const missing = [];
    if (!appleId) {
      missing.push("APPLE_ID");
    }
    if (!applePassword) {
      missing.push("APPLE_PASSWORD");
    }
    if (!teamId) {
      missing.push("APPLE_TEAM_ID");
    }
    if (missing.length > 0) {
      return {
        mode: null,
        error: `Apple ID 公证配置不完整，缺少: ${missing.join(", ")}`,
      };
    }
    return { mode: "apple-id" };
  }

  return { mode: null };
}
