# Releases

This project publishes native desktop binaries for Windows and Linux via a GitHub Actions workflow.

What is included in a release:

- Linux: `rfheadless-VERSION-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `rfheadless-VERSION-x86_64-pc-windows-msvc.zip`

How releases are triggered:

- Push an annotated tag matching `v*` (eg):

```bash
  git tag -a v0.2.0 -m "v0.2.0"
  git push origin v0.2.0
```

- Or trigger the workflow manually from the Actions tab (workflow dispatch).

---