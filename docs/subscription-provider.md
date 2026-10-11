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

The manifest adopts the generic declarations introduced by host PR580. The
generic start API uses `oauth_adapter.authorization_code.client_default` when
the global operator omits an explicit `client`. This does not widen operator
authorization. The public client settings, extracted from the existing core
Claude adapter, are:

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
permissions. No client secret is contributed. The manifest selects JSON token
exchange/refresh and inclusion of the host-verified state during exchange only.
The authorization URL carries `code=true`. Identity reads declare only the
`oauth-2025-04-20` beta; Messages requests declare that beta and version
`2023-06-01`. These are typed fixed protocol headers, not arbitrary header or
credential injection. The host owns state, PKCE, token storage and same-account
checks. A changed installed OAuth declaration requires reconnection under the
host's contract-drift policy.

## Host dependency and remaining activation gates

Requires the PR580 generic backend contract, reviewed at
`1f885ad45cd59957addadcc4de334b4fa82266a0`, in addition to core572's fragment and
identity hooks. Older hosts do not support these new declarations; do not treat
acceptance by an older parser as compatibility. Host mock tests passing is not
supplier authorization or entitlement verification.

| Owner / path | Remaining responsibility |
| --- | --- |
| Host backend / PR580 | Deploy the reviewed generic contract; its mocks cover JSON exchange/refresh, state, declared identity headers, contract-bound reauthorization and Responses-to-Messages headers. |
| Separate host UI owner / `web/src/types.ts`, `web/src/operator/authorizationCode.ts`, `AuthorizationCodeConnection.tsx`, `authorizationCodeCopy.ts`, original authorization-code e2e contracts | Add mode-driven `code#state` validation/copy, expose generic reauthorization from host capability, and represent the new declarations without provider-specific branching or browser persistence. |
| Plugin release owner / `.github/oci-release.json` | Replace the 1.0.1 pin only with separately authorized, actual reviewed release hashes; source candidate packaging is not release-pin readiness. |

This declaration adoption does not change the request-body privacy guest or host
UI. Do not enable this candidate for subscription accounts or retire the built-in
until host/UI compatibility and isolated installation acceptance cover
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
preparation fails closed when that manifest differs from source. Before publication,
a separately authorized release must replace that record with
reviewed 1.1.0 release bytes and checksums, after host acceptance. This PR creates
no release/tag and publishes no production or candidate OCI image.
