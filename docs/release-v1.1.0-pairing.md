# v1.1.0 release-byte pairing

This record binds actual downloaded GitHub Release assets, not predicted hashes
or a source candidate alone. It does not authorize OCI publication, installation,
TEST/PROD activation, OAuth or inference. Subscription login is not verified.

## Source and publisher

- Source and lightweight tag `v1.1.0`: `ab7df5591b8afbb73d8b2a164c90df6e7593ef78`.
- Cargo and manifest versions: `1.1.0`.
- Existing tag-triggered [CI run 38178017688](https://github.com/memeloop-online/claude-code-wire/actions/runs/38178017688): test and release jobs both succeeded.
- [Actual Release](https://github.com/memeloop-online/claude-code-wire/releases/tag/v1.1.0) published at `2026-10-11T21:51:17Z`.
- Workflows, permissions and signature gates are unchanged. The installer pin
  is updated to the compatible published source documented below.

## Downloaded assets

| File | GitHub asset ID | SHA-256 |
| --- | --- | --- |
| `plugin.json` | `631897885` | `118cff3218b1d4512477d6fdef17001841a18c3393a81b586801cf39f5d21031` |
| `plugin.wasm` | `631897889` | `51043af426632581a218c227a0f90b2de2253d27b5438519098c44d965c01f3c` |
| `SHA256SUMS` | `631897887` | `3e1777308502937713ef704beba4c268f85533fdf1b8aac9c216d65e422bd7e1` |

All three files were downloaded anew from the actual Release after publication.
`sha256sum --check --strict SHA256SUMS` passed for both payloads. Their computed
hashes match GitHub's asset digests. All three also compare byte-for-byte with
the exact master CI run 38177006128 candidate; the manifest compares byte-for-byte
with the tagged source. The WASM remains identical to 1.0.1 because the runtime
privacy implementation is unchanged; the manifest is different.

## Pending promotion boundary

The release-byte pairing changes the proposed release version/source/manifest
hash. R1 additionally requires a compatible, actually published installer pin;
the old production pin remains unchanged until a separately approved merge.
Merging this PR onto master triggers the existing automatic OCI signing and
installer workflow and therefore must not be treated as a documentation-only
action. No such merge or OCI publication was performed for this preparation.

R1 established a known failure: the original installer revision
`2cd0580f6546044dcb40f06ecf53b4271bba5bde` rejects the new manifest's typed OAuth
fields. The existing workflow pushes and signs before its installation check,
so using that installer can leave a signed SHA artifact even though publication
fails before the version alias update. PR packaging did not cover this failure.

The earlier replacement was the official installer produced after core PR582's normal
merge at `93bcc9d1d51133055ba48cfab2ae606efc2b4a97` (including PR583):
`ghcr.io/memeloop-online/memeloop-token-center-plugin-installer@sha256:0e9bec4cd0393e56b2ba382a48c5b1744339be2a41a828e0ab2ff7a185a61d52`.
[Official run 38182879323](https://github.com/memeloop-online/memeloop-token-center/actions/runs/38182879323)
succeeded, including both image publications and immutable release verification.
The delegated owner's receipt binds the live installer index digest to its
same-run artifact, raw image revision label and subject-bound SPDX/SLSA evidence.
Its source-bound manifest schema supports the PR580 declarations and is compiled
into the installer from the same release input. Only installer digest, source
revision and evidence URL bind this artifact; repository and signer policy remain unchanged.
PR assembly and UI-only artifacts were not used as substitutes. No guessed
digest, schema relaxation or skipped validation is used. This receipt establishes
artifact/schema provenance, not installation of plugin 1.1.0 or subscription
runtime acceptance; neither was performed for this pin correction.
Host/UI deployment and subscription entitlement remain separate acceptance.
Final R2 identified a second consumer mismatch: the source-bound installer bundles
Cosign `v3.1.3-mtc.4`, while the publisher still expected mtc.3. The exact expected
version now lives in `installer.cosign_version` in the pin record; both the
publisher gate and evidence validation consume that single value. A static
contract in the existing native test suite binds this version to the reviewed
installer source/digest and checks the consumers and retained safety gates.
The consumer mismatch was closed, but the later master publication at
`cf403151514c727d8fd11daab41c59033ea86955`, run 38191320301, failed after
push/signature verification: the source93 installer rejected the actual package.
The signed artifact remains at
`ghcr.io/memeloop-online/claude-code-wire@sha256:a77fffb8280029527ae52d96b7b424aef94d5bab580572fb3d7741cb2d124d07`;
its version-alias promotion and positive installation receipt were not completed.
Do not rerun or retag that failed revision.

Core PR587 identified the precise schema/runtime mismatch: the closed manifest
schema omitted the existing `responses_via_anthropic_messages_v1` boolean.
The immutable public two-file package regression passed in exact PR CI
38195072740, explicitly through the existing ignored-test entry, after that
property was added without loosening unknown-field or runtime safety checks.
The normal merge is `3dbf18898357cd00ab33e8a79523d85dd6a08367`.
[Official run 38198375515](https://github.com/memeloop-online/memeloop-token-center/actions/runs/38198375515)
is successful at that exact source. The current proposed installer pin binds
`ghcr.io/memeloop-online/memeloop-token-center-plugin-installer@sha256:47872d7dffc26a1c1908fed395d7387d09534bbdddf4186c5d74d3479fbdf783`
to the delegated owner's same-source paired receipt. The platform digest is
`sha256:8538f9eccf57ab11d47119eaf4bdd0a8633f090d9ccd5ed56819bdabdbc1d3b4`;
the actual raw-config bytes were independently downloaded and verified against
`sha256:ad9562bb6ef504e83473dafda35975ca1435d7676d66b8d7d0e573b98e712119`.
Their revision label is `3dbf18898357cd00ab33e8a79523d85dd6a08367` and Cosign
label is `v3.1.3-mtc.4`, matching the source build and the existing single pin
field consumed by the publisher and evidence validator. No digest is predicted
or substituted from source93. Release payloads, hashes and source remain
unchanged. The existing static pin contract's expected source/digest are updated
in lockstep. These checks do not imply that the next plugin OCI publication,
installation, deployment or subscription login has already passed. Merge remains
a separate parent decision because the existing master workflow publishes and
performs isolated positive/negative installer acceptance automatically.
Do not bypass signer trust, overwrite the source tag, or infer activation from
GitHub Release availability.
