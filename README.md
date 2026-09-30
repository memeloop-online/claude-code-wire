# claude-code-wire — quarantined

This repository previously shipped a wire shim that made arbitrary clients look like Claude Code to a subscription OAuth upstream. That behavior is unsafe and is no longer supported. The current component rejects every request; it never rewrites a body or emits headers. Do not install an older release or the previously checked-in `plugin.wasm`.

The plugin is not an OAuth login client. OAuth authorization belongs to the gateway's separately reviewed flow and must be performed by the account owner. No subscription credential, local source file, or telemetry should be supplied to this repository or its CI.

CI runs native tests and builds the quarantined WASM component. Neither CI success nor a build authorizes installation, merging, or deployment. Keep production candidate `da263` frozen until isolated acceptance is complete.
