use std::clone;

use log::debug;
use serde_json::json;
use twitter_rs_api::{self, auth::SuspiciousLoginError};
use dotenv::dotenv;

fn main() {
    dotenv().ok(); 
    let username = std::env::var("USER").unwrap();
    let password = std::env::var("PASSWORD").unwrap();
    debug!("username: {username}");
    debug!("password: {password}");
    let mut api = twitter_rs_api::TwAPI::new();
    let result = api.login(&username, &password, "", None);
    match result {
        Err(err) => {
            if let error = err.downcast_ref::<SuspiciousLoginError>().unwrap() {
                println!("Enter your username (eg. @user): ");
                let username = api.read_string();
                api.login(&username, &password, "".into(), Some(error.1.clone()));
            }
            // match err {
            //     SuspiciousLoginError => {
            //         debug!("Suspicious error!!!");
            //         let error = err.downcast_ref::<SuspiciousLoginError>();

            //         // println!("Enter your username (eg. @user): ");
            //         // let username = api.read_string();
            //         // let data = json!({
            //         //     "flow_token": token,
            //         //     "subtask_inputs": [{"subtask_id": subtask_id, "enter_text": {"text": username,"link":"next_link"}}]
            //         // });
            //         // // self.handle_suspicies(token.clone(), subtask_id.clone());
            //         // let flow_token = api.get_flow_token(data).unwrap().unwrap();
            //     }
            // }

        }
        Ok(_) => {}
    }
    
    let is_logged_in = api.is_logged_in();
    println!("is logged: {is_logged_in}");
    let user_id = api.me_rest_id().unwrap();
    println!("userid is {user_id}");
    let following = api.get_friends(user_id, true);
}