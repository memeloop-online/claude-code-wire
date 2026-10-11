# v1.1.0 release-byte pairing

This record binds actual downloaded GitHub Release assets, not predicted hashes
or a source candidate alone. It does not authorize OCI publication, installation,
TEST/PROD activation, OAuth or inference. Subscription login is not verified.

## Source and publisher

- Source and lightweight tag `v1.1.0`: `ab7df5591b8afbb73d8b2a164c90df6e7593ef78`.
- Cargo and manifest versions: `1.1.0`.
- Existing tag-triggered [CI run 38178017688](https://github.com/memeloop-online/claude-code-wire/actions/runs/38178017688): test and release jobs both succeeded.
- [Actual Release](https://github.com/memeloop-online/claude-code-wire/releases/tag/v1.1.0) published at `2026-10-11T21:51:17Z`.
- Workflows, permissions, signature gates and installer pin are unchanged.

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

The pairing PR changes only the proposed release version/source/manifest hash;
the old production pin remains unchanged until a separately approved merge.
Merging this PR onto master triggers the existing automatic OCI signing and
installer workflow and therefore must not be treated as a documentation-only
action. No such merge or OCI publication was performed for this preparation.

The retained installer pin is provenance for the existing packaging workflow,
not evidence that its old host revision supports the new provider declarations.
Host/UI compatibility and subscription entitlement require separate acceptance.
Do not bypass signer trust, overwrite the source tag, or infer activation from
GitHub Release availability.
