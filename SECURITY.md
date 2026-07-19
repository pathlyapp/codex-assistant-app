# Security Policy

## Reporting

This is a private commercial repository. Report vulnerabilities through a private GitHub Security Advisory for this repository. Do not include production Router keys, customer configuration, signing credentials, or access tokens in issues, pull requests, screenshots, or logs.

## Supported Version

Only the latest release line is supported during the MVP stage.

## Repository Rules

- Never commit `.env` files, Router keys, runtime state, signing certificates, or decrypted credentials.
- Windows secrets must remain protected with current-user DPAPI.
- Release signing credentials must be stored in GitHub Secrets or an approved signing service.
- Logs and diagnostics must redact authorization headers, query-string keys, and bearer tokens.
- Security-sensitive changes require review of token handling, process identity validation, CDP loopback restrictions, and rollback behavior.
