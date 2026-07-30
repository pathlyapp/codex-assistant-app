import assert from "node:assert/strict";
import test from "node:test";

import {
  tauriBuildArguments,
  updaterBuildSettings,
} from "./build-updater.mjs";

const signing = {
  CODEX_ASSISTANT_UPDATE_PUBKEY: "public-key",
  CODEX_ASSISTANT_UPDATE_CHANNEL: "internal-test",
  TAURI_SIGNING_PRIVATE_KEY_PATH: ".local-update/test.key",
};

test("production updater builds require HTTPS", () => {
  assert.throws(
    () =>
      updaterBuildSettings(["--platform", "windows"], {
        ...signing,
        CODEX_ASSISTANT_UPDATE_ENDPOINT: "http://updates.example.test/latest.json",
      }),
    /must use HTTPS/,
  );
  assert.throws(
    () =>
      updaterBuildSettings(["--platform", "windows"], {
        ...signing,
        CODEX_ASSISTANT_UPDATE_ENDPOINT:
          "https://token@updates.example.test/latest.json",
      }),
    /embedded credentials/,
  );
});

test("mock updater builds only allow loopback HTTP", () => {
  const settings = updaterBuildSettings(["--platform", "mac", "--mock"], {
    ...signing,
    CODEX_ASSISTANT_UPDATE_ENDPOINT:
      "http://127.0.0.1:43123/updates/{{target}}/{{arch}}/{{current_version}}",
  });
  assert.equal(settings.config.plugins.updater.dangerousInsecureTransportProtocol, true);
  assert.deepEqual(tauriBuildArguments(settings, "generated.json"), [
    "build",
    "--bundles",
    "app",
    "--config",
    "generated.json",
    "--features",
    "updater-mock",
  ]);

  assert.throws(
    () =>
      updaterBuildSettings(["--platform", "mac", "--mock"], {
        ...signing,
        CODEX_ASSISTANT_UPDATE_ENDPOINT: "http://192.168.50.130:43123/latest.json",
      }),
    /loopback host/,
  );
});

test("updater build requires public and private signing material", () => {
  assert.throws(
    () =>
      updaterBuildSettings(["--platform", "windows"], {
        CODEX_ASSISTANT_UPDATE_ENDPOINT: "https://updates.example.test/latest.json",
      }),
    /UPDATE_PUBKEY/,
  );
  assert.throws(
    () =>
      updaterBuildSettings(["--platform", "windows"], {
        CODEX_ASSISTANT_UPDATE_ENDPOINT: "https://updates.example.test/latest.json",
        CODEX_ASSISTANT_UPDATE_PUBKEY: "public-key",
      }),
    /TAURI_SIGNING_PRIVATE_KEY/,
  );
});

test("updater build forwards only matching desktop targets", () => {
  const settings = updaterBuildSettings(
    [
      "--platform",
      "windows",
      "--target",
      "aarch64-pc-windows-msvc",
    ],
    {
      ...signing,
      CODEX_ASSISTANT_UPDATE_ENDPOINT: "https://updates.example.test/latest.json",
    },
  );
  assert.deepEqual(tauriBuildArguments(settings, "generated.json").slice(-2), [
    "--target",
    "aarch64-pc-windows-msvc",
  ]);
  assert.throws(
    () =>
      updaterBuildSettings(
        ["--platform", "mac", "--target", "aarch64-pc-windows-msvc"],
        {
          ...signing,
          CODEX_ASSISTANT_UPDATE_ENDPOINT: "https://updates.example.test/latest.json",
        },
      ),
    /does not match/,
  );
});
