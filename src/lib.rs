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
    fn subscription_provider_coexists_with_builtin_privacy_processing() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let providers = manifest["contributions"]["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 1);
        let provider = &providers[0];
        assert_ne!(provider["id"], "anthropic-claude");
        let drivers = manifest["contributions"]["wire_shim"]["drivers"]
            .as_array()
            .unwrap();
        assert!(drivers.contains(&provider["id"]));
        assert!(drivers.contains(&serde_json::json!("anthropic-claude")));
        assert_eq!(manifest["capabilities"], serde_json::json!([]));
        assert_eq!(manifest["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(
            provider["credential_schema"]["properties"]["type"]["const"],
            "oauth"
        );
        assert!(
            provider["credential_schema"]["properties"]
                .get("value")
                .is_none()
        );
    }

    #[test]
    fn installer_pin_and_publisher_share_exact_cosign_contract() {
        let record: Value =
            serde_json::from_str(include_str!("../.github/oci-release.json")).unwrap();
        let installer = &record["installer"];
        assert_eq!(
            installer["source_revision"],
            "93bcc9d1d51133055ba48cfab2ae606efc2b4a97"
        );
        assert_eq!(
            installer["digest"],
            "sha256:0e9bec4cd0393e56b2ba382a48c5b1744339be2a41a828e0ab2ff7a185a61d52"
        );
        assert_eq!(installer["cosign_version"], "v3.1.3-mtc.4");
        let publisher = include_str!("../scripts/oci/publish.sh");
        let evidence = include_str!("../scripts/oci/evidence.py");
        assert!(
            publisher.contains("INSTALLER_COSIGN_VERSION=$(jq -er '.installer.cosign_version'")
        );
        assert!(
            publisher.contains(
                "--arg expected \"$INSTALLER_COSIGN_VERSION\" '.gitVersion == $expected'"
            )
        );
        assert!(evidence.contains("read_json(\"cosign-version.json\")[\"gitVersion\"] == record[\"installer\"][\"cosign_version\"]"));
        for consumer in [
            publisher,
            evidence,
            include_str!("../scripts/oci/prepare.sh"),
            include_str!("../.github/workflows/publish-oci.yml"),
        ] {
            assert!(!consumer.contains("v3.1.3-mtc."));
            assert!(!consumer.contains("2cd0580f6546044dcb40f06ecf53b4271bba5bde"));
            assert!(
                !consumer
                    .contains("df6908c6e736ec150e64882bba02ac474637fe9fe9e31cc6b9452f7e6c91d6c0")
            );
        }
        for gate in [
            "[[ \"$actual_revision\" == \"$INSTALLER_SOURCE_REVISION\" ]]",
            "--mtc-cosign-runtime-check",
            "--certificate-identity \"$SIGNING_IDENTITY\"",
            "--certificate-oidc-issuer \"$SIGNING_ISSUER\"",
            "not-the-signer.yml@refs/heads/master",
            "grep -q '\"category\":\"signature\"'",
        ] {
            assert!(publisher.contains(gate));
        }
        assert!(
            publisher.find("'.gitVersion == $expected'").unwrap()
                < publisher.find("oras push").unwrap()
        );
    }

    #[test]
    fn optional_refresh_token_schema_matches_host_null_serialization() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let schema = &manifest["contributions"]["providers"][0]["credential_schema"];
        let types = schema["properties"]["refresh_token"]["type"]
            .as_array()
            .unwrap();
        let accepts = |value: &Value| {
            types.iter().any(|kind| match kind.as_str() {
                Some("string") => value.is_string(),
                Some("null") => value.is_null(),
                _ => false,
            })
        };
        for valid in [serde_json::json!("fixture-refresh"), Value::Null] {
            assert!(accepts(&valid));
        }
        for invalid in [
            serde_json::json!(false),
            serde_json::json!(42),
            serde_json::json!({}),
            serde_json::json!([]),
        ] {
            assert!(!accepts(&invalid));
        }
        let required = schema["required"].as_array().unwrap();
        assert!(!required.contains(&serde_json::json!("refresh_token")));
        assert!(required.contains(&serde_json::json!("access_token")));
        assert!(required.contains(&serde_json::json!("expires_at")));
        assert_eq!(schema["properties"]["access_token"]["type"], "string");
        assert_eq!(schema["properties"]["access_token"]["minLength"], 1);
        assert_eq!(schema["properties"]["expires_at"]["type"], "integer");
    }

    #[test]
    fn subscription_authorization_is_host_owned_and_identity_bound() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let adapter = &manifest["contributions"]["providers"][0]["oauth_adapter"];
        assert_eq!(adapter["flow_kind"], "authorization_code_pkce");
        assert_eq!(adapter["response_mode"], "fragment_manual");
        assert_eq!(
            adapter["identity"]["url"],
            "https://api.anthropic.com/api/oauth/profile"
        );
        let profile = serde_json::json!({
            "account": {"uuid": "fixture-account"},
            "organization": {"uuid": "wrong-organization"}
        });
        assert_eq!(
            profile.pointer(adapter["identity"]["id_pointer"].as_str().unwrap()),
            Some(&serde_json::json!("fixture-account"))
        );
        assert_eq!(adapter["poll_url"], adapter["refresh_url"]);
        assert!(adapter.get("client_secret").is_none());
    }

    #[test]
    fn subscription_declarations_use_closed_generic_oauth_types() {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct PublicClient {
            client_id: String,
            redirect_uri: String,
            scopes: Vec<String>,
        }
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum TokenEncoding {
            Form,
            Json,
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct AuthorizationCode {
            token_encoding: TokenEncoding,
            include_state: bool,
            client_default: PublicClient,
        }
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let adapter = &manifest["contributions"]["providers"][0]["oauth_adapter"];
        assert_eq!(
            adapter["login_url"],
            "https://claude.com/cai/oauth/authorize?code=true"
        );
        let declaration = &adapter["authorization_code"];
        let parsed: AuthorizationCode = serde_json::from_value(declaration.clone()).unwrap();
        assert!(matches!(parsed.token_encoding, TokenEncoding::Json));
        assert!(parsed.include_state);
        assert_eq!(
            parsed.client_default.client_id,
            "9d1c250a-e61b-44d9-88ed-5944d1962f5e"
        );
        assert_eq!(
            parsed.client_default.redirect_uri,
            "https://platform.claude.com/oauth/code/callback"
        );
        assert_eq!(
            parsed.client_default.scopes,
            [
                "org:create_api_key",
                "user:profile",
                "user:inference",
                "user:sessions:claude_code",
                "user:mcp_servers",
                "user:file_upload"
            ]
        );
        for (field, invalid) in [
            ("token_encoding", serde_json::json!("xml")),
            ("token_encoding", Value::Null),
            ("include_state", serde_json::json!("true")),
            ("client_default", Value::Null),
        ] {
            let mut changed = declaration.clone();
            changed[field] = invalid;
            assert!(serde_json::from_value::<AuthorizationCode>(changed).is_err());
        }
        for field in ["client_secret", "authorization", "headers"] {
            let mut changed = declaration.clone();
            changed["client_default"][field] = serde_json::json!("forbidden");
            assert!(serde_json::from_value::<AuthorizationCode>(changed).is_err());
        }
    }

    #[test]
    fn subscription_fixed_headers_exclude_credential_injection() {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct FixedHeaders {
            anthropic_version: Option<String>,
            anthropic_beta: Vec<String>,
        }
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let provider = &manifest["contributions"]["providers"][0];
        for (declaration, version) in [
            (&provider["oauth_adapter"]["identity"]["headers"], None),
            (
                &provider["request_compatibility"]["anthropic_messages_headers"],
                Some("2023-06-01"),
            ),
        ] {
            let parsed: FixedHeaders = serde_json::from_value(declaration.clone()).unwrap();
            assert_eq!(parsed.anthropic_version.as_deref(), version);
            assert_eq!(parsed.anthropic_beta, ["oauth-2025-04-20"]);
            for name in ["authorization", "cookie", "host", "x-api-key"] {
                let mut changed = declaration.clone();
                changed[name] = serde_json::json!("forbidden");
                assert!(serde_json::from_value::<FixedHeaders>(changed).is_err());
            }
            for invalid in [
                Value::Null,
                serde_json::json!("oauth-2025-04-20"),
                serde_json::json!([false]),
            ] {
                let mut changed = declaration.clone();
                changed["anthropic_beta"] = invalid;
                assert!(serde_json::from_value::<FixedHeaders>(changed).is_err());
            }
        }
    }

    #[test]
    fn responses_translation_retains_privacy_and_inference_content() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let provider = &manifest["contributions"]["providers"][0];
        assert_eq!(
            provider["request_compatibility"]["responses_via_anthropic_messages_v1"],
            true
        );
        assert_eq!(provider["request_compatibility"]["third_party"], true);
        assert_eq!(
            provider["request_compatibility"]["codex_multi_agent_v2"],
            true
        );
        let request = serde_json::json!({
            "model": "fixture-claude",
            "messages": [{"role": "assistant", "content": [
                {"type": "tool_use", "id": "call-fixture", "name": "shell", "input": {"command": "pwd"}}
            ]}],
            "metadata": {"user_id": "fixture-identity"},
            "stream": true
        });
        let result = finalize_request(context("{}"), request.to_string(), "{}".into()).unwrap();
        let sanitized: Value = serde_json::from_str(&result.request_json).unwrap();
        assert!(sanitized["metadata"].get("user_id").is_none());
        assert_eq!(sanitized["messages"], request["messages"]);
        assert_eq!(sanitized["model"], request["model"]);
        assert_eq!(sanitized["stream"], request["stream"]);
        assert!(result.set_headers.is_empty());
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
        expected["metadata"]
            .as_object_mut()
            .unwrap()
            .remove("user_id");
        assert_eq!(
            serde_json::from_str::<Value>(&result.request_json).unwrap(),
            expected
        );
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
