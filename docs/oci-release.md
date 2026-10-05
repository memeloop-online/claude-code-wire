# Signed OCI release and isolated installation

The `Signed OCI package` workflow packages the existing v1.0.1 release without rebuilding or modifying its WASM or manifest. `.github/oci-release.json` pins both files to the exact SHA-256 values exercised by MTC PR #446, plus the installer image published by [MTC run 37108937447](https://github.com/memeloop-online/memeloop-token-center/actions/runs/37108937447). Updating the release or installer requires reviewing that record in a PR. No mutable installer tag or dispatch-supplied executable is trusted.

The package has artifact type `application/vnd.memeloop.token-center.plugin.v1`, config type `application/vnd.memeloop.token-center.plugin.config.v1+json`, and separate `plugin.json` and `plugin.wasm` layers using MTC's manifest and WASM media types. It contains no image filesystem or tar layer.

## Publication and trust

PRs download and check the pinned release bytes without registry write or OIDC permissions. Pushes to `release/oci-*` exercise signing and installation in `ghcr.io/memeloop-online/claude-code-wire-candidate`; this is a rehearsal package. Master pushes or a manual master dispatch publish `ghcr.io/memeloop-online/claude-code-wire`. The `v1.0.1` source tag and GitHub Release predate this OCI workflow; successful signature verification, negative signature testing, and installation gate OCI publication, not source-tag creation. Consumers must pin the resulting digest, not the tag.

The official certificate identity is exactly:

```
https://github.com/memeloop-online/claude-code-wire/.github/workflows/publish-oci.yml@refs/heads/master
```

The issuer is `https://token.actions.githubusercontent.com`. Candidate evidence records the exact branch identity instead; do not add it to production trust. The workflow extracts `v3.1.3-mtc.3` Cosign from the pinned installer, checks the image source revision and verifier compatibility, signs with GitHub OIDC, and preserves all default certificate, transparency-log, and signature checks. It uses the existing workflow token for GHCR; it neither creates credentials nor changes package visibility.

## Exact test/install plan

1. Require green native CI and `package` checks on the PR. Review the candidate `publish` run and its `oci-evidence-<commit>` artifact.
2. Require `plugin-release.json` to identify the expected source, digest, source revision, installer digest, exact certificate identity, issuer, and both release hashes. Its `official` field is false for a candidate. Inspect the raw OCI manifest, Cosign verification JSON, installed receipt, and negative signature result in the same artifact.
3. The Actions rehearsal runs the pinned real installer in a read-only container, with dropped capabilities, an isolated writable plugin directory, temporary registry credential files, and no MTC service/account. A wrong certificate identity must fail specifically with category `signature` and leave no installed plugin. The correct identity must install both files byte-for-byte and record `cosign-keyless` provenance.
4. After merge, wait for successful official master publication. Download that run's evidence and use its `install-plan.md`, which contains the complete digest-pinned reference and exact installer invocation. Candidate success is not evidence of a master signature. With an existing authorized registry credential, repeat in an empty disposable directory; no dependency install or build is needed. Never disable verification to work around registry or signer errors.
5. For host ABI acceptance, the identical v1.0.1 files already passed MTC's `privacy_wire_shim_release_component` ignored tests and `installable_example_contributes_provider_oauth_policy_and_rewrite` in run 37108937447. If repeating on another host revision, run those checks only in GitHub Actions, setting `MTC_CLAUDE_WIRE_FIXTURE` to the installed package directory. This repository does not modify host code or claim a new host test run.
6. Keep production login, provider account setup, OAuth, activation, and ledger updates with the parent owner. This packaging change neither calls a provider nor changes plugin behavior. Remove disposable install directories and credential files after acceptance.

The workflow summary includes the immutable reference and signing identity. Evidence artifacts are retained for 90 days; retain the evidence with the release handoff before expiry. Registry access is proven with the workflow token; anonymous and deployment-account access must be checked independently using already-authorized access.
