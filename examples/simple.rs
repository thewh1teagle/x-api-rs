/*
cp .env.template .env
export RUST_LOG=DEBUG
cargo run --example simple
*/
use dotenvy::dotenv;
use std::path::PathBuf;
use x_api_rs::auth::SuspiciousLoginError;

fn read_string() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("can not read user input");
    input
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let _ = dotenv();
    let cookies_path = PathBuf::from("cookies.json");
    let username = std::env::var("USERNAME")?;
    let password = std::env::var("PASSWORD")?;
    tracing::debug!("username: {username}");
    tracing::debug!("password: {password}");
    let mut api = x_api_rs::TwAPI::new(Some(cookies_path.clone()))?;
    if !cookies_path.exists() {
        let result = api.login(&username, &password, "", None).await;
        if let Err(error) = result {
            let error = error.downcast_ref::<SuspiciousLoginError>().unwrap();
            println!("Enter your username (eg. @user): ");
            let username = read_string();
            api.login(&username, &password, "", Some(error.1.clone()))
                .await?;
        }
        api.save_session().unwrap();
    }
    // always call this for extract csrf
    let is_logged_in = api.is_logged_in().await?;
    tracing::info!("is logged: {is_logged_in}");

    let user_id = api.me_rest_id().await?;
    let res = api.get_following_ids(user_id.to_string(), -1).await?;
    tracing::debug!("response: {res:?}");
    let ids = res
        .entries
        .iter()
        .map(|v| v.as_i64().unwrap_or_default().to_string())
        .collect();
    let res = api.users_lookup(ids).await?;
    tracing::debug!("response: {res:?}");
    // loop {
    //     let pagination = api.get_friends(user_id, true, Some(cursor.into()))?;
    //     cursor = pagination.cursor.clone();
    //     tracing::debug!("Found {:?} following", pagination.entries.len());
    //     if !pagination.has_more {
    //         break;
    //     }
    // }
    Ok(())
}
