# claude-code-wire

[中文文档](README.zh-CN.md)

A [memeloop-token-center](https://github.com/memeloop-online/memeloop-token-center)
wire-shim plugin that rewrites Anthropic Messages requests destined for
**anthropic-claude (Claude subscription OAuth) upstreams** into the exact wire
format of the official Claude Code CLI — so the upstream sees byte-identical
traffic no matter which client actually sent the request.

The transformation is a bit-for-bit port of the
[pi-black](https://github.com/paoloanzn/pi-black) reference implementation:

- **system block surgery** — strips any stale billing / Agent SDK / legacy
  block, then prepends a fresh billing block (with the cc_version
  fingerprint and cch checksum) and the Agent SDK block;
- **cch checksum** — seeded XXH64 over the final serialized body (with
  model blanked and max_tokens removed), low 20 bits as 5 hex digits.
  The host forwards the plugin output byte-for-byte and never re-serializes,
  so the checksum stays self-consistent on the wire;
- **canonical headers** — user-agent, x-app, x-claude-code-session-id,
  per-request x-client-request-id, and the x-stainless-* SDK fingerprint
  family. The host strips the client's own fingerprint headers before
  applying these, so a non-Claude-Code SDK (e.g. a Python client) cannot
  leak through;
- **optional identity metadata** — metadata.user_id carrying
  device_id / account_uuid / a stable derived session_id.

Fail-closed: if the request cannot be rewritten safely, or the plugin is
disabled, the host **rejects** the request. An unrewritten request is never
forwarded to a subscription OAuth upstream.

## Build

    rustup target add wasm32-unknown-unknown
    ./build.sh        # produces plugin.wasm
    cargo test        # native unit tests (XXH64 vectors, fingerprint, cch)

To refresh the vendored WIT definitions from a local checkout of the host:

    MTC_REPO=/path/to/memeloop-token-center ./build.sh

## Install

Ship plugin.json + plugin.wasm (+ this README) as one plugin package
directory and mount it into the host's plugin directory
(MTC_PLUGIN_DIR, or the OCI installer flow described in the host's
plugins/README.md). The plugin requires the host's WIT 0.3.0 wire-shim ABI.

## Configuration

All settings have safe defaults matching Claude Code 2.1.258 and can be set
globally or per tenant:

| key | default | notes |
| --- | --- | --- |
| enabled | true | Disabling rejects requests (fail-closed), it does not pass them through |
| claude_code_version | 2.1.258 | Embedded in user-agent and the cc_version fingerprint |
| entrypoint | sdk-cli | cc_entrypoint billing value |
| device_id | — | 64 lowercase hex, from the account's ~/.claude.json userID; must be set together with account_uuid |
| account_uuid | — | The Claude account UUID |
| stainless_* | js / 0.60.0 / MacOS / arm64 / node / v22.14.0 / 600 | x-stainless-* fingerprint values; keep them stable per account |

Configuration changes apply at request time (bounded 5 s cache), so version
bumps are hot-swappable without restarting the gateway.

## Client-side telemetry (outside the gateway's reach)

Claude Code's Statsig/Sentry telemetry connects **directly** from the client
and never passes through the gateway. If the client is Claude Code itself,
set:

    export DISABLE_TELEMETRY=1
    export DISABLE_ERROR_REPORTING=1
    export CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1

or block the telemetry domains at DNS level. Non-Claude-Code clients that
call the gateway's Anthropic endpoint directly do not have this problem.

## Egress IP

A perfect request fingerprint does not help if the source IP is in an
unsupported region. Configure a SOCKS5 egress proxy in a supported region
for the anthropic-claude upstream account (proxy_url at login;
socks5h + private-IP literal).

## Risk notice

This plugin makes subscription traffic indistinguishable from the official
client on the wire. Anthropic's risk control may change at any time (new
version numbers, new fingerprint dimensions); keep claude_code_version
in sync with the official Claude Code releases. Evaluate the applicable
terms of service yourself before use.

See [docs/protocol.md](docs/protocol.md) for the exact wire-format details.
