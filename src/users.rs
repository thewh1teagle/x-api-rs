use crate::BEARER_TOKEN;
use log::{debug, trace};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

use super::{
    types::{parse_legacy_tweet, Data, Tweet},
    TwAPI,
};

const SEARCH_URL: &str =
    "https://twitter.com/i/api/graphql/9zwVLJ48lmVUk8u_Gh9DmA/ProfileSpotlightsQuery";

const FOLLOWING_URL: &str =
    "https://twitter.com/i/api/graphql/OLcddmNLPVXGDgSdSVj0ow/Following";

const FOLLOWERS_URL: &str =
    "https://twitter.com/i/api/graphql/WWFQL1d4gxtqm2mjZCRa-Q/Followers";

    

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationResponse {
    pub cursor: String,
    pub entries: Vec<Value>,
    pub has_more: bool
}

fn findkey(name: &str, value: Value) -> Option<Value> {
    match value {
        Value::Object(map) => {
            // Check if the key exists in the map
            if let Some(val) = map.get(name) {
                return Some(val.clone());
            }

            // Recursively search in nested objects
            for (_, v) in map {
                if let Some(result) = findkey(name, v) {
                    return Some(result);
                }
            }
        }
        Value::Array(vec) => {
            // Recursively search in array elements
            for v in vec {
                if let Some(result) = findkey(name, v) {
                    return Some(result);
                }
            }
        }
        _ => {}
    }

    None
}

fn find_object(data: Vec<Value>, key_start_with: &str, value_start_with: &str) -> Option<Value> {
    // Find object by inner key and value
    for value in data {
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                if key.starts_with(key_start_with) && val.is_string() {
                    let string_val = val.as_str().unwrap();
                    if string_val.starts_with(value_start_with) {
                        return Some(value.clone());
                    }
                }
            }
        }
    }
    None
}

impl TwAPI {
    pub fn user_id(&mut self, username: String) -> Result<String, Box<dyn std::error::Error>> {
        let username = username.replace("@", "");
        if username.as_bytes()[0].is_ascii_digit() {
            return Ok(username);
        } else {
            let variables = json!(
                {"screen_name": username}
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
                .unwrap();
            let text = self.client.execute(req).unwrap().text().unwrap();
            let res: Value = serde_json::from_str(&text)?;
            debug!("me res {res}");
            let rest_id = res
                .get("data")
                .ok_or("data")?
                .get("viewer")
                .ok_or("viewer")?
                .get("user_results")
                .ok_or("user resulsts")?
                .get("result")
                .ok_or("result")?
                .get("rest_id")
                .ok_or("rest id")?
                .as_str()
                .ok_or("convert to str failed for rest id")?
                .to_string();
            return Ok(rest_id);
        }
    }

    pub fn get_friends(&mut self, user_id: i64, following: bool, start_cursor: Option<String>) -> Result<PaginationResponse, Box<dyn std::error::Error>> {
        let variables = json!(
            {"userId": user_id, "count": 2,
                 "includePromotedContent": true, "cursor": start_cursor, "product": "latest"}
        );
        debug!("variables: {variables}");



        let features = json!({
            "responsive_web_graphql_exclude_directive_enabled": true, "verified_phone_label_enabled": true,
            "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
            "responsive_web_graphql_timeline_navigation_enabled": true, "user_data_features": true,

            "rweb_lists_timeline_redesign_enabled": true, "creator_subscriptions_tweet_preview_api_enabled": true, "tweetypie_unmention_optimization_enabled": true,
            "responsive_web_edit_tweet_api_enabled": true, "graphql_is_translatable_rweb_tweet_is_translatable_enabled": true, "view_counts_everywhere_api_enabled": true,
            "longform_notetweets_consumption_enabled": true, "responsive_web_twitter_article_tweet_consumption_enabled": false, "tweet_awards_web_tipping_enabled": false,
            "freedom_of_speech_not_reach_fetch_enabled": true, "standardized_nudges_misinfo": true, "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": false,
            "longform_notetweets_rich_text_read_enabled": true, "longform_notetweets_inline_media_enabled": false, "responsive_web_media_download_video_enabled": false, "responsive_web_enhance_cards_enabled": false});

        let q = [
            ("variables", variables.to_string()),
            ("features", features.to_string()),
        ];

        let url = if following {FOLLOWING_URL} else {FOLLOWERS_URL};
        let req = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", BEARER_TOKEN))
            .header("X-CSRF-Token", self.csrf_token.to_owned())
            .query(&q)
            .build()
            .unwrap();
        let text = self.client.execute(req).unwrap().text().unwrap();
        let res: Value = serde_json::from_str(&text)?;
        let instructions = &res["data"]["user"]["result"]["timeline"]["timeline"]["instructions"];
        trace!("instructions: {instructions}");

        let entries = findkey("entries", instructions.to_owned()).unwrap_or_default();
        let entries = entries
            .as_array()
            .ok_or_else(|| "entries is not an array or not present".to_string())?;

        let bottom_cursor = find_object(entries.to_owned(), "entryId", "cursor-bottom").unwrap_or_default();
        let bottom_cursor = bottom_cursor["content"]["value"].as_str().unwrap_or_default();

        trace!("bottom_cursor {bottom_cursor}");
        let data_entries: Vec<Value> = entries.iter().filter(|entry| {
            let entry_id = entry["entryId"].as_str().unwrap_or_default();
            debug!("entry id: {entry_id}");
            return entry_id.starts_with("user-");
        }).cloned().collect();
        // Assuming you want to return a clone of the data
        return Ok(PaginationResponse{cursor: bottom_cursor.into(), entries: data_entries.to_owned(), has_more: data_entries.len() > 0 && !bottom_cursor.is_empty()});
    }
}
