#!/usr/bin/env bash
set -euo pipefail

record="$GITHUB_WORKSPACE/.github/oci-release.json"
version=$(jq -er '.version' "$record")
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
mkdir -p "$RUNNER_TEMP/plugin-package"
cd "$RUNNER_TEMP/plugin-package"
for name in plugin.json plugin.wasm; do
  digest=$(jq -er --arg name "$name" '.files[$name]' "$record")
  [[ "$digest" =~ ^[a-f0-9]{64}$ ]]
  curl --fail --location --retry 3 --max-time 120 \
    "https://github.com/memeloop-online/claude-code-wire/releases/download/v$version/$name" \
    --output "$name"
  printf '%s  %s\n' "$digest" "$name" | sha256sum --check --strict
done
jq -e --arg version "$version" \
  '.id == "claude-code-wire" and .version == $version and .wasm == "plugin.wasm" and .wit_version == "0.3.0" and .capabilities == []' plugin.json
cmp plugin.json "$GITHUB_WORKSPACE/plugin.json"
printf '{"format_version":1}\n' > config.json
