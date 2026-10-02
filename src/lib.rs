wit_bindgen::generate!({
    world: "memeloop:token-center/wire-shim-plugin@0.3.0",
    path: "wit",
});

use exports::memeloop::token_center0_3_0::wire_shim_v1::FinalizeResult;
#[cfg(target_arch = "wasm32")]
use exports::memeloop::token_center0_3_0::wire_shim_v1::Guest;
use memeloop::token_center0_3_0::types::RequestContext;
use serde::Deserialize;
use serde_json::Value;

#[cfg(target_arch = "wasm32")]
struct ClaudeCodeWire;

#[derive(Deserialize)]
#[serde(default)]
struct Configuration {
    enabled: bool,
    remove_user_id: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            enabled: true,
            remove_user_id: true,
        }
    }
}

fn finalize_request(
    context: RequestContext,
    request_json: String,
    _headers_json: String,
) -> Result<FinalizeResult, String> {
    let configuration: Configuration = if context.config_json.trim().is_empty() {
        Configuration::default()
    } else {
        serde_json::from_str(&context.config_json)
            .map_err(|_| "Invalid Claude request privacy configuration".to_owned())?
    };
    if !configuration.enabled || !configuration.remove_user_id {
        return Ok(FinalizeResult {
            request_json,
            set_headers: Vec::new(),
        });
    }
    let mut body: Value = serde_json::from_str(&request_json)
        .map_err(|_| "Claude request body must be valid JSON".to_owned())?;
    let changed = body
        .get_mut("metadata")
        .and_then(Value::as_object_mut)
        .and_then(|metadata| metadata.remove("user_id"))
        .is_some();
    let request_json = if changed {
        serde_json::to_string(&body)
            .map_err(|_| "Could not serialize Claude request body".to_owned())?
    } else {
        request_json
    };
    Ok(FinalizeResult {
        request_json,
        set_headers: Vec::new(),
    })
}

#[cfg(target_arch = "wasm32")]
impl Guest for ClaudeCodeWire {
    fn finalize(
        context: RequestContext,
        request_json: String,
        headers_json: String,
    ) -> Result<FinalizeResult, String> {
        finalize_request(context, request_json, headers_json)
    }
}

#[cfg(target_arch = "wasm32")]
export!(ClaudeCodeWire);

#[cfg(test)]
mod tests {
    use super::*;

    fn context(config: &str) -> RequestContext {
        RequestContext {
            tenant_id: "tenant".into(),
            principal_id: "principal".into(),
            key_id: "key".into(),
            protocol: "anthropic".into(),
            model: "model".into(),
            config_json: config.into(),
        }
    }

    #[test]
    fn removes_tracking_identity_without_changing_inference_content() {
        let original = serde_json::json!({
            "model": "claude-model",
            "messages": [{"role": "user", "content": "Keep user_id in this text."}],
            "system": [{"type": "text", "text": "Original instructions"}],
            "tools": [{"name": "user_id", "input_schema": {"type": "object"}}],
            "metadata": {"user_id": "private-device-and-account", "future_field": "preserve"},
            "future_option": true
        });
        let result = finalize_request(context("{}"), original.to_string(), "{}".into())
            .expect("valid request");
        let mut expected = original;
        expected["metadata"].as_object_mut().unwrap().remove("user_id");
        assert_eq!(serde_json::from_str::<Value>(&result.request_json).unwrap(), expected);
        assert!(result.set_headers.is_empty());
    }

    #[test]
    fn preserves_exact_bytes_when_nothing_needs_removing() {
        for request in ["{ \"messages\": [] }", "{\"metadata\":{\"future\":true}}"] {
            let result = finalize_request(context("{}"), request.into(), "{}".into())
                .expect("valid request");
            assert_eq!(result.request_json, request);
            assert!(result.set_headers.is_empty());
        }
    }

    #[test]
    fn explicit_opt_out_preserves_existing_client_identity() {
        let request = "{ \"metadata\": {\"user_id\": \"client-owned\"} }";
        for configuration in ["{\"enabled\":false}", "{\"remove_user_id\":false}"] {
            let result = finalize_request(context(configuration), request.into(), "{}".into())
                .expect("valid configuration");
            assert_eq!(result.request_json, request);
            assert!(result.set_headers.is_empty());
        }
    }

    #[test]
    fn legacy_settings_do_not_inject_identity_or_reject_requests() {
        let result = finalize_request(
            context(r#"{"device_id":"legacy-device","account_uuid":"legacy-account","claude_code_version":"legacy"}"#),
            "{\"messages\":[]}".into(),
            "{}".into(),
        )
        .expect("legacy configuration remains usable");
        assert_eq!(result.request_json, "{\"messages\":[]}");
        assert!(result.set_headers.is_empty());
    }

    #[test]
    fn malformed_configuration_does_not_echo_private_input() {
        let error = finalize_request(context("private-invalid-json"), "{}".into(), "{}".into())
            .err()
            .expect("invalid configuration");
        assert_eq!(error, "Invalid Claude request privacy configuration");
    }
}
