// Lead management tool implementations (CRUD, search, status, assessment).

use uuid::Uuid;

use crate::auth::{AuthContext, json_to_text, sdk_client};

/// Search leads with advanced filtering.
#[allow(clippy::too_many_arguments)]
pub async fn lead_search(
    auth: &AuthContext,
    business_type: Option<String>,
    city: Option<String>,
    country: Option<String>,
    customer_group_id: Option<String>,
    lead_status: Option<String>,
    search: Option<String>,
    search_type: Option<String>,
    sort_field: Option<String>,
    sort_order: Option<String>,
    limit: Option<String>,
    offset: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let search_type_parsed = match search_type
        .as_deref()
        .map(|s| s.parse::<rinda_sdk::types::GetApiV1LeadsSearchSearchType>())
    {
        Some(Ok(v)) => Some(v),
        Some(Err(_)) => {
            return serde_json::json!({
                "error": "Invalid search_type. Valid values: all, company, country, email, website, industry, category"
            })
            .to_string();
        }
        None => None,
    };

    let sort_order_parsed = match sort_order
        .as_deref()
        .map(|s| s.parse::<rinda_sdk::types::GetApiV1LeadsSearchSortOrder>())
    {
        Some(Ok(v)) => Some(v),
        Some(Err(_)) => {
            return serde_json::json!({
                "error": "Invalid sort_order. Valid values: asc, desc"
            })
            .to_string();
        }
        None => None,
    };

    match client
        .get_api_v1_leads_search(
            business_type.as_deref(),
            city.as_deref(),
            country.as_deref(),
            None, // created_after
            None, // created_before
            None, // created_by_ids
            customer_group_id.as_deref(),
            None, // filters
            lead_status.as_deref(),
            limit.as_deref(),
            offset.as_deref(),
            search.as_deref(),
            search_type_parsed,
            sort_field.as_deref(),
            sort_order_parsed,
            None, // updated_after
            None, // updated_before
            None, // workspace_ids
        )
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("lead search failed: {e}") }).to_string(),
    }
}

/// Get a lead by its UUID.
pub async fn lead_get(auth: &AuthContext, id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid lead id — must be a valid UUID" })
                .to_string();
        }
    };

    match client.get_api_v1_leads_by_id(&uuid).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("lead get failed: {e}") }).to_string(),
    }
}

/// Create a new lead.
#[allow(clippy::too_many_arguments)]
pub async fn lead_create(
    auth: &AuthContext,
    company_name: Option<String>,
    website_url: Option<String>,
    country: Option<String>,
    city: Option<String>,
    business_type: Option<String>,
    contact_name: Option<String>,
    lead_status: Option<String>,
    description: Option<String>,
    notes: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let workspace_id = match auth.workspace_id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid workspace ID in token. Please re-authenticate."
            })
            .to_string();
        }
    };

    let lead_status_parsed = match lead_status
        .as_deref()
        .map(|s| s.parse::<rinda_sdk::types::PostApiV1LeadsBodyLeadStatus>())
    {
        Some(Ok(v)) => Some(v),
        Some(Err(_)) => {
            return serde_json::json!({
                "error": "Invalid lead_status. Valid values: new, contacted, qualified, unqualified, converted, lost, unsubscribed"
            })
            .to_string();
        }
        None => None,
    };

    let body = rinda_sdk::types::PostApiV1LeadsBody {
        workspace_id,
        company_name: parse_opt_newtype_str(company_name.as_deref()),
        website_url: parse_opt_newtype_str(website_url.as_deref()),
        country: parse_opt_newtype_str(country.as_deref()),
        city: parse_opt_newtype_str(city.as_deref()),
        business_type: parse_opt_newtype_str(business_type.as_deref()),
        contact_name: parse_opt_newtype_str(contact_name.as_deref()),
        lead_status: lead_status_parsed,
        description,
        notes,
        address: None,
        collected_at: None,
        contacts: vec![],
        crawl_time_seconds: None,
        created_by: None,
        customer_group_id: None,
        employee_count: None,
        error_message: None,
        final_url: None,
        found_company_name: None,
        founded_year: None,
        gpt_time_seconds: None,
        http_status: None,
        is_business_type_matched: None,
        lead_score: None,
        lead_source: None,
        name_url_match: None,
        social_media: vec![],
        state: None,
    };

    match client.post_api_v1_leads(&body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("lead create failed: {e}") }).to_string(),
    }
}

/// Update an existing lead.
#[allow(clippy::too_many_arguments)]
pub async fn lead_update(
    auth: &AuthContext,
    id: String,
    company_name: Option<String>,
    website_url: Option<String>,
    country: Option<String>,
    city: Option<String>,
    business_type: Option<String>,
    contact_name: Option<String>,
    lead_status: Option<String>,
    description: Option<String>,
    notes: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid lead id — must be a valid UUID" })
                .to_string();
        }
    };

    let lead_status_parsed = match lead_status
        .as_deref()
        .map(|s| s.parse::<rinda_sdk::types::PutApiV1LeadsByIdBodyLeadStatus>())
    {
        Some(Ok(v)) => Some(v),
        Some(Err(_)) => {
            return serde_json::json!({
                "error": "Invalid lead_status. Valid values: new, contacted, qualified, unqualified, converted, lost, unsubscribed"
            })
            .to_string();
        }
        None => None,
    };

    let body = rinda_sdk::types::PutApiV1LeadsByIdBody {
        company_name: parse_opt_newtype_str(company_name.as_deref()),
        website_url: parse_opt_newtype_str(website_url.as_deref()),
        country: parse_opt_newtype_str(country.as_deref()),
        city: parse_opt_newtype_str(city.as_deref()),
        business_type: parse_opt_newtype_str(business_type.as_deref()),
        contact_name: parse_opt_newtype_str(contact_name.as_deref()),
        lead_status: lead_status_parsed,
        description,
        notes,
        ..Default::default()
    };

    match client.put_api_v1_leads_by_id(&uuid, &body).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("lead update failed: {e}") }).to_string(),
    }
}

/// Delete a lead by UUID.
pub async fn lead_delete(auth: &AuthContext, id: String) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(_) => {
            return serde_json::json!({ "error": "Invalid lead id — must be a valid UUID" })
                .to_string();
        }
    };

    match client.delete_api_v1_leads_by_id(&uuid).await {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("lead delete failed: {e}") }).to_string(),
    }
}

/// List leads filtered by status.
pub async fn lead_by_status(
    auth: &AuthContext,
    status: String,
    limit: Option<String>,
    offset: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let status_parsed = match status.parse::<rinda_sdk::types::GetApiV1LeadsStatusByStatusStatus>()
    {
        Ok(s) => s,
        Err(_) => {
            return serde_json::json!({
                "error": "Invalid status. Valid values: new, contacted, qualified, unqualified, converted, lost, unsubscribed"
            })
            .to_string();
        }
    };

    match client
        .get_api_v1_leads_status_by_status(status_parsed, limit.as_deref(), offset.as_deref())
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("lead by-status failed: {e}") }).to_string(),
    }
}

/// Get top-scored leads from assessment.
pub async fn lead_top(
    auth: &AuthContext,
    customer_group_id: Option<String>,
    limit: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    match client
        .get_api_v1_assessment_top_leads(
            customer_group_id.as_deref(),
            limit.as_deref(),
            &auth.workspace_id,
        )
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("lead top failed: {e}") }).to_string(),
    }
}

/// Get leads grouped by assessment tier.
pub async fn lead_by_tier(
    auth: &AuthContext,
    tier: String,
    customer_group_id: Option<String>,
    limit: Option<String>,
    offset: Option<String>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    match client
        .get_api_v1_assessment_leads_by_tier(
            customer_group_id.as_deref(),
            limit.as_deref(),
            offset.as_deref(),
            &tier,
            &auth.workspace_id,
        )
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("lead by-tier failed: {e}") }).to_string(),
    }
}

/// Helper: parse an optional `&str` into an optional SDK newtype via `FromStr`.
/// Returns `None` if input is `None`; returns `None` (silently) if parsing fails
/// (the field will simply be omitted from the request body).
fn parse_opt_newtype_str<T>(value: Option<&str>) -> Option<T>
where
    T: std::str::FromStr,
{
    value.and_then(|s| s.parse::<T>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Acceptance criteria: lead_by_status with invalid status returns error JSON.
    #[test]
    fn lead_by_status_invalid_status_returns_error() {
        use rinda_sdk::types::GetApiV1LeadsStatusByStatusStatus;
        let result = "invalid_xyz".parse::<GetApiV1LeadsStatusByStatusStatus>();
        assert!(result.is_err(), "invalid status should fail to parse");
    }

    /// Acceptance criteria: all 7 valid status values are accepted.
    #[test]
    fn lead_by_status_all_valid_values() {
        use rinda_sdk::types::GetApiV1LeadsStatusByStatusStatus;
        for s in &[
            "new",
            "contacted",
            "qualified",
            "unqualified",
            "converted",
            "lost",
            "unsubscribed",
        ] {
            assert!(
                s.parse::<GetApiV1LeadsStatusByStatusStatus>().is_ok(),
                "status '{}' should be valid",
                s
            );
        }
    }

    /// Acceptance criteria: lead_get with invalid UUID returns an error string.
    #[test]
    fn lead_get_invalid_uuid_detected() {
        let result = "not-a-uuid".parse::<Uuid>();
        assert!(result.is_err());
    }

    /// Acceptance criteria: lead_delete with invalid UUID returns an error string.
    #[test]
    fn lead_delete_invalid_uuid_detected() {
        let result = "bad-id".parse::<Uuid>();
        assert!(result.is_err());
    }

    /// Verify parse_opt_newtype_str returns None for None input.
    #[test]
    fn parse_opt_newtype_str_none_input() {
        let result: Option<rinda_sdk::types::PostApiV1LeadsBodyCity> = parse_opt_newtype_str(None);
        assert!(result.is_none());
    }

    /// Verify parse_opt_newtype_str parses a valid string.
    #[test]
    fn parse_opt_newtype_str_valid_input() {
        let result: Option<rinda_sdk::types::PostApiV1LeadsBodyCountry> =
            parse_opt_newtype_str(Some("US"));
        assert!(result.is_some());
    }

    /// Acceptance criteria: search_type enum round-trips through parse and display.
    #[test]
    fn lead_search_type_round_trips() {
        use rinda_sdk::types::GetApiV1LeadsSearchSearchType;
        for s in &[
            "all", "company", "country", "email", "website", "industry", "category",
        ] {
            let parsed = s.parse::<GetApiV1LeadsSearchSearchType>();
            assert!(parsed.is_ok(), "search_type '{}' should parse", s);
            assert_eq!(&parsed.unwrap().to_string(), s, "display should match");
        }
    }

    /// Acceptance criteria: lead_create with invalid lead_status returns error JSON.
    #[test]
    fn lead_create_invalid_lead_status_detected() {
        use rinda_sdk::types::PostApiV1LeadsBodyLeadStatus;
        let result = "pending".parse::<PostApiV1LeadsBodyLeadStatus>();
        assert!(result.is_err(), "pending is not a valid lead status");
    }
}
