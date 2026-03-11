// Order history tool implementation.
//
// Note: The RINDA API does not have a dedicated /orders endpoint. This tool
// uses the leads/search endpoint as the closest available approximation.

use crate::auth::{AuthContext, json_to_text, sdk_client};

/// Retrieve order history using leads/search as approximation.
pub async fn order_history(
    auth: &AuthContext,
    buyer_id: Option<String>,
    days_inactive: Option<u32>,
) -> String {
    let client = sdk_client(Some(&auth.access_token));

    let updated_before_str = days_inactive.map(|days| {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days));
        cutoff.format("%Y-%m-%d").to_string()
    });

    let search_ref = buyer_id.as_deref();
    let updated_before_ref = updated_before_str.as_deref();

    match client
        .get_api_v1_leads_search(
            None,               // business_type
            None,               // city
            None,               // country
            None,               // created_after
            None,               // created_before
            None,               // created_by_ids
            None,               // customer_group_id
            None,               // filters
            None,               // lead_status
            None,               // limit
            None,               // offset
            search_ref,         // search (buyer_id as name/id filter)
            None,               // search_type
            None,               // sort_field
            None,               // sort_order
            None,               // updated_after
            updated_before_ref, // updated_before (days_inactive cutoff)
            None,               // workspace_ids
        )
        .await
    {
        Ok(resp) => json_to_text(&resp.into_inner()),
        Err(e) => serde_json::json!({ "error": format!("order history failed: {e}") }).to_string(),
    }
}
