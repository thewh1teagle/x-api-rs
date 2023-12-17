use log::debug;
use twitter_rs_api::{self, auth::SuspiciousLoginError};
use dotenv::dotenv;

fn read_string() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("can not read user input");
    input
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); 
    let username = std::env::var("USER")?;
    let password = std::env::var("PASSWORD")?;
    debug!("username: {username}");
    debug!("password: {password}");
    let mut api = twitter_rs_api::TwAPI::new()?;
    let result = api.login(&username, &password, "", None).await;
    match result {
        Err(err) => {
            let error = err.downcast_ref::<SuspiciousLoginError>().unwrap();
            println!("Enter your username (eg. @user): ");
            let username = read_string();
            api.login(&username, &password, "".into(), Some(error.1.clone())).await?;
        }
        Ok(_) => {}
    }
    
    let is_logged_in = api.is_logged_in().await?;
    println!("is logged: {is_logged_in}");
    let user_id = api.me_rest_id().await?;
    println!("userid is {user_id}");
    let mut cursor = "".to_string();
    loop {
        let pagination = api.get_friends(user_id, true, Some(cursor.into())).await?;
        cursor = pagination.cursor.clone();
        debug!("Found {:?} following", pagination.entries.len());
        if !pagination.has_more {
            break;
        }
    }
    Ok(())
    
}