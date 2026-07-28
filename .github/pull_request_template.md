## SPEC Traceability

- Requirement IDs:
- Milestone / work package:
- Related issue:

## User Behavior

Describe the user-visible behavior before and after this change.

## Failure And Recovery

- Failure states introduced or changed:
- Retry, rollback, or recovery behavior:
- Compatibility impact:

## Verification Evidence

- [ ] Rust formatting, Clippy, and tests pass.
- [ ] Relevant Windows or macOS build completes.
- [ ] Automated tests cover success, failure, and idempotent retry when applicable.
- [ ] Router keys, credentials, logs, installers, and generated artifacts are not committed.
- [ ] Configuration writes and rollback behavior were checked when affected.
- [ ] Windows VM acceptance was completed when ChatGPT installation, launch, or theming changed.

List commands, test cases, screenshots, logs, or release artifacts used as evidence.

## Security And Privacy

- [ ] Logs and exported diagnostics were checked for URL credentials, Access Keys, bearer tokens, and local personal paths.
- [ ] New downloads or executables have an explicit trusted source and integrity check.
- [ ] No new telemetry or external data transfer was added without a SPEC decision.

## Documentation

- [ ] `docs/spec-progress.zh-CN.md` is updated.
- [ ] The external `TRACEABILITY_MATRIX.md` is updated when requirement status changes.
- [ ] The external `DECISION_LOG.md` is updated for new cross-module or irreversible decisions.
