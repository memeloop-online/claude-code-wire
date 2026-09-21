# Claude Code wire format (English)

[中文](protocol.zh-CN.md)

This document describes exactly what the plugin emits, cross-checked against
pi-black's src/claude-code-protocol.ts for Claude Code 2.1.258.

## Request body

### system array

The rewritten request always begins with two system blocks, in this order:

1. Billing block (a text block):

       x-anthropic-billing-header: cc_version=<VERSION>.<FP>; cc_entrypoint=<ENTRYPOINT>; cch=<CCH>;

   - VERSION: claude_code_version configuration (default 2.1.258).
   - FP: first 3 hex characters of SHA-256 over
     59cf53e54c78 + selected + VERSION, where selected is the first user
     message text's UTF-16 code units at indices 4, 7 and 20 (the character
     '0' fills missing indices; a lone surrogate from a split astral
     character encodes as U+FFFD, matching TextEncoder).
   - ENTRYPOINT: entrypoint configuration (default sdk-cli).
   - CCH: see below.
2. Agent SDK block (a text block):

       You are a Claude agent, built on Anthropic's Claude Agent SDK.

Before insertion, a stale billing + Agent SDK block pair at the head, or a
single legacy "You are Claude Code, Anthropic's official CLI for Claude."
block, is stripped. Any other system blocks are preserved, in order.

A non-array system field is rejected (fail-closed).

### cch checksum

Computed after the final serialization, on the exact bytes that will be
sent:

1. Parse the body, keep key insertion order.
2. normalized = deep copy; normalized.model = ""; delete normalized.max_tokens.
3. hash = XXH64(serialize(normalized), seed = 0x4d659218e32a3268).
4. cch = (hash AND 0xfffff) as 5 lowercase hex digits, zero-padded.
5. Replace the cch=00000 placeholder in the billing block with the value.

The host forwards the plugin output byte-for-byte, so a validator that
recomputes the checksum from the wire bytes (restoring the placeholder)
always matches.

### metadata.user_id (optional)

Only when both device_id and account_uuid are configured:

    metadata.user_id = "{\"device_id\":\"...\",\"account_uuid\":\"...\",\"session_id\":\"...\"}"

session_id is derived as the first 16 bytes of SHA-256(tenant_id ++ key_id)
shaped as a UUIDv4: stable per API key, distinct across keys.

## Headers

Set by the plugin (the host strips the client's own copies first):

- user-agent: claude-cli/<VERSION> (external, <ENTRYPOINT>)
- x-app: cli
- x-claude-code-session-id: derived session id
- x-client-request-id: fresh random UUID per request (stable across the
  host's upstream retries of the same request)
- x-stainless-lang / -package-version / -os / -arch / -runtime /
  -runtime-version / -timeout: from configuration
- x-stainless-retry-count: 0; rewritten by the host to the zero-based
  attempt index on retries

anthropic-version and anthropic-beta are handled by the host core (the
client's betas are preserved and merged with oauth-2025-04-20).

## Failure modes

The plugin errors (and the host rejects the request) when: the body is not
a JSON object; model or max_tokens is missing; the system field is not an
array; the configuration is invalid (unknown fields, malformed identity);
or enabled is false.
