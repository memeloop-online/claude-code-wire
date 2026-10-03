import hashlib
import json
import os
from pathlib import Path

root = Path(os.environ["RUNNER_TEMP"]) / "plugin-evidence"
record = json.loads((Path(os.environ["GITHUB_WORKSPACE"]) / ".github/oci-release.json").read_text())


def read_json(name):
    return json.loads((root / name).read_text())


def digest(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


expected = os.environ["PLUGIN_DIGEST"]
source = os.environ["PLUGIN_SOURCE"]
manifest_bytes = (root / "plugin-oci-manifest.json").read_bytes()
assert digest(manifest_bytes) == expected
manifest = json.loads(manifest_bytes)
assert manifest["schemaVersion"] == 2
assert manifest["artifactType"] == "application/vnd.memeloop.token-center.plugin.v1"
assert manifest["config"]["mediaType"] == "application/vnd.memeloop.token-center.plugin.config.v1+json"
config_bytes = (root / "config.json").read_bytes()
assert manifest["config"]["digest"] == digest(config_bytes)
assert manifest["config"]["size"] == len(config_bytes)
assert read_json("config.json") == {"format_version": 1}
assert len(manifest["layers"]) == 2
files = []
for name, media_type in {
    "plugin.json": "application/vnd.memeloop.token-center.plugin.manifest.v1+json",
    "plugin.wasm": "application/vnd.wasm.content.layer.v1+wasm",
}.items():
    data = (root / name).read_bytes()
    layer = next(item for item in manifest["layers"] if item["annotations"]["org.opencontainers.image.title"] == name)
    assert layer["digest"] == digest(data) == "sha256:" + record["files"][name]
    assert layer["size"] == len(data)
    assert layer["mediaType"] == media_type
    files.append({"name": name, "digest": digest(data), "size": len(data)})
installed = read_json("plugin-installation.json")
assert installed["id"] == "claude-code-wire"
assert installed["version"] == record["version"]
for item in (installed, read_json("install-receipt.json")):
    assert item["digest"] == expected
    assert item["source"] == source
assert read_json("install-receipt.json")["signature_policy"] == "cosign-keyless"
verification = read_json("plugin-signature-verification.json")
assert isinstance(verification, list) and verification
assert any(item["critical"]["image"]["docker-manifest-digest"] == expected for item in verification)
reference = f"{source}@{expected}"
installer = f"{os.environ['INSTALLER_SOURCE']}@{os.environ['INSTALLER_DIGEST']}"
identity = os.environ["SIGNING_IDENTITY"]
issuer = os.environ["SIGNING_ISSUER"]
evidence = {
    "format_version": 1,
    "reference": reference,
    "version": record["version"],
    "release_source_revision": record["source_revision"],
    "packaging_revision": os.environ["GITHUB_SHA"],
    "workflow_run": f"https://github.com/{os.environ['GITHUB_REPOSITORY']}/actions/runs/{os.environ['GITHUB_RUN_ID']}",
    "official": os.environ["GITHUB_REF"] == "refs/heads/master",
    "signature": {"policy": "cosign-keyless", "identity": identity, "issuer": issuer},
    "installer_reference": installer,
    "installer_source_revision": os.environ["INSTALLER_SOURCE_REVISION"],
    "files": files,
    "installation_verified": True,
    "wrong_identity_rejected": True,
    "registry_access": "workflow-token; anonymous access not established",
}
(root / "plugin-release.json").write_text(json.dumps(evidence, indent=2) + "\n")
(root / "install-plan.md").write_text(f"""### Signed OCI installation rehearsal

- Package: `{reference}`
- Official master publication: `{evidence['official']}`
- Installer: `{installer}`
- Exact certificate identity: `{identity}`
- OIDC issuer: `{issuer}`
- Real installer accepted the package and wrote a `cosign-keyless` receipt; installed bytes match v{record['version']} SHA-256 pins.
- The same installer rejected a wrong certificate identity with category `signature` and published no plugin directory.
- Evidence: `oci-evidence-{os.environ['GITHUB_SHA']}` on [this run]({evidence['workflow_run']}).

For an isolated test, use the installer above with its bundled Cosign, an empty writable plugin directory, and:

```sh
install-plugin-oci '{reference}' \\
  --plugin-dir /isolated/plugins --allowed-source '{source}' \\
  --cosign-certificate-identity '{identity}' \\
  --cosign-certificate-oidc-issuer '{issuer}'
```

For private registry access, add `--registry-username-file` and `--registry-password-file` pointing to existing authorized, read-only credential files. Anonymous access has not been established. Compare both installed files against `plugin-release.json` and check `.mtc-oci-install.json` for the same source/digest and `cosign-keyless`. Installation does not activate the plugin or create an account. Production account login and activation remain separate owner-approved work.
""")
