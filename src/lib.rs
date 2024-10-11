pub mod auth;
pub mod search;
pub mod types;
use std::{path::PathBuf, sync::Arc};

use reqwest::Client;
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
    cookie_store: Arc<reqwest_cookie_store::CookieStoreMutex>,
    session_path: Option<PathBuf>,
}
