# Contributing

Thanks for contributing to EnergiaPro.

## Local checks

Run these commands before opening a pull request:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked
```

## CI pipeline

The CI workflow runs on every push and pull request. It validates formatting,
lints the workspace, runs tests, and builds the CLI on Linux, macOS, and
Windows.

## Release pipelines

Releases are automated with GitHub Actions and use separate versioning for the
CLI and SDK.

### Required repository configuration

Set these repository secrets in `nhedger/energiapro`:

- Secret `CARGO_REGISTRY_TOKEN`: crates.io API token used to publish
  `energiapro-sdk`.
- Secret `HOMEBREW_TAP_SSH_KEY`: private SSH deploy key with write access to
  `nhedger/energiapro-homebrew` used to sync `Formula/energiapro.rb`.

Configure the matching public key as a write-enabled deploy key in
`nhedger/energiapro-homebrew`.

### Tag conventions

- CLI releases use tags that match `cli-vX.Y.Z`.
- SDK releases use tags that match `sdk-vX.Y.Z`.

### CLI release workflow

When `cli-vX.Y.Z` is pushed:

1. The workflow checks that the tag version matches
   `crates/energiapro-cli/Cargo.toml`.
2. It builds release binaries for:
   - `x86_64-unknown-linux-gnu`
   - `x86_64-apple-darwin`
   - `aarch64-apple-darwin`
   - `x86_64-pc-windows-msvc`
3. It creates a GitHub release in `nhedger/energiapro` with platform binaries
   and a `SHA256SUMS.txt` file.
4. It renders `crates/energiapro-cli/homebrew/energiapro.rb` by replacing
   `{{version}}` and checksum placeholders with release values, then commits
   the result to `nhedger/energiapro-homebrew` as `Formula/energiapro.rb`.

### SDK release workflow

When `sdk-vX.Y.Z` is pushed:

1. The workflow checks that the tag version matches
   `crates/energiapro-sdk/Cargo.toml`.
2. It runs a packaging validation step.
3. It publishes `energiapro-sdk` to crates.io.

### Cut a new CLI release

1. Update `crates/energiapro-cli/Cargo.toml`.
2. Merge the release commit to `main`.
3. Create and push the tag:

```sh
git tag cli-vX.Y.Z
git push origin cli-vX.Y.Z
```

### Cut a new SDK release

1. Update `crates/energiapro-sdk/Cargo.toml`.
2. Merge the release commit to `main`.
3. Create and push the tag:

```sh
git tag sdk-vX.Y.Z
git push origin sdk-vX.Y.Z
```

### Verify the result

After workflows complete, verify:

- For CLI tags: a GitHub release exists with all expected platform binaries and
  `SHA256SUMS.txt`.
- For SDK tags: `energiapro-sdk` appears on crates.io at the tagged version.
