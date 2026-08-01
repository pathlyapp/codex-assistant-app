# Changelog

All notable changes to Codex Assistant are documented in this file.

## [Unreleased]

## [0.9.1] - 2026-08-01

- Added a signed in-app updater for the assistant itself: automatic and manual update checks, signature-verified downloads, one-click install, and post-update health confirmation.
- Added model input-modality awareness: the app reads input modalities (text, image, video) declared by the Router /models endpoint and shows per-model capability hints in the UI.
- Added a Rust-generated support diagnostic bundle with bounded redacted logs, safe system and transaction summaries, per-file checksums, support IDs, and an export-blocking secret scan.
- Added Rust-owned targeted repair plans with pre-execution state revalidation, before/after receipts, Router revalidation, transactional configuration recovery, safe official-app rechecks, and stale appearance-session cleanup.
- Replaced the destructive all-in-one factory reset with explicit assistant uninstall, transactional managed-config cleanup, dependency-safe data deletion, and OS-owned ChatGPT management.
- Added release policy gates that mark unsigned artifacts as internal-test only, block customer-channel metadata, and generate explicit limitations, upgrade, recovery, and integrity notes.
- Streamlined the setup flow and simplified frontend UI elements.
- Refreshed user-facing terminology across the frontend and contracts for clarity and consistency.
- Updated application icons and bundled assets.

## [0.8.8] - 2026-07-28

- Integrated DreamSkin gallery themes into the appearance page: three bundled preset themes (悟空 WUKONG, firefly, 保险柜 办公室 卡通, redistributed under MIT/CC BY with attribution) plus an online popular-theme library fetched from dreamskin.cc with hash-verified downloads, per-theme licensing display, and offline reuse after first apply.
- Reworked theme injection to render CSS from each theme's color palette and artwork focus instead of a single fixed template, while keeping official, focus, and custom-image modes unchanged.

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
