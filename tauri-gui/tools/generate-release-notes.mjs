import { promises as fs } from "node:fs";
import { resolve } from "node:path";

function argument(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

const manifestPath = resolve(
  argument("--manifest") || "release-assets/package-manifest.json",
);
const outputPath = resolve(
  argument("--output") || "release-assets/RELEASE-NOTES.md",
);
const manifest = JSON.parse(await fs.readFile(manifestPath, "utf8"));

if (
  manifest.schemaVersion !== 2 ||
  manifest.releasePolicy?.channel !== "internal-test" ||
  manifest.releasePolicy?.customerReady !== false
) {
  throw new Error("Release notes require an internal-test schema v2 manifest");
}
if (!Array.isArray(manifest.artifacts) || manifest.artifacts.length === 0) {
  throw new Error("Release manifest does not contain artifacts");
}

const artifactLines = manifest.artifacts
  .map(
    (artifact) =>
      `- \`${artifact.file}\` (${artifact.platform}/${artifact.arch}, ${artifact.bytes} bytes, SHA256 \`${artifact.sha256}\`)`,
  )
  .join("\n");

const notes = `# Codex Assistant ${manifest.version} - Internal Test Build

> This prerelease is for internal testing and controlled demonstrations only. It is not a customer-ready release.

## Available Artifacts

${artifactLines}

## Known Limitations

- Windows executables have not completed the commercial code-signing gate.
- The macOS application has not completed Developer ID signing or notarization.
- The release manifest is checksummed but not cryptographically signed.
- Automatic assistant updates are disabled until signed-manifest verification and rollback are implemented.
- Real Windows x64 hardware and signed macOS acceptance remain separate release gates.

## Upgrade

- Install the matching platform package over the previous internal build.
- The installer must not launch Codex Assistant before installation completes.
- Existing ChatGPT, Codex configuration, and assistant data remain outside the default assistant upgrade boundary.

## Recovery And Uninstall

- Reinstall the same verified package if the assistant program is damaged.
- Use **Restore original configuration** before deleting assistant data that is still referenced by Codex.
- Uninstalling Codex Assistant preserves ChatGPT, Codex configuration, and assistant data by default.
- Export a redacted diagnostic bundle before destructive troubleshooting.

## Integrity

- Verify every package against \`SHA256SUMS.txt\` and \`package-manifest.json\`.
- SHA256 verifies file integrity, not publisher identity. Do not treat these unsigned artifacts as customer releases.
`;

await fs.writeFile(outputPath, notes, "utf8");
console.log(`Generated internal-test release notes at ${outputPath}.`);
