import { spawnSync } from "node:child_process";
import { promises as fs } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  detectDeveloperIdIdentity,
  loadSigningEnv,
  notarizationCredentials,
  SIGNING_ENV_PATH,
} from "./signing-env.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const USAGE = `用法: node tools/build-mac-signed.mjs [选项] [-- <透传给 tauri build 的参数>]

本地 macOS 分发构建：Developer ID 签名 + Apple 公证 + 校验 + 打包。

选项:
  --dmg             打包 DMG 安装镜像（默认只产出 .app 并打成 zip）
  --skip-notarize   只做代码签名，跳过公证（产物仍无法通过 Gatekeeper，仅供测试）
  --target <triple> 指定 Rust 目标（aarch64-apple-darwin / x86_64-apple-darwin）
  -h, --help        显示帮助

签名配置（按优先级）:
  1. 环境变量 APPLE_SIGNING_IDENTITY / APPLE_API_* / APPLE_ID 等
  2. ${SIGNING_ENV_PATH}
     （模板见 tauri-gui/signing.env.example，该文件已被 git 忽略）
  3. 签名身份缺省时自动检测钥匙串中第一个 "Developer ID Application" 证书
`;

function parseArguments(argv) {
  const options = {
    dmg: false,
    skipNotarize: false,
    target: null,
    passthrough: [],
  };
  const rest = [...argv];
  while (rest.length > 0) {
    const arg = rest.shift();
    if (arg === "--") {
      options.passthrough.push(...rest);
      break;
    }
    if (arg === "-h" || arg === "--help") {
      return { help: true };
    }
    if (arg === "--dmg") {
      options.dmg = true;
      continue;
    }
    if (arg === "--skip-notarize") {
      options.skipNotarize = true;
      continue;
    }
    if (arg === "--target") {
      options.target = rest.shift();
      if (!options.target) {
        throw new Error("--target 需要一个 Rust target 参数");
      }
      continue;
    }
    throw new Error(`未知参数: ${arg}\n${USAGE}`);
  }
  if (
    options.target &&
    !["aarch64-apple-darwin", "x86_64-apple-darwin"].includes(options.target)
  ) {
    throw new Error(`--target 只支持 macOS 桌面目标，收到: ${options.target}`);
  }
  return options;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: ROOT,
    stdio: options.inherit ? "inherit" : "pipe",
    encoding: "utf8",
    env: options.env,
  });
  if (result.error) {
    throw result.error;
  }
  return result;
}

function runChecked(command, args, options = {}) {
  const result = run(command, args, options);
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "").trim();
    throw new Error(
      `${command} ${args.join(" ")} 失败（退出码 ${result.status}）${detail ? `\n${detail}` : ""}`,
    );
  }
  return result;
}

function bundleDirectory(target, kind) {
  return join(
    ROOT,
    "src-tauri",
    "target",
    ...(target ? [target] : []),
    "release",
    "bundle",
    kind,
  );
}

async function locateBundle(target, kind, extension) {
  const directory = bundleDirectory(target, kind);
  const entries = await fs.readdir(directory).catch(() => []);
  const bundle = entries.find((entry) => entry.endsWith(extension));
  if (!bundle) {
    throw new Error(`未在 ${directory} 找到 ${extension} 产物`);
  }
  return join(directory, bundle);
}

function notarytoolCredentials(env, notarization) {
  if (notarization.mode === "api-key") {
    return [
      "--key",
      notarization.keyFile,
      "--key-id",
      env.APPLE_API_KEY.trim(),
      "--issuer",
      env.APPLE_API_ISSUER.trim(),
    ];
  }
  return [
    "--apple-id",
    env.APPLE_ID.trim(),
    "--password",
    env.APPLE_PASSWORD.trim(),
    "--team-id",
    env.APPLE_TEAM_ID.trim(),
  ];
}

function stapleTicket(path) {
  runChecked("xcrun", ["stapler", "staple", "-v", path]);
}

function isStapled(path) {
  return run("xcrun", ["stapler", "validate", path]).status === 0;
}

function verifySignature(appPath, { expectNotarized }) {
  runChecked("codesign", ["--verify", "--deep", "--strict", "--verbose=2", appPath]);
  console.log("✔ codesign 校验通过（--deep --strict）");

  const info = run("codesign", ["-dv", "--verbose=4", appPath]);
  const authority = (info.stderr || "").match(/Authority=(.+)/);
  console.log(`✔ 签名身份: ${authority ? authority[1] : "未知"}`);
  if (!authority || !authority[1].startsWith("Developer ID Application")) {
    throw new Error("产物未使用 Developer ID Application 证书签名");
  }

  const assess = run("spctl", ["--assess", "--type", "execute", "--verbose=4", appPath]);
  if (assess.status === 0) {
    console.log("✔ Gatekeeper 评估通过（spctl --assess）");
  } else if (expectNotarized) {
    throw new Error(
      `Gatekeeper 评估失败，公证可能未生效:\n${(assess.stderr || "").trim()}`,
    );
  } else {
    console.warn("⚠ Gatekeeper 评估未通过（预期行为：已跳过公证，仅供本机测试）");
  }

  if (expectNotarized) {
    if (!isStapled(appPath)) {
      throw new Error("公证票据未附加到 .app（stapler validate 失败）");
    }
    console.log("✔ 公证票据已附加（stapler validate）");
  }
}

async function verifyDmg(dmgPath, { expectNotarized, buildEnvironment, notarization }) {
  runChecked("codesign", ["--verify", "--verbose=2", dmgPath]);
  console.log("✔ DMG 签名校验通过");

  if (!expectNotarized) {
    console.warn("⚠ DMG 未公证（--skip-notarize 模式）");
    return;
  }

  if (!isStapled(dmgPath)) {
    console.log("ℹ DMG 未附带公证票据，单独提交公证...");
    runChecked("xcrun", [
      "notarytool",
      "submit",
      dmgPath,
      ...notarytoolCredentials(buildEnvironment, notarization),
      "--wait",
    ], { inherit: true, env: buildEnvironment });
    stapleTicket(dmgPath);
  }
  if (!isStapled(dmgPath)) {
    throw new Error("DMG 公证票据装订失败（stapler validate 失败）");
  }
  console.log("✔ DMG 公证票据已附加（stapler validate）");
}

async function withMountedDmg(dmgPath, fn) {
  const mountPoint = await fs.mkdtemp(join(tmpdir(), "codex-assistant-dmg-"));
  try {
    runChecked("hdiutil", [
      "attach",
      "-nobrowse",
      "-readonly",
      "-mountpoint",
      mountPoint,
      dmgPath,
    ]);
    return await fn(mountPoint);
  } finally {
    const detach = run("hdiutil", ["detach", mountPoint, "-quiet"]);
    if (detach.status !== 0) {
      run("hdiutil", ["detach", mountPoint, "-force", "-quiet"]);
    }
    await fs.rm(mountPoint, { recursive: true, force: true });
  }
}

async function stageArtifact(appPath, dmgPath, target) {
  const packageJson = JSON.parse(
    await fs.readFile(join(ROOT, "package.json"), "utf8"),
  );
  const arch = target
    ? { "aarch64-apple-darwin": "arm64", "x86_64-apple-darwin": "x64" }[target]
    : { arm64: "arm64", x64: "x64" }[process.arch] || process.arch;
  const artifactDirectory = join(ROOT, "artifact");
  await fs.mkdir(artifactDirectory, { recursive: true });
  const baseName = `CodexAssistant-${packageJson.version}-macos-${arch}`;

  if (dmgPath) {
    const destination = join(artifactDirectory, `${baseName}.dmg`);
    await fs.copyFile(dmgPath, destination);
    return destination;
  }

  const zipPath = join(artifactDirectory, `${baseName}.app.zip`);
  await fs.rm(zipPath, { force: true });
  runChecked("ditto", [
    "-c",
    "-k",
    "--sequesterRsrc",
    "--keepParent",
    appPath,
    zipPath,
  ]);
  return zipPath;
}

async function main() {
  const options = parseArguments(process.argv.slice(2));
  if (options.help) {
    console.log(USAGE);
    return;
  }

  const buildEnvironment = { ...process.env };
  const envFileLoaded = await loadSigningEnv(buildEnvironment);
  console.log(
    envFileLoaded
      ? `✔ 已加载签名配置: ${SIGNING_ENV_PATH}`
      : `ℹ 未找到 ${SIGNING_ENV_PATH}，仅使用当前环境变量`,
  );

  if (!buildEnvironment.APPLE_SIGNING_IDENTITY?.trim()) {
    const detected = detectDeveloperIdIdentity();
    if (!detected) {
      throw new Error(
        "钥匙串中没有可用的 Developer ID Application 证书。\n" +
          "请先安装证书（参考 tauri-gui/signing.env.example），" +
          "或设置 APPLE_SIGNING_IDENTITY 环境变量。",
      );
    }
    buildEnvironment.APPLE_SIGNING_IDENTITY = detected.hash;
    console.log(`✔ 自动检测到签名身份: ${detected.name}`);
  }
  console.log(`✔ 使用签名身份: ${buildEnvironment.APPLE_SIGNING_IDENTITY}`);

  const notarization = notarizationCredentials(buildEnvironment);
  let expectNotarized = false;
  if (options.skipNotarize) {
    console.warn("⚠ 已指定 --skip-notarize，本次构建不会公证，无法直接分发");
    for (const key of [
      "APPLE_API_ISSUER",
      "APPLE_API_KEY",
      "APPLE_API_KEY_PATH",
      "APPLE_ID",
      "APPLE_PASSWORD",
      "APPLE_TEAM_ID",
    ]) {
      delete buildEnvironment[key];
    }
  } else if (notarization.error) {
    throw new Error(
      `${notarization.error}\n请完善 ${SIGNING_ENV_PATH} 后重试；` +
        "如确需跳过公证，使用 --skip-notarize。",
    );
  } else if (!notarization.mode) {
    throw new Error(
      "未配置公证凭据，签名后的 App 仍会被 Gatekeeper 拦截，无法分发。\n" +
        `请参照 tauri-gui/signing.env.example 创建 ${SIGNING_ENV_PATH}，` +
        "配置 App Store Connect API Key（推荐，与 deeppath 一致）或 Apple ID 专用密码；\n" +
        "如确需跳过公证，使用 --skip-notarize。",
    );
  } else {
    expectNotarized = true;
    if (notarization.mode === "api-key") {
      buildEnvironment.APPLE_API_KEY_PATH = notarization.keyFile;
      console.log(`✔ 公证方式: App Store Connect API（密钥 ${notarization.keyFile}）`);
    } else {
      console.log("✔ 公证方式: Apple ID 专用密码");
    }
  }

  const require = createRequire(import.meta.url);
  const tauriCli = require.resolve("@tauri-apps/cli/tauri.js");
  const tauriArgs = ["build", "--bundles", options.dmg ? "dmg" : "app"];
  if (options.target) {
    tauriArgs.push("--target", options.target);
  }
  tauriArgs.push(...options.passthrough);
  console.log(`\n开始构建: tauri ${tauriArgs.join(" ")}\n`);
  const build = run(process.execPath, [tauriCli, ...tauriArgs], {
    env: buildEnvironment,
    inherit: true,
  });
  if (build.status !== 0) {
    throw new Error(`tauri build 失败（退出码 ${build.status}）`);
  }

  let appPath;
  let dmgPath = null;
  if (options.dmg) {
    // DMG 模式下 .app 在打包后被 Tauri 清理，改为挂载 DMG 校验内部 App
    dmgPath = await locateBundle(options.target, "dmg", ".dmg");
    console.log(`\nDMG 产物: ${dmgPath}`);
    await verifyDmg(dmgPath, { expectNotarized, buildEnvironment, notarization });
    appPath = await withMountedDmg(dmgPath, async (mountPoint) => {
      const entries = await fs.readdir(mountPoint);
      const app = entries.find((entry) => entry.endsWith(".app"));
      if (!app) {
        throw new Error("DMG 中未找到 .app");
      }
      const mountedApp = join(mountPoint, app);
      console.log(`\n校验 DMG 内的 App: ${mountedApp}`);
      verifySignature(mountedApp, { expectNotarized });
      return mountedApp;
    });
  } else {
    appPath = await locateBundle(options.target, "macos", ".app");
    console.log(`\n构建产物: ${appPath}`);
    verifySignature(appPath, { expectNotarized });
  }

  const artifactPath = await stageArtifact(appPath, dmgPath, options.target);
  console.log(`\n✔ 分发包已生成: ${artifactPath}`);
  if (expectNotarized) {
    console.log("✔ 签名 + 公证完成，可直接分发。");
  } else {
    console.warn("⚠ 未公证，仅供本机测试；分发前请配置公证凭据后重新构建。");
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(`\n✘ ${error.message}`);
    process.exitCode = 1;
  });
}
