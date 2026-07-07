# Releasing CodeCompass

## Quick Release (local)

```bash
npm run tauri:build
```

Artifacts are in `src-tauri/target/release/bundle/`:

- NSIS: `bundle/nsis/CodeCompass_0.1.1_x64-setup.exe`
- MSI: `bundle/msi/CodeCompass_0.1.1_x64_en-US.msi`

## Versioning

Semantic versioning: `MAJOR.MINOR.PATCH`

The version must be aligned across **four** sources:

1. `package.json` — `"version"` field
2. `src-tauri/tauri.conf.json` — `"version"` field
3. `src-tauri/Cargo.toml` — `[package] version` field
4. The Git tag (`vX.Y.Z`)

### Version validation

A release-time script fails when any of these disagree:

```bash
npm run check:versions                       # internal consistency
node scripts/check-versions.mjs --tag=v0.1.1   # also check the tag
```

The CI and Release workflows run `check:versions` automatically; the
Release workflow validates the tag against the three manifest versions
before building.

## Tag & Release

```bash
# 1. Confirm all checks pass locally:
npm run lint && npm run typecheck && npm run test && npm run build
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo check && cd ..
npm run check:versions

# 2. Confirm the installer builds and the icon is the branded compass:
npm run tauri:build

# 3. Tag exactly (do not let CI infer the version):
git tag v0.1.1
git push origin v0.1.1
```

Pushing a tag matching `v*` triggers the GitHub Actions release workflow
(`.github/workflows/release.yml`), which:

1. Validates version alignment against the tag (`check:versions --tag=…`)
2. Runs the full CI suite (lint, typecheck, test, build, fmt, clippy, test, check)
3. Builds NSIS (required) and MSI (optional) installers
4. Fails clearly if the NSIS installer is missing
5. Creates a GitHub **prerelease** with uploaded artifacts and unsigned-installer notes

### Permissions

The release workflow uses the least privilege required:
`permissions: contents: write` (needed to create the GitHub Release). No
secrets are committed; the default `GITHUB_TOKEN` is used.

## Installation

### Windows

Download the NSIS `.exe` or MSI `.msi` from the [Releases page](https://github.com/Jelly-RayTian/CodeCompass/releases).

Run the installer. CodeCompass installs to `%LOCALAPPDATA%\CodeCompass`.

The window icon, installer icon, and Start Menu icon all use the
branded CodeCompass compass badge (generated from
`src-tauri/icons/icon-source.svg` via `npx tauri icon`).

### Uninstall

- **NSIS**: Run `Uninstall CodeCompass.exe` from the install directory, or use _Settings → Apps → Installed apps_.
- **MSI**: Use _Settings → Apps → Installed apps_ → CodeCompass → Uninstall.

## Known Limitations (Windows)

- **SmartScreen warning**: Installers are unsigned. Click "More info" → "Run anyway".
- **Installer identifier**: The identifier is `io.github.jellyraytian.codecompass`. v0.1.0-alpha installs are **not compatible** with future releases using a different identifier — uninstall the old version first.
- **No auto-update**: Users must manually download new versions. The app performs no runtime update checks.

## Pre-release Checklist

- [ ] All tests pass: `cargo test && npm run test`
- [ ] Clippy clean: `cargo clippy --all-targets -- -D warnings`
- [ ] Build succeeds: `npm run tauri:build`
- [ ] Version numbers aligned: `npm run check:versions`
- [ ] Benchmark run: `npm run bench:summary` (paste into `docs/benchmarks.md`)
- [ ] Screenshots captured into `docs/screenshots/` (see checklist)
- [ ] Privacy audit re-run (see [docs/privacy-audit.md](privacy-audit.md))
- [ ] Tag created: `git tag vX.Y.Z`
- [ ] Tag pushed: `git push origin vX.Y.Z`
- [ ] GitHub Release created by CI
- [ ] Installers uploaded and downloadable
- [ ] Smoke test: install and launch the `.exe`, verify branded icon
