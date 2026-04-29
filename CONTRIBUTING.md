# Contributing

## Releasing a new version

This crate is published to crates.io by CI whenever a tag matching `v*` is pushed to GitHub. The tag drives the release — there is no other trigger.

CI verifies that the tag (`vX.Y.Z`) matches the `version` in `Cargo.toml` and aborts the publish if they diverge.

We follow [semver](https://semver.org/): `MAJOR.MINOR.PATCH`.

- `MAJOR` — breaking changes to the public API
- `MINOR` — new functionality, backwards compatible
- `PATCH` — backwards compatible bug fixes

### Preferred: use `cargo release`

[`cargo-release`](https://github.com/crate-ci/cargo-release) is configured via [`release.toml`](release.toml) to bump `Cargo.toml`, create a `chore: release vX.Y.Z` commit, tag it `vX.Y.Z`, and push to `origin`. Publishing itself is left to CI (`publish = false`).

```sh
cargo install cargo-release   # one-time

cargo release patch --execute   # 0.8.0 -> 0.8.1
cargo release minor --execute   # 0.8.0 -> 0.9.0
cargo release major --execute   # 0.8.0 -> 1.0.0
```

Run from a clean `main` that's up to date with `origin/main`. Without `--execute`, `cargo release` runs as a dry-run.

### Manual release

If the version was already bumped in `Cargo.toml` as part of a feature commit (or `cargo release` can't be used for some reason), you have to create and push the tag yourself. **The tag name must exactly match the `version` in `Cargo.toml`, prefixed with `v`** — CI will reject the publish otherwise.

```sh
# Confirm Cargo.toml version, then:
git tag v0.8.0
git push origin v0.8.0
```

After pushing, also remember to run `cargo update -p shuriken-sdk` (or the equivalent) in any consumers so `Cargo.lock` picks up the new release.
