//! Quarantined wire-shim component. No subscription request may be rewritten.

wit_bindgen::generate!({
    world: "memeloop:token-center/wire-shim-plugin@0.3.0",
    path: "wit",
});

use exports::memeloop::token_center0_3_0::wire_shim_v1::FinalizeResult;
#[cfg(target_arch = "wasm32")]
use exports::memeloop::token_center0_3_0::wire_shim_v1::Guest;
use memeloop::token_center0_3_0::types::RequestContext;

#[cfg(target_arch = "wasm32")]
struct ClaudeCodeWire;

fn reject_request(
    _context: RequestContext,
    _request_json: String,
    _headers_json: String,
) -> Result<FinalizeResult, String> {
    Err("Claude subscription wire shim is quarantined".into())
}

#[cfg(target_arch = "wasm32")]
impl Guest for ClaudeCodeWire {
    fn finalize(
        _context: RequestContext,
        _request_json: String,
        _headers_json: String,
    ) -> Result<FinalizeResult, String> {
        reject_request(_context, _request_json, _headers_json)
    }
}

#[cfg(target_arch = "wasm32")]
export!(ClaudeCodeWire);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_requests_without_rewriting() {
        let context = RequestContext {
            tenant_id: "tenant".into(),
            principal_id: "principal".into(),
            key_id: "key".into(),
            protocol: "anthropic".into(),
            model: "model".into(),
            config_json: "{}".into(),
        };
        let result = reject_request(context, "{\"messages\":[]}".into(), "{}".into());
        assert!(result.is_err());
    }
}
