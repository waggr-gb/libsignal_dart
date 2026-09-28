# waggr-gb fork of libsignal_dart

Upstream: <https://github.com/djx-y-z/libsignal_dart> (pub package `libsignal`).
Consumer: [waggr-gb/waggr-flutter](https://github.com/waggr-gb/waggr-flutter),
which depends on this repository by **git tag**, never via pub.dev.

## Why this fork exists

Waggr needs changes upstream has not (yet) shipped. The first one: every Rust
API error surfaces as a typed `LibSignalException { code, message }` instead of
a bare `String`, so the app classifies failures by a stable `code` and never by
matching message text (upstream issue djx-y-z/libsignal_dart#106).

## What differs from upstream

| Area | Upstream | This fork |
|------|----------|-----------|
| Errors | `String` thrown from Rust | `LibSignalException { code, message }` (`rust/src/api/error.rs`) |
| Native binaries | `djx-y-z/libsignal_dart` releases | `waggr-gb/libsignal_dart` releases (`_githubRepo` in `hook/build.dart`, and `scripts/check_exists_frb_release.dart`) |
| Versions | Dart 7.3.1 / crate `libsignal_frb` 6.3.1 | Dart **8.0.0** / crate **7.0.0** |
| pub.dev | published | `publish_to: none` — never published |
| Automation | scheduled bots, AI review, pub.dev publish | disabled — see below |

Checksum verification in the build hook is unchanged and still fail-closed.
Never set `LIBSIGNAL_ALLOW_UNVERIFIED_DOWNLOAD` in anything Waggr ships.

## Build policy: we build when we change it

Nothing in this fork builds, tests or opens pull requests on a schedule. Native
binaries are built **only** when we deliberately release:

1. Change the code; bump `rust/Cargo.toml` `[package] version` (and the
   `libsignal_frb` entry in `rust/Cargo.lock`) whenever the Rust side or the
   vendored libsignal changes. Bump `pubspec.yaml` `version` too.
2. Merge to `main`.
3. Push the tag `libsignal_frb-<crate version>` on that commit. The tag push
   triggers `build-libsignal.yml`, which builds every platform, writes
   `libsignal_frb-<ver>-checksums.sha256`, attests provenance and creates the
   GitHub Release the build hook downloads from.
4. Push the Dart tag `v<pubspec version>` on the same commit (plain git tag;
   `publish.yml` is disabled here, so nothing is published) and point
   waggr-flutter's `pubspec.yaml` at it.

Upstream's `make release-frb` creates a **signed** commit + tag. The signature
is local tooling plus upstream's `protect-release-tags` ruleset; the workflow
itself does not verify it and a fork inherits no rulesets. We push an
**annotated** tag by hand (`git tag -a libsignal_frb-X.Y.Z -m ...`). The
workflow still validates that the tag equals the crate version, that
`THIRD_PARTY_NOTICES.txt` matches the dependency graph, and it never
overwrites an existing release.

If a tag push does not start a run (it happened for the fork's first release,
`libsignal_frb-7.0.0`: the tag was pushed moments after the fork's workflows
were first registered), dispatch the build ON THE TAG:
`gh workflow run build-libsignal.yml -R waggr-gb/libsignal_dart --ref libsignal_frb-X.Y.Z`.
`github.ref_type` is then `tag`, so the tag/crate-version check still runs.

### Workflows

| Workflow | State | Why |
|----------|-------|-----|
| `build-libsignal.yml` | enabled | the release path (tag push only) |
| `test.yml` / `test-reusable.yml` | enabled | runs on PRs, pushes to `main`, and after a build — only when we change something; GitHub-hosted, free on a public repo |
| `codegen-guard.yml` | enabled | PR-only bindings drift check |
| `check-libsignal-updates.yml` | **disabled** | daily cron, opens update PRs |
| `check-template-updates.yml` | **disabled** | daily cron, opens template PRs |
| `refresh-notices.yml` | **disabled** | daily cron for Dependabot PRs |
| `repair-build.yml` | **disabled** | twice-daily cron, AI repair agent |
| `fuzz.yml` | **disabled** | weekly cron (run by hand if wanted) |
| `ai-review.yml` | **disabled** | needs upstream's secrets |
| `publish.yml` | **disabled** | would publish to pub.dev under upstream's name |

Disabled with `gh workflow disable <file> -R waggr-gb/libsignal_dart`; state
lives in the repository settings, not in these files, so syncing upstream does
not re-enable them. Dependabot is off for the fork as well.

## Syncing upstream

```bash
git remote add upstream https://github.com/djx-y-z/libsignal_dart.git
git fetch upstream
git checkout -b sync-upstream origin/main
git merge upstream/main          # resolve: keep waggr-gb repo refs + our versions
# regenerate FRB bindings if rust/src/api changed, run the gates:
#   codegen, cargo clippy -D warnings, cargo test, dart analyze, dart test
```

If the merge changes anything under `rust/` (or the vendored libsignal), bump
the crate version and cut a release as above; the app must move to the new tag.
