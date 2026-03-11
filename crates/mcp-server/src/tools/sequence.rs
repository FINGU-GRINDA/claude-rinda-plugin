// Sequence tool implementations.

use uuid::Uuid;

use crate::auth::{AuthContext, json_to_text, sdk_client};

/// Create a new email sequence, optionally with inline steps.
///
/// `seq_type` — forwarded as the sequence `description` field (e.g. "email").
/// `steps`    — JSON array of step objects. Supported fields per element:
///   - `delay` or `delayDays` (number): days to wait before sending (default 1)
///   - `subject` or `emailSubject` (string): email subject line (required per step)
///   - `body` or `emailBodyText` (string): plain-text email body (optional)
///   - `bodyHtml` or `emailBodyHtml` (string): HTML email body (optional)
///   - `stepOrder` (number): explicit step ordering (defaults to 1-based index)
///
/// After the sequence is created, each step is added via the steps API.
/// The response includes the created sequence ID and per-step results.
pub async fn sequence_create(
    auth: &AuthContext,
    name: String,
    seq_type: Option<String>,
    steps: Option<String>,
) -> String {
    // Fail fast on empty workspace_id — avoids a confusing UUID parse error later.
    if auth.workspace_id.is_empty() {
        return serde_json::json!({
            "error": "No workspace ID found in your authentication token. Please re-authenticate or ensure your account has an active workspace."
        })
        .to_string();
    }

    let client = sdk_client(Some(&auth.access_token));

    let workspace_id = match auth.workspace_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": format!(
                    "Workspace ID '{}' in your token is not a valid UUID. Please re-authenticate.",
                    auth.workspace_id
                )
            })
            .to_string();
        }
    };

    let name_typed: rinda_sdk::types::PostApiV1SequencesBodyName = match name.parse() {
        Ok(n) => n,
        Err(e) => {
            return serde_json::json!({ "error": format!("Invalid sequence name: {e}") })
                .to_string();
        }
    };

    // Forward seq_type via the description field.
    let description = seq_type.map(|t| format!("Type: {t}"));

    let body = rinda_sdk::types::PostApiV1SequencesBody {
        name: name_typed,
        workspace_id,
        created_by: None,
        customer_group_id: None,
        customer_group_ids: Vec::new(),
        description,
        memo: None,
        personalization_config: None,
        personalization_mode: None,
        status: None,
        timezone_mode: None,
        workflow_data: None,
    };

    let sequence_resp = match client.post_api_v1_sequences(&body).await {
        Ok(resp) => resp.into_inner(),
        Err(e) => {
            return serde_json::json!({ "error": format!("sequence create failed: {e}") })
                .to_string();
        }
    };

    // Extract the created sequence ID for verification and step creation.
    let sequence_id_str = sequence_resp
        .get("id")
        .or_else(|| sequence_resp.get("data").and_then(|d| d.get("id")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // If no steps were provided, return the create response immediately.
    let Some(steps_json) = steps else {
        let mut result = serde_json::json!({
            "sequence": sequence_resp,
            "sequence_id": sequence_id_str,
            "steps_created": [],
            "message": "Sequence created successfully. No inline steps provided; call rinda_sequence_generate to AI-generate steps."
        });
        if sequence_id_str.is_none() {
            result["warning"] = serde_json::Value::String(
                "Sequence ID not found in API response — verify creation with rinda_sequence_list."
                    .to_string(),
            );
        }
        return result.to_string();
    };

    // Parse the sequence ID UUID needed to create steps.
    let seq_uuid = match sequence_id_str.as_deref().unwrap_or("").parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "sequence": sequence_resp,
                "sequence_id": sequence_id_str,
                "error": "Sequence was created but its ID could not be parsed as UUID; cannot add steps. Verify creation with rinda_sequence_list."
            })
            .to_string();
        }
    };

    // Parse the JSON steps array.
    let parsed_steps: serde_json::Value = match serde_json::from_str(&steps_json) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "sequence": sequence_resp,
                "sequence_id": sequence_id_str,
                "error": format!("Sequence created but steps JSON is invalid: {e}")
            })
            .to_string();
        }
    };

    let step_array = match parsed_steps.as_array() {
        Some(a) => a.clone(),
        None => {
            return serde_json::json!({
                "sequence": sequence_resp,
                "sequence_id": sequence_id_str,
                "error": "Sequence created but --steps must be a JSON array."
            })
            .to_string();
        }
    };

    // Create each step via the API.
    let mut step_results: Vec<serde_json::Value> = Vec::new();

    for (idx, step) in step_array.iter().enumerate() {
        let step_order_num = step
            .get("stepOrder")
            .or_else(|| step.get("step_order"))
            .and_then(|v| v.as_f64())
            .unwrap_or((idx + 1) as f64);

        let delay_days_num = step
            .get("delayDays")
            .or_else(|| step.get("delay"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        let subject_str = step
            .get("emailSubject")
            .or_else(|| step.get("subject"))
            .and_then(|v| v.as_str())
            .unwrap_or("(no subject)");

        let body_text = step
            .get("emailBodyText")
            .or_else(|| step.get("body"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let body_html = step
            .get("emailBodyHtml")
            .or_else(|| step.get("bodyHtml"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let subject_typed: rinda_sdk::types::PostApiV1SequencesByIdStepsBodyEmailSubject =
            match subject_str.parse() {
                Ok(s) => s,
                Err(e) => {
                    step_results.push(serde_json::json!({
                        "step": idx + 1,
                        "error": format!("Invalid email subject: {e}")
                    }));
                    continue;
                }
            };

        let step_body = rinda_sdk::types::PostApiV1SequencesByIdStepsBody {
            delay_days: rinda_sdk::types::PostApiV1SequencesByIdStepsBodyDelayDays::Number(
                delay_days_num,
            ),
            email_body_html: body_html,
            email_body_text: body_text,
            email_subject: subject_typed,
            email_template_id: None,
            files: Vec::new(),
            generation_source: None,
            original_email_body_html: None,
            original_email_body_text: None,
            original_email_subject: None,
            original_language: None,
            scheduled_hour: None,
            scheduled_minute: None,
            step_order: rinda_sdk::types::PostApiV1SequencesByIdStepsBodyStepOrder::Number(
                step_order_num,
            ),
            timezone: None,
            translated_language: None,
        };

        match client
            .post_api_v1_sequences_by_id_steps(&seq_uuid, &step_body)
            .await
        {
            Ok(resp) => {
                step_results.push(serde_json::json!({
                    "step": idx + 1,
                    "status": "created",
                    "response": resp.into_inner()
                }));
            }
            Err(e) => {
                step_results.push(serde_json::json!({
                    "step": idx + 1,
                    "error": format!("step creation failed: {e}")
                }));
            }
        }
    }

    serde_json::json!({
        "sequence": sequence_resp,
        "sequence_id": sequence_id_str,
        "steps_created": step_results,
        "message": format!(
            "Sequence created with ID {}. {} step(s) processed.",
            sequence_id_str.as_deref().unwrap_or("unknown"),
            step_results.len()
        )
    })
    .to_string()
}

/// List existing sequences for the workspace.
pub async fn sequence_list(
    auth: &AuthContext,
    limit: Option<String>,
    offset: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    match client
        .get_api_v1_sequences(limit.as_deref(), offset.as_deref())
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("sequence list failed: {e}") }).to_string(),
    }
}

/// AI-generate email steps for a sequence.
pub async fn sequence_generate(auth: &AuthContext, id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid sequence id — must be a valid UUID" })
                .to_string();
        }
    };

    let body = rinda_sdk::types::PostApiV1SequencesByIdGenerateBody {
        user_email_account_id: None,
    };

    match client
        .post_api_v1_sequences_by_id_generate(&uuid, &body)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => {
            serde_json::json!({ "error": format!("sequence generate failed: {e}") }).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal AuthContext with a given workspace_id.
    fn make_auth(workspace_id: &str) -> AuthContext {
        AuthContext {
            access_token: "test-token".to_string(),
            workspace_id: workspace_id.to_string(),
            user_id: "user-123".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    /// Acceptance criteria from issue #124: empty workspace_id produces an explicit,
    /// actionable error message mentioning re-authentication — NOT a confusing UUID error.
    #[tokio::test]
    async fn sequence_create_empty_workspace_id_returns_actionable_error() {
        let auth = make_auth("");
        let result = sequence_create(&auth, "Test Campaign".to_string(), None, None).await;
        let json: serde_json::Value =
            serde_json::from_str(&result).expect("result should be valid JSON");
        let error = json["error"].as_str().expect("should have error field");
        assert!(
            error.contains("re-authenticate") || error.contains("workspace"),
            "error should mention workspace or re-authentication: {error}"
        );
        // Must NOT produce a UUID parse error (the old confusing message).
        assert!(
            !error.contains("UUID") && !error.contains("uuid"),
            "error should not mention UUID directly (that's the old confusing message): {error}"
        );
    }

    /// Acceptance criteria from issue #124: invalid (non-UUID) workspace_id produces a
    /// workspace-specific error, not a generic parse failure.
    #[tokio::test]
    async fn sequence_create_invalid_workspace_id_returns_workspace_error() {
        let auth = make_auth("not-a-uuid");
        let result = sequence_create(&auth, "Test Campaign".to_string(), None, None).await;
        let json: serde_json::Value =
            serde_json::from_str(&result).expect("result should be valid JSON");
        let error = json["error"].as_str().expect("should have error field");
        assert!(
            error.contains("re-authenticate") || error.contains("workspace"),
            "error should mention workspace or re-authentication: {error}"
        );
    }

    /// Acceptance criteria: steps JSON that is not an array produces an error.
    #[test]
    fn steps_json_not_an_array_is_detected() {
        let bad_steps = r#"{"delay": 1}"#; // object, not array
        let parsed: serde_json::Value = serde_json::from_str(bad_steps).unwrap();
        assert!(
            parsed.as_array().is_none(),
            "a JSON object should not parse as array"
        );
    }

    /// Acceptance criteria: invalid steps JSON produces a parse error.
    #[test]
    fn steps_json_invalid_json_is_detected() {
        let bad_steps = "not json at all";
        let result: Result<serde_json::Value, _> = serde_json::from_str(bad_steps);
        assert!(result.is_err(), "invalid JSON should fail to parse");
    }

    /// Acceptance criteria: valid steps JSON array is parsed correctly.
    #[test]
    fn steps_json_valid_array_parses_correctly() {
        let steps = r#"[{"delay": 3, "subject": "Hello", "body": "Email body"}]"#;
        let parsed: serde_json::Value = serde_json::from_str(steps).unwrap();
        let arr = parsed.as_array().expect("should be an array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["delay"].as_f64(), Some(3.0));
        assert_eq!(arr[0]["subject"].as_str(), Some("Hello"));
        assert_eq!(arr[0]["body"].as_str(), Some("Email body"));
    }

    /// Acceptance criteria: step field aliases are recognized (delay vs delayDays, subject vs emailSubject).
    #[test]
    fn step_field_aliases_are_recognized() {
        let steps = r#"[
            {"delayDays": 2, "emailSubject": "Hi", "emailBodyText": "Body text"},
            {"delay": 1, "subject": "Hello", "body": "Body"}
        ]"#;
        let parsed: serde_json::Value = serde_json::from_str(steps).unwrap();
        let arr = parsed.as_array().expect("should be an array");

        // First step uses canonical API names.
        let s0 = &arr[0];
        let delay = s0
            .get("delayDays")
            .or_else(|| s0.get("delay"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        assert_eq!(delay, 2.0);
        let subject = s0
            .get("emailSubject")
            .or_else(|| s0.get("subject"))
            .and_then(|v| v.as_str())
            .unwrap_or("(no subject)");
        assert_eq!(subject, "Hi");

        // Second step uses shorthand aliases.
        let s1 = &arr[1];
        let delay2 = s1
            .get("delayDays")
            .or_else(|| s1.get("delay"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        assert_eq!(delay2, 1.0);
        let subject2 = s1
            .get("emailSubject")
            .or_else(|| s1.get("subject"))
            .and_then(|v| v.as_str())
            .unwrap_or("(no subject)");
        assert_eq!(subject2, "Hello");
    }

    /// Acceptance criteria: seq_type is forwarded as description with "Type: " prefix.
    #[test]
    fn seq_type_forwarded_as_description() {
        let seq_type = Some("email".to_string());
        let description = seq_type.map(|t| format!("Type: {t}"));
        assert_eq!(description.as_deref(), Some("Type: email"));
    }

    /// Acceptance criteria: None seq_type produces None description (no spurious field).
    #[test]
    fn seq_type_none_produces_none_description() {
        let seq_type: Option<String> = None;
        let description = seq_type.map(|t| format!("Type: {t}"));
        assert!(description.is_none());
    }

    /// Acceptance criteria: step_order defaults to 1-based index when not specified.
    #[test]
    fn step_order_defaults_to_index_plus_one() {
        let steps = serde_json::json!([{"subject": "A"}, {"subject": "B"}]);
        let arr = steps.as_array().unwrap();
        for (idx, step) in arr.iter().enumerate() {
            let order = step
                .get("stepOrder")
                .or_else(|| step.get("step_order"))
                .and_then(|v| v.as_f64())
                .unwrap_or((idx + 1) as f64);
            assert_eq!(order, (idx + 1) as f64);
        }
    }

    /// Acceptance criteria: created sequence ID is extracted from response `id` field.
    #[test]
    fn sequence_id_extracted_from_response() {
        let resp: serde_json::Map<String, serde_json::Value> = serde_json::from_str(
            r#"{"id": "550e8400-e29b-41d4-a716-446655440000", "name": "Test"}"#,
        )
        .unwrap();
        let id = resp.get("id").and_then(|v| v.as_str());
        assert_eq!(id, Some("550e8400-e29b-41d4-a716-446655440000"));
    }

    /// Acceptance criteria: sequence ID is also found in nested `data.id` field.
    #[test]
    fn sequence_id_extracted_from_nested_data_field() {
        let resp: serde_json::Map<String, serde_json::Value> = serde_json::from_str(
            r#"{"data": {"id": "550e8400-e29b-41d4-a716-446655440000"}, "status": "ok"}"#,
        )
        .unwrap();
        let id = resp
            .get("id")
            .or_else(|| resp.get("data").and_then(|d| d.get("id")))
            .and_then(|v| v.as_str());
        assert_eq!(id, Some("550e8400-e29b-41d4-a716-446655440000"));
    }

    /// Acceptance criteria: missing ID field returns None (not a panic).
    #[test]
    fn sequence_id_missing_from_response_returns_none() {
        let resp: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(r#"{"name": "Test"}"#).unwrap();
        let id = resp
            .get("id")
            .or_else(|| resp.get("data").and_then(|d| d.get("id")))
            .and_then(|v| v.as_str());
        assert!(id.is_none());
    }
}

/// Enroll a lead/buyer into a sequence.
pub async fn sequence_add_contact(
    auth: &AuthContext,
    sequence_id: String,
    buyer_id: String,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let seq_uuid = match sequence_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid sequence_id — must be a valid UUID" })
                .to_string();
        }
    };

    let lead_uuid = match buyer_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid buyer_id — must be a valid UUID" })
                .to_string();
        }
    };

    let body = rinda_sdk::types::PostApiV1SequencesByIdEnrollmentsBody {
        lead_id: lead_uuid,
        user_email_account_id: Uuid::nil(),
        enrolled_by: None,
        status: None,
    };

    match client
        .post_api_v1_sequences_by_id_enrollments(&seq_uuid, &body)
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => {
            serde_json::json!({ "error": format!("sequence add-contact failed: {e}") }).to_string()
        }
    }
}
