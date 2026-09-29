# Forge release

La release `v0.1.0` est publiée : <https://github.com/Cliedd/Lama_Facher/releases/tag/v0.1.0>. Le workflow du tag a terminé avec succès, y compris `build (macos-15-intel, macos-x86_64)` et le job `release`. Les cinq assets attendus (quatre archives et `SHA256SUMS`) sont présents. Vérification reproductible : `gh run view 36535908555 --repo Cliedd/Lama_Facher` et `gh release view v0.1.0 --repo Cliedd/Lama_Facher`.

The package version in `Cargo.toml` is `0.1.0`. After the release commit passes CI on `main`, create and push the matching tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The tag workflow checks that the tag matches the Cargo version. It builds and tests on Linux x86-64, macOS Intel, macOS Apple Silicon, and Windows x86-64. It then publishes four archives in GitHub Releases:

| Platform | Archive |
| --- | --- |
| Linux x86-64 | `forge-linux-x86_64.tar.gz` |
| macOS Intel | `forge-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `forge-macos-aarch64.tar.gz` |
| Windows x86-64 | `forge-windows-x86_64.zip` |

Each archive contains the executable and `exercises/`. The release also includes `SHA256SUMS` for all four archives. The Unix and Windows installers verify the archive against this file before extracting it. Installation from GitHub Releases needs no Cargo or Git. The Unix installer requires `curl` and `tar`.

To check an archive manually, download it alongside `SHA256SUMS`, then run `sha256sum -c SHA256SUMS` on Linux or `shasum -a 256 -c SHA256SUMS` on macOS. The check command reports missing files for archives you did not download; use the matching line from `SHA256SUMS` when checking a single archive.

The release is published only after every build succeeds. The installation commands work after GitHub finishes publishing the release.
