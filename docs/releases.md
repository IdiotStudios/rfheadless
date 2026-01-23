# Releases

This project publishes native desktop binaries for Windows, Linux, and macOS via a GitHub Actions workflow.

What is included in a release:

- Linux: `rfheadless-VERSION-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `rfheadless-VERSION-x86_64-pc-windows-msvc.zip`
- macOS: `rfheadless-VERSION-x86_64-apple-darwin.tar.gz` and `rfheadless-VERSION-aarch64-apple-darwin.tar.gz`

Verifying artifacts:

- On Linux: `sha256sum -c rfheadless-VERSION-x86_64-unknown-linux-gnu.tar.gz.sha256`
- On macOS: `shasum -a 256 -c rfheadless-VERSION-x86_64-apple-darwin.tar.gz.sha256`
- On Windows (PowerShell): `Get-FileHash .\\rfheadless-VERSION-x86_64-pc-windows-msvc.zip -Algorithm SHA256`

How releases are triggered:

- Push an annotated tag matching `v*` (eg):

```bash
  git tag -a v0.2.0 -m "v0.2.0"
  git push origin v0.2.0
```

- Or trigger the workflow manually from the Actions tab (workflow dispatch).


Building and packaging locally (for testing):

- Linux:

```bash
cargo build --release
tar -czf rfheadless-VERSION-x86_64-unknown-linux-gnu.tar.gz -C target/release rfheadless
```

- macOS:

```bash
cargo build --release --target x86_64-apple-darwin
# or
cargo build --release --target aarch64-apple-darwin

# package
tar -czf rfheadless-VERSION-x86_64-apple-darwin.tar.gz -C target/x86_64-apple-darwin/release rfheadless
```

---