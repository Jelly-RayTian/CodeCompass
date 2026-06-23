# Releasing CodeCompass

## Quick Release (local)

```bash
npm run tauri:build
```

Artifacts are in `src-tauri/target/release/bundle/`:

- NSIS: `bundle/nsis/CodeCompass_0.1.0_x64-setup.exe`
- MSI: `bundle/msi/CodeCompass_0.1.0_x64_en-US.msi`

## Versioning

Semantic versioning: `MAJOR.MINOR.PATCH`

Update version in three places:

1. `package.json` — `"version"` field
2. `src-tauri/tauri.conf.json` — `"version"` field
3. `src-tauri/Cargo.toml` — `[package] version` field

## Tag & Release

```bash
git tag v0.1.0-alpha
git push origin v0.1.0-alpha
```

Pushing a tag matching `v*` triggers the GitHub Actions release workflow
(`.github/workflows/release.yml`), which:

1. Runs the full CI suite
2. Builds NSIS and MSI installers
3. Creates a GitHub prerelease with uploaded artifacts

## Installation

### Windows

Download the NSIS `.exe` or MSI `.msi` from the [Releases page](https://github.com/Jelly-RayTian/CodeCompass/releases).

Run the installer. CodeCompass installs to `%LOCALAPPDATA%\CodeCompass`.

### Uninstall

- **NSIS**: Run `Uninstall CodeCompass.exe` from the install directory, or use _Settings → Apps → Installed apps_.
- **MSI**: Use _Settings → Apps → Installed apps_ → CodeCompass → Uninstall.

## Known Limitations (Windows)

- **SmartScreen warning**: Installers are unsigned. Click "More info" → "Run anyway".
- **Installer identifier**: The identifier has been changed to `io.github.jellyraytian.codecompass`. This means v0.1.0-alpha installs are **not compatible** with future releases using a different identifier — uninstall the old version first.

- **Identifier change**: Changed from `com.codecompass.app` to `io.github.jellyraytian.codecompass` before the first public release. This follows the reverse-domain convention with the GitHub account name.
- **Icons**: Uses Tauri's default placeholder icons. Replace with branded icons before a stable release.
- **No auto-update**: Users must manually download new versions.

## Pre-release Checklist

- [ ] All tests pass: `cargo test && npm run test`
- [ ] Clippy clean: `cargo clippy --all-targets -- -D warnings`
- [ ] Build succeeds: `npm run tauri:build`
- [ ] Version numbers aligned across `package.json`, `Cargo.toml`, `tauri.conf.json`
- [ ] CHANGELOG updated
- [ ] Tag created: `git tag vX.Y.Z`
- [ ] Tag pushed: `git push origin vX.Y.Z`
- [ ] GitHub Release created by CI
- [ ] Installers uploaded and downloadable
- [ ] Smoke test: install and launch the `.exe`
