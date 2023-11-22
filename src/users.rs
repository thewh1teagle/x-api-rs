use crate::BEARER_TOKEN;
use log::debug;
use serde_json::{json, Value};
use std::cmp;

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
            let res: Value = serde_json::from_str(&text).unwrap();
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

    pub fn get_friends(&mut self, user_id: i64, following: bool) -> Result<Value, Box<dyn std::error::Error>> {
        let variables = json!(
            {"userId": user_id, "count": 100,
                 "includePromotedContent": false}
        );



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
        println!("text is {text}");
        let res: Value = serde_json::from_str(&text).unwrap();
        debug!("following res {res}");
        return Ok(res);
    }
}
