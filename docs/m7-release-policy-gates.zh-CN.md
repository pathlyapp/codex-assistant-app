# M7 发布分级与商业门禁

本文定义 `WP-700` 第一工作包，关联 `SECR-008`、`SECR-009`、`REL-041`、
`REL-042`、`REL-043` 和 `LIFE-004`。

## 1. 目标

在代码签名、macOS 公证、签名更新清单和客户下载服务尚未落地时，发布流程必须明确
区分“内部测试包”和“客户版本”，不能只依赖 GitHub 的 `prerelease` 标签或人工文案。

本工作包只建立发布策略门禁，不实现：

- Windows Authenticode 签名。
- macOS Developer ID 签名、公证或 stapling。
- 客户下载服务。
- 助手自动更新。
- 更新失败回退。

这些能力仍由 `WP-701`、`WP-702` 和 `WP-604` 关闭。

## 2. 发布渠道

`generate-release-manifest.mjs` 要求显式传入：

```text
--channel internal-test
```

当前允许的行为：

| 渠道 | 结果 |
| --- | --- |
| 未指定 | 失败，不生成发布清单 |
| `internal-test` | 生成 schema v2 清单、SHA256 和内部测试发布说明 |
| `customer` | 硬失败，直到代码签名与签名清单验证真正实现 |

工作流不能通过修改 Release 标题、Tag 或 `prerelease` 状态绕过该门禁。

## 3. 清单契约

`package-manifest.json` 升级为 schema v2：

```json
{
  "schemaVersion": 2,
  "product": "codex-assistant",
  "version": "0.9.0",
  "releasePolicy": {
    "channel": "internal-test",
    "customerReady": false,
    "codeSigning": "not_verified",
    "manifestSignature": "not_configured",
    "blockingReason": "unsigned_internal_test_only"
  },
  "artifacts": [
    {
      "file": "CodexAssistant-0.9.0-windows-arm64-setup.exe",
      "platform": "windows",
      "arch": "aarch64",
      "format": "nsis",
      "bytes": 0,
      "sha256": "<hex>",
      "signing": {
        "requiredForCustomer": true,
        "status": "not_verified"
      }
    }
  ]
}
```

`not_verified` 不等于确认“未签名”，只表示当前流程没有验证并形成可审计签名证据。
因此 `customerReady` 必须保持 `false`。

## 4. 发布说明

`RELEASE-NOTES.md` 由清单生成，必须包含：

1. 内部测试用途和非客户版本声明。
2. 每个平台产物、字节数和 SHA256。
3. 代码签名、公证、清单签名、自更新和真实设备门禁。
4. 覆盖安装方式。
5. 配置恢复、助手卸载和诊断包导出方式。
6. SHA256 只证明文件完整性、不证明发布者身份的说明。

Release 工作流使用该文件作为 GitHub prerelease 正文，并显式设置
`--latest=false`。

## 5. 失败语义

- 生成器参数错误、未知文件名或空产物目录：发布构建失败。
- 请求 `customer`：发布构建失败，不产生可误用的客户清单。
- 清单不是 schema v2、不是 `internal-test` 或意外标记 `customerReady=true`：
  发布说明生成失败。
- 任一阶段失败：不执行 `gh release create`。

## 6. 自动验证

`npm run test:release-policy` 使用隔离临时目录验证：

1. 内部清单固定为 schema v2 和 `customerReady=false`。
2. 三个平台产物元数据和 SHA256 正确。
3. 三个平台规范文件名必须完整，缺包、错版本和额外附件均被阻断。
4. 客户渠道被硬阻断。
5. 缺少渠道被硬阻断。
6. 发布说明包含限制、升级、恢复和完整性边界。

GitHub Actions YAML 继续由 `actionlint` 校验。

## 7. 供应链边界

GitHub artifact attestations 可以证明构建来源和工作流，但不能替代操作系统代码签名，
也不能单独使安装包成为客户版本。私有仓库是否可用 attestation 还受 GitHub 套餐
约束，作为 `WP-701` 的可选补充评估。

参考：

- [GitHub CLI release create](https://cli.github.com/manual/gh_release_create)
- [GitHub artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations)

## 8. 后续门禁

- `WP-701`：Windows/macOS 代码签名、公证与受托密钥管理。
- `WP-702`：客户下载服务、撤销和审计。
- `WP-604`：客户端签名清单验证、平台匹配、下载校验和失败回退。
- `DEC-012`：客户下载服务选择。
- `DEC-013`：签名供应商和 CI 托管方式。
- `DEC-014`：正式支持平台下限。
