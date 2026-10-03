#!/usr/bin/env bash
set -euo pipefail

record="$GITHUB_WORKSPACE/.github/oci-release.json"
export INSTALLER_SOURCE INSTALLER_DIGEST INSTALLER_SOURCE_REVISION PLUGIN_SOURCE PLUGIN_DIGEST
INSTALLER_SOURCE=$(jq -er '.installer.repository' "$record")
INSTALLER_DIGEST=$(jq -er '.installer.digest' "$record")
INSTALLER_SOURCE_REVISION=$(jq -er '.installer.source_revision' "$record")
[[ "$INSTALLER_SOURCE" == ghcr.io/memeloop-online/memeloop-token-center-plugin-installer ]]
[[ "$INSTALLER_DIGEST" =~ ^sha256:[a-f0-9]{64}$ ]]
[[ "$INSTALLER_SOURCE_REVISION" =~ ^[a-f0-9]{40}$ ]]
PLUGIN_SOURCE=ghcr.io/memeloop-online/claude-code-wire
if [[ "$GITHUB_REF" != refs/heads/master ]]; then
  PLUGIN_SOURCE+=-candidate
fi
evidence="$RUNNER_TEMP/plugin-evidence"
mkdir -p "$evidence" "$RUNNER_TEMP/plugin-tools"
docker pull "$INSTALLER_SOURCE@$INSTALLER_DIGEST"
actual_revision=$(docker image inspect "$INSTALLER_SOURCE@$INSTALLER_DIGEST" --format '{{index .Config.Labels "org.opencontainers.image.revision"}}')
[[ "$actual_revision" == "$INSTALLER_SOURCE_REVISION" ]]
container=$(docker create "$INSTALLER_SOURCE@$INSTALLER_DIGEST")
trap 'docker rm "$container" >/dev/null; sudo rm -rf "$RUNNER_TEMP/plugin-registry"' EXIT
docker cp "$container:/usr/local/bin/cosign" "$RUNNER_TEMP/plugin-tools/cosign"
chmod 0555 "$RUNNER_TEMP/plugin-tools/cosign"
export PATH="$RUNNER_TEMP/plugin-tools:$PATH"
cosign version --json | tee "$evidence/cosign-version.json" | jq -e '.gitVersion == "v3.1.3-mtc.3"'
docker run --rm --entrypoint /usr/local/bin/install-plugin-oci \
  "$INSTALLER_SOURCE@$INSTALLER_DIGEST" --mtc-cosign-runtime-check

cd "$RUNNER_TEMP/plugin-package"
for name in plugin.json plugin.wasm; do
  digest=$(jq -er --arg name "$name" '.files[$name]' "$record")
  printf '%s  %s\n' "$digest" "$name" | sha256sum --check --strict
done
source_revision=$(jq -er '.source_revision' "$record")
version=$(jq -er '.version' "$record")
oras push "$PLUGIN_SOURCE:sha-$GITHUB_SHA" \
  --artifact-type application/vnd.memeloop.token-center.plugin.v1 \
  --config config.json:application/vnd.memeloop.token-center.plugin.config.v1+json \
  --annotation "org.opencontainers.image.source=https://github.com/$GITHUB_REPOSITORY" \
  --annotation "org.opencontainers.image.revision=$source_revision" \
  --annotation "org.opencontainers.image.version=$version" \
  plugin.json:application/vnd.memeloop.token-center.plugin.manifest.v1+json \
  plugin.wasm:application/vnd.wasm.content.layer.v1+wasm \
  --format json > "$evidence/plugin-push.json"
PLUGIN_DIGEST=$(jq -er '.digest' "$evidence/plugin-push.json")
[[ "$PLUGIN_DIGEST" =~ ^sha256:[a-f0-9]{64}$ ]]
cosign sign --yes "$PLUGIN_SOURCE@$PLUGIN_DIGEST"
cosign verify --certificate-identity "$SIGNING_IDENTITY" \
  --certificate-oidc-issuer "$SIGNING_ISSUER" \
  "$PLUGIN_SOURCE@$PLUGIN_DIGEST" > "$evidence/plugin-signature-verification.json"
oras manifest fetch "$PLUGIN_SOURCE@$PLUGIN_DIGEST" --output "$evidence/plugin-oci-manifest.json"

mkdir -p "$RUNNER_TEMP/plugin-install" "$RUNNER_TEMP/plugin-rejected" "$RUNNER_TEMP/plugin-registry"
umask 077
printf '%s' "$REGISTRY_USERNAME" > "$RUNNER_TEMP/plugin-registry/username"
printf '%s' "$REGISTRY_PASSWORD" > "$RUNNER_TEMP/plugin-registry/password"
sudo chown -R 10001:10001 "$RUNNER_TEMP/plugin-registry"
sudo chown 10001:10001 "$RUNNER_TEMP/plugin-install" "$RUNNER_TEMP/plugin-rejected"
install_package() {
  docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges \
    --tmpfs /tmp:rw,nosuid,nodev,size=32m \
    --mount "type=bind,src=$1,dst=/plugins" \
    --mount "type=bind,src=$RUNNER_TEMP/plugin-registry,dst=/registry,readonly" \
    --entrypoint /usr/local/bin/install-plugin-oci "$INSTALLER_SOURCE@$INSTALLER_DIGEST" \
    "$PLUGIN_SOURCE@$PLUGIN_DIGEST" --plugin-dir /plugins --allowed-source "$PLUGIN_SOURCE" \
    --cosign-certificate-identity "$2" --cosign-certificate-oidc-issuer "$SIGNING_ISSUER" \
    --registry-username-file /registry/username --registry-password-file /registry/password
}
if install_package "$RUNNER_TEMP/plugin-rejected" \
  "https://github.com/$GITHUB_REPOSITORY/.github/workflows/not-the-signer.yml@refs/heads/master" \
  > "$evidence/rejected-installation.json" 2> "$evidence/rejected-installation.stderr"; then
  echo 'Installer accepted a wrong signing identity' >&2
  exit 1
fi
grep -q '"category":"signature"' "$evidence/rejected-installation.stderr"
[[ ! -e "$RUNNER_TEMP/plugin-rejected/claude-code-wire" ]]
install_package "$RUNNER_TEMP/plugin-install" "$SIGNING_IDENTITY" > "$evidence/plugin-installation.json"
cmp plugin.json "$RUNNER_TEMP/plugin-install/claude-code-wire/plugin.json"
cmp plugin.wasm "$RUNNER_TEMP/plugin-install/claude-code-wire/plugin.wasm"
cp "$RUNNER_TEMP/plugin-install/claude-code-wire/.mtc-oci-install.json" "$evidence/install-receipt.json"
cp plugin.json plugin.wasm config.json "$evidence/"
python3 "$GITHUB_WORKSPACE/scripts/oci/evidence.py"
if [[ "$GITHUB_REF" == refs/heads/master ]]; then
  oras tag "$PLUGIN_SOURCE@$PLUGIN_DIGEST" "v$version"
fi
cat "$evidence/install-plan.md" >> "$GITHUB_STEP_SUMMARY"
