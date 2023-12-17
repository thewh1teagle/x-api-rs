use std::path::PathBuf;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); 

    let cookies_path = PathBuf::from("cookies.json");
    let username = std::env::var("USER")?;
    let password = std::env::var("PASSWORD")?;
    debug!("username: {username}");
    debug!("password: {password}");
    let mut api = twitter_rs_api::TwAPI::new(Some(&cookies_path))?;
    if !cookies_path.exists() {
        let result = api.login(&username, &password, "", None);
        match result {
            Err(err) => {
                let error = err.downcast_ref::<SuspiciousLoginError>().unwrap();
                println!("Enter your username (eg. @user): ");
                let username = read_string();
                api.login(&username, &password, "".into(), Some(error.1.clone()))?;
            }
            Ok(_) => {}
        }
        
        api.save_session(cookies_path);
    }
    // always call this for extract csrf
    let is_logged_in = api.is_logged_in()?;
    println!("is logged: {is_logged_in}");
    
    let user_id = api.me_rest_id()?;
    println!("userid is {user_id}");
    let mut cursor = "".to_string();
    loop {
        let pagination = api.get_friends(user_id, true, Some(cursor.into()))?;
        cursor = pagination.cursor.clone();
        debug!("Found {:?} following", pagination.entries.len());
        if !pagination.has_more {
            break;
        }
    }
    Ok(())
    
}