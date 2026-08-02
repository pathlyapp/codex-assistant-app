# Codex 助手

**第三方模型接入 + 个性化背景，全部在本地完成配置。**

Codex 助手是一款桌面配置工具，把「准备第三方模型服务 → 测试连接 → 选择模型 → 写入配置 → 验证结果 → 打开 Codex → 设置个性化背景」整理成一条清晰、可检查、可恢复的本地流程。

[![Release](https://img.shields.io/github/v/release/pathlyapp/codex-assistant-app?include_prereleases)](https://github.com/pathlyapp/codex-assistant-app/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-blue)](https://github.com/pathlyapp/codex-assistant-app/releases)

> 产品介绍与视频演示：**[产品主页](https://pathlyapp.github.io/codex-assistant-app/)**
> 中文详细说明：[README.zh-CN.md](README.zh-CN.md)

## 它解决什么问题

很多用户不是不会使用 AI，而是在真正开始之前，就被账号、网络和付费环节挡住了：

1. **账号注册与验证麻烦** — 注册流程、地区限制或验证步骤增加了使用门槛
2. **网络访问不够稳定** — 连接不稳定影响登录、模型调用和日常使用
3. **购买、支付和续费不方便** — 海外支付、套餐选择和持续续费让长期使用变得复杂

换一种思路：**配置你已有的第三方模型服务**。Codex 助手支持接入 OpenAI-compatible 模型服务，用户准备好服务地址和访问密钥后，助手完成连接测试、模型读取、配置写入和结果验证。

## 功能特性

- **状态首页** — 一眼看到 ChatGPT、模型服务和 Codex 配置状态，需要处理的项目直接显示
- **模型服务接入** — 填写服务地址与 Access Key，真实测试连接，从接口读取真实模型列表，不用占位数据
- **账号与数据** — 查看 Codex 账号（邮箱/套餐/用量额度）与本地会话、任务、存储概览
- **主题换肤** — 官方外观、专注深色、在线精选主题，或使用自己的本地图片（PNG/JPEG/WebP，本地校验不上传）
- **安全与可恢复** — 密钥本地加密保存（Windows DPAPI），不写明文进配置文件；每次修改前保留备份，可一键恢复
- **真实验证** — 任一检查失败都会明确显示失败，不允许"只写配置"伪装成完整成功

## 下载安装

从 [Releases](https://github.com/pathlyapp/codex-assistant-app/releases) 下载最新版本：

| 平台 | 文件 |
| --- | --- |
| Windows x64 | `CodexAssistant-*-windows-x64-setup.exe` |
| macOS Apple Silicon | `CodexAssistant-*-macos-arm64.app.zip` / `.dmg` |

每个发布附带 `SHA256SUMS.txt` 校验文件。

**使用前你只需要准备：**

1. 已安装并完成必要初始化的官方 ChatGPT 客户端
2. 第三方服务商提供的 OpenAI-compatible 接口地址
3. 服务商提供的 Access Key，以及你希望使用的模型

## 快速开始

1. **打开助手，检查当前状态** — 首页检查 ChatGPT 是否安装、模型服务是否可访问、Codex 配置是否完成
2. **填写第三方模型服务信息** — 进入"模型服务"，填写服务地址和访问密钥，点击"测试连接"
3. **读取真实模型并选择默认模型** — 连接成功后选择需要使用的模型，保存配置
4. **确认重启 ChatGPT** — 验证完成后由你确认重启，让新配置稳定加载
5. **进入 Codex，发送一个真实任务** — 模型名称会显示在输入区域，便于确认当前使用的服务

## 重要边界

- Codex 助手**不是** OpenAI 官方客户端，不代办账号注册或网络服务，也不重新分发或修改 ChatGPT
- 普通聊天、账号、订阅和聊天记录不受第三方模型配置影响；相关设置只作用于 Codex 模式及读取同一配置的 CLI/IDE
- 第三方服务由用户自行选择；使用前应阅读服务商条款，确认费用与隐私政策，并遵守所在地法律法规
- 不要在截图、录屏或公开文档中展示完整 Access Key

## 本地开发

```bash
cd tauri-gui
npm ci
npm run dev        # 开发模式
npm run build:mac  # macOS 打包
```

Windows 可分发包需在 Windows 环境构建：`npm run build:windows`。详见 [README.zh-CN.md](README.zh-CN.md) 与 `docs/`。

## License

[MIT](LICENSE) © 2026 523Tech
