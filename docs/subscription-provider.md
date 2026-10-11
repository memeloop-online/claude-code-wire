# Community subscription provider candidate

Version 1.1.0 adds `claude-code-subscription` alongside the existing
`anthropic-claude` wire-shim selector. The distinct ID permits installation
without replacing or removing the core built-in provider. This is the plugin
implementation PR2 in the split plan, not GitHub issue/PR number 2.

The contribution selects the host's generic Anthropic Messages transport and
`responses_via_anthropic_messages_v1` bridge for Codex Responses. It does not
execute Claude Code, connect a CLI back to MTC, or use an independently billed
Anthropic API key. Model availability and subscription entitlement are decided
by the supplier; capability declarations do not prove successful inference.

## Required host UI contract

The UI must read `oauth_adapter.flow_kind=authorization_code_pkce` and
`response_mode=fragment_manual`, open the returned authorization URL, and offer
a `code#state` paste field using the generic completion endpoint. It must retain
the pending session token, expiry, tenant/operator scope and idempotent completion
handling. Reauthorization must keep the existing account and compare the declared
`identity.url` response at `/account/uuid`; a different account must be rejected.

The generic start API currently accepts an explicit `client` object from a global
operator. The reviewed subscription client settings, extracted from the existing
core Claude adapter, are:

```json
{
  "client_id": "9d1c250a-e61b-44d9-88ed-5944d1962f5e",
  "redirect_uri": "https://platform.claude.com/oauth/code/callback",
  "scopes": [
    "org:create_api_key",
    "user:profile",
    "user:inference",
    "user:sessions:claude_code",
    "user:mcp_servers",
    "user:file_upload"
  ]
}
```

These scope names are the existing supplier contract, not plugin filesystem
permissions. No client secret is needed. Current `oauth-adapter-v1` has no field
for contributing fixed client settings: a generic, reviewed declarative default
is needed for ordinary account-owner UI authorization. Do not inject unsupported
fields into the signed manifest or silently use another provider's client.

## Known blockers at core572

Source baseline: `2746ed35b80233fe50a12b772b12217dabc08019`.
The fragment and identity hooks alone do not establish end-to-end compatibility:

| Host path | Exact gap requiring host-owner resolution |
| --- | --- |
| `src/oauth/authorization_code.rs`, completion and `prepare_token_request` | Generic exchange uses form encoding and omits state from the token body. The existing Claude adapter sends JSON including state; refresh also uses JSON. A reviewed generic encoding/state contract and HTTP mock coverage are needed. |
| `src/oauth/authorization_code.rs`, `bind_declared_identity` | Generic identity GET has no declared extra headers. The existing Claude profile read sends `anthropic-beta: oauth-2025-04-20`. |
| `src/api/upstreams/oauth_authorization_code.rs`, start reauthorization guard | Still rejects drivers other than Antigravity despite generic identity support. |
| `src/api/proxy/routing/http.rs`, Anthropic header application | OAuth beta is added only for the built-in `anthropic-claude` driver, not the new contribution ID. Codex Responses callers cannot be expected to supply this supplier header. |
| Generic start API and provider UI | Fixed client defaults and response-mode-driven paste/reauthorization UI must be available without Claude-specific branching. |

The manifest uses only existing contribution fields. It cannot fix these gaps
inside a request-body privacy guest. Do not enable this candidate for subscription
accounts or retire the built-in until host HTTP-level mock acceptance covers
authorization, profile identity, refresh, same-account reauthorization, Messages
and Codex Responses, including errors, state mismatch and duplicate completion.
No real OAuth or paid inference was performed for this change.

## Privacy and validation boundaries

The WASM guest has no network/filesystem/logging/storage capabilities and keeps
the existing `metadata.user_id` removal behavior. It does not read the user's
disk or add telemetry. The host performs declared OAuth/profile requests and
sends model, prompts, messages, tool definitions/results and other inference
content to Anthropic. This is not a zero-leakage guarantee: the body shim does not
sanitize arbitrary secrets or all forwarded headers, or stop other client traffic.

Existing GitHub Actions performs unit tests and component builds. Added tests
check coexistence, manifest version agreement, OAuth-only credentials, declared
identity selection and preservation of tool calls during privacy processing.
These are plugin contract tests, not a supplier or host integration success claim.
Local tests, builds and dependency installation are intentionally not run.

PR OCI packaging builds candidate source bytes without publication. The existing
keyless signing workflow, signer identities and fingerprint trust remain intact.
`.github/oci-release.json` continues to pin the reviewed 1.0.1 release; production
preparation fails closed when that manifest differs from source. Before merging
for publication, a separately authorized release must replace that record with
reviewed 1.1.0 release bytes and checksums, after host acceptance. This PR creates
no release/tag and publishes no production or candidate OCI image.
