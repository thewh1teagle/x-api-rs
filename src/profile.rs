use crate::BEARER_TOKEN;
use anyhow::{Context, Result};
use log::debug;
use serde_json::{json, Value};
use super::TwAPI;

const SEARCH_URL: &str = "https://twitter.com/i/api/graphql/k3027HdkVqbuDPpdoniLKA/Viewer";

impl TwAPI {
    pub fn me(&self) -> Result<Value> {

        let variables = json!(
            {"withCommunitiesMemberships": true,
                     "withSubscribedTab": true, "withCommunitiesCreation": true}
        );

        let features = json!({"responsive_web_graphql_exclude_directive_enabled": true, "verified_phone_label_enabled": true,
        "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
        "responsive_web_graphql_timeline_navigation_enabled": true, "user_data_features": true});

        let q = [
            ("variables", variables.to_string()),
            ("features", features.to_string()),
        ];
        let req = self
            .client
            .get(SEARCH_URL)
            .header("Authorization", format!("Bearer {}", BEARER_TOKEN))
            .header("X-CSRF-Token", self.csrf_token.to_owned())
            .query(&q)
            .build()
            ?;
        let text = self.client.execute(req)?.text()?;
        let res: Value = serde_json::from_str(&text).context("can't convert response to json")?;
        debug!("me res {res}");
        return Ok(res);
    }

    pub fn me_following(&mut self) {

    }

    pub fn me_rest_id(&mut self) -> Result<i64, Box<dyn std::error::Error>> {
        let me = self.me()?;
        let res_id = me
            .get("data").ok_or("data")?
            .get("viewer").ok_or("viewer")?
            .get("user_results").ok_or("viewer")?
            .get("result").ok_or("viewer")?
            .get("rest_id").ok_or("rest id")?.as_str().ok_or("str error")?.parse::<i64>()?;
        Ok(res_id)
        
    }
}
