pub mod auth;
pub mod search;
pub mod types;
use reqwest::blocking::Client;
mod profile;
mod users;

pub const LOGIN_URL: &str = "https://api.twitter.com/1.1/onboarding/task.json";
pub const LOGOUR_URL: &str = "https://api.twitter.com/1.1/account/logout.json";
pub const GUEST_ACTIVE_URL: &str = "https://api.twitter.com/1.1/guest/activate.json";
pub const VERIFY_CREDENTIALS_URL: &str =
    "https://api.twitter.com/1.1/account/verify_credentials.json";
pub const OAUTH_URL: &str = "https://api.twitter.com/oauth2/token";
pub const BEARER_TOKEN: &str = "AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA";
pub const APP_CONSUMER_KEY: &str = "3nVuSoBZnx6U4vzUxf5w";
pub const APP_CONSUMER_SECRET: &str = "Bcs59EFbbsdF6Sl9Ng71smgStWEGwXXKSjYvPVt7qys";

pub struct TwAPI {
    client: Client,
    guest_token: String,
    csrf_token: String,
}

// #[cfg(test)]
// mod tests {
//     use crate::auth::Flow;

//     use super::{
//         types::{Data, Tweet},
//         *,
//     };

//     fn login(api: &mut TwAPI) -> Result<Option<Flow>, Box<dyn std::error::Error>> {
//         dotenv::dotenv().ok();
//         let name = std::env::var("TWITTER_USER_NAME").unwrap();
//         let pwd = std::env::var("TWITTER_USER_PASSWORD").unwrap();
//         api.login(&name, &pwd, "")
//     }

//     fn search(api: &mut TwAPI) -> Result<Data, reqwest::Error> {
//         let content = "@shareverse_bot -filter:retweets";
//         let limit = 50;
//         let cursor = "";
//         api.search(content, limit, cursor)
//     }

//     fn search_tweets(api: &mut TwAPI) -> Result<(Vec<Tweet>, String), reqwest::Error> {
//         let content = "@shareverse_bot -filter:retweets";
//         let limit = 50;
//         let cursor = "";
//         api.search_tweets(content, limit, cursor)
//     }

//     fn test() {
//         let mut api = TwAPI::new();
//         let loggined = login(&mut api);
//         assert!(loggined.is_ok());

//         let is_logged_in = api.is_logged_in();
//         assert!(is_logged_in);

//         let result = search(&mut api);
//         assert!(result.is_ok());

//         let res = search_tweets(&mut api);
//         assert!(res.is_ok());

//         let (tweets, _) = res.unwrap();
//         assert!(tweets.len() > 0);
//     }
// }
