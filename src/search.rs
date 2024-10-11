use crate::BEARER_TOKEN;
use serde_json::json;
use std::cmp;
use eyre::{ContextCompat, Result};
use super::{
    types::{parse_legacy_tweet, Data, Tweet},
    TwAPI,
};

const SEARCH_URL: &str = "https://twitter.com/i/api/graphql/nK1dw4oV3k4w5TdtcAdSww/SearchTimeline";

impl TwAPI {
    pub async fn search(
        &self,
        query: &str,
        limit: u8,
        cursor: &str,
    ) -> Result<Data> {
        let limit = cmp::min(50u8, limit);

        let mut variables = json!(
            {
                "rawQuery":     query.to_string(),
                "count":        limit,
                "querySource":  "typed_query",
                "product":      "Top"
            }
        );
        let features = json!(
            {
                "rweb_lists_timeline_redesign_enabled":                                    true,
                "responsive_web_graphql_exclude_directive_enabled":                        true,
                "verified_phone_label_enabled":                                            false,
                "creator_subscriptions_tweet_preview_api_enabled":                         true,
                "responsive_web_graphql_timeline_navigation_enabled":                      true,
                "responsive_web_graphql_skip_user_profile_image_extensions_enabled":       false,
                "tweetypie_unmention_optimization_enabled":                                true,
                "responsive_web_edit_tweet_api_enabled":                                   true,
                "graphql_is_translatable_rweb_tweet_is_translatable_enabled":              true,
                "view_counts_everywhere_api_enabled":                                      true,
                "longform_notetweets_consumption_enabled":                                 true,
                "responsive_web_twitter_article_tweet_consumption_enabled":                false,
                "tweet_awards_web_tipping_enabled":                                        false,
                "freedom_of_speech_not_reach_fetch_enabled":                               true,
                "standardized_nudges_misinfo":                                             true,
                "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": true,
                "longform_notetweets_rich_text_read_enabled":                              true,
                "longform_notetweets_inline_media_enabled":                                true,
                "responsive_web_media_download_video_enabled":                             false,
                "responsive_web_enhance_cards_enabled":                                    false,
            }
        );
        let field_toggles = json!(
            {
                "withArticleRichContentState": false
            }
        );
        if cursor.ne("") {
            variables["cursor"] = cursor.to_string().into();
        }
        variables["product"] = "Latest".into();
        let q = [
            ("variables", variables.to_string()),
            ("features", features.to_string()),
            ("fieldToggles", field_toggles.to_string()),
        ];
        let req = self
            .client
            .get(SEARCH_URL)
            .header("Authorization", format!("Bearer {}", BEARER_TOKEN))
            .header("X-CSRF-Token", self.csrf_token.to_owned())
            .query(&q)
            .build()?;
        let text = self
            .client
            .execute(req).await?
            .text().await?;
        let res: Data = serde_json::from_str(&text)?;
        return Ok(res);
    }

    pub async fn search_tweets(
        &self,
        query: &str,
        limit: u8,
        cursor: &str,
    ) -> Result<(Vec<Tweet>, String)> {
        let search_result = self.search(query, limit, cursor).await;
        let mut cursor = String::from("");
        match search_result {
            Ok(res) => {
                let mut tweets: Vec<Tweet> = vec![];
                let instructions = res
                    .data
                    .search_by_raw_query
                    .search_timeline
                    .timeline
                    .instructions
                    .context("can't parse tweets from timeline, instructions is none")?;
                for item in instructions {
                    if item.instruction_type.ne("TimelineAddEntries")
                        && item.instruction_type.ne("TimelineReplaceEntry")
                    {
                        continue;
                    }
                    if item.entry.is_some() {
                        let entry = item.entry.context("entry is none")?;
                        let cursor_type = entry.content.cursor_type.unwrap_or("".to_string());
                        if cursor_type.eq("Bottom") {
                            if entry.content.value.is_some() {
                                cursor = entry.content.value.context("content value is none")?;
                            }
                        }
                    }
                    for entry in item.entries {
                        if entry.content.item_content.is_none() {
                            continue;
                        }
                        let item = entry.content.item_content.context("item content is none")?;
                        if item.tweet_display_type.eq("Tweet") {
                            let core = item.tweet_results.result.core;
                            if core.is_none() {
                                continue;
                            }
                            let u = core.context("core is none")?.user_results.result.legacy.context("legacy is none")?;
                            let t = item.tweet_results.result.legacy;
                            if let Ok(tweet) = parse_legacy_tweet(&u, &t) {
                                tweets.push(tweet)
                            }
                        }
                    }
                }
                tracing::debug!("Found tweets: {tweets:?}");
                Ok((tweets, cursor))
            }
            Err(e) => Err(e),
        }
    }
}
