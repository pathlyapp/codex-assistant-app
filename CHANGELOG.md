# Changelog

All notable changes to Codex Assistant are documented in this file.

## [Unreleased]

## [0.8.7] - 2026-07-28

- The overview primary action now installs ChatGPT first when it is missing, then continues into Router setup, so "开始配置" no longer requires filling the form before the app is present.

## [0.8.6] - 2026-07-28

- Added a one-click factory reset on the diagnostics page that stops and uninstalls ChatGPT, removes the managed Codex configuration, and deletes all local assistant data, with per-step results and a strong confirmation.

## [0.8.5] - 2026-07-28

- Added a one-click ChatGPT download and install action on the home environment check, using the Microsoft Store official channel when ChatGPT is missing.

## [0.8.4] - 2026-07-19

- Established the Rust and Tauri implementation as the only product mainline.
- Added real ChatGPT detection, Router validation, model discovery, Codex configuration, diagnostics, and recovery.
- Added Windows DPAPI protection and the Rust token helper.
- Added reversible official, focus, and custom-image appearance modes.
- Removed payment, activation, mock token, legacy Go, and legacy PowerShell installer flows.
