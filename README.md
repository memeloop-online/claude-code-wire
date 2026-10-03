# Claude request privacy plugin

[中文说明](README.zh-CN.md)

This MTC plugin removes the optional `metadata.user_id` tracking field from outbound Anthropic Messages requests. It preserves model selection, system instructions, messages, tools, and unknown protocol fields. Requests without the field retain their exact original bytes. It does not add client fingerprints, billing markers, session identifiers, or headers.

The plugin declares no capabilities and performs no network, filesystem, logging, or storage calls. It receives only the request already supplied by the gateway. It cannot stop telemetry sent directly by a client through a different connection, or remove secrets from the user's prompts.

## Configuration

Both options default to `true`:

| Option | Behavior |
| --- | --- |
| `enabled` | Enable request privacy processing. Disable for exact pass-through. |
| `remove_user_id` | Remove `metadata.user_id`. Disable when the client explicitly requires that field. |

Legacy fingerprint configuration is ignored; it never causes identity injection. The component no longer rejects every request. Malformed JSON configuration or request bodies return a generic error without echoing input.

## Login and installation

OAuth login belongs to MTC's account connection flow and must be approved by the account owner. The plugin neither receives nor manages subscription credentials. Removing metadata does not establish account entitlement or guarantee that an upstream accepts a request.

Use `plugin.wasm` and `plugin.json` from the same reviewed CI artifact or release. Validate against the target MTC version in an isolated environment before enabling it for an account. Do not combine old fingerprint-rewriting builds or manifests with this component.

GitHub Actions runs privacy regression tests and builds the WASM component. Tests cover tracking removal, exact pass-through, preservation of inference content, and legacy configuration compatibility. Tagged releases publish the component, manifest, and checksums together.

For signed OCI packaging, exact signer trust, digest evidence, and an isolated MTC installer rehearsal, see [OCI release and installation](docs/oci-release.md). Official packages are published from master; candidate packages use a separate repository and branch identity.
