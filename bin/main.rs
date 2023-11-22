use log::debug;
use twitter_rs_api;
use dotenv::dotenv;

fn main() {
    dotenv().ok(); 
    let username = std::env::var("USER").unwrap();
    let password = std::env::var("PASSWORD").unwrap();
    debug!("username: {username}");
    debug!("password: {password}");
    let mut api = twitter_rs_api::TwAPI::new();
    api.login(&username, &password, "").unwrap();
    let is_logged_in = api.is_logged_in();
    println!("is logged: {is_logged_in}");
    let user_id = api.me_rest_id().unwrap();
    println!("userid is {user_id}");
    let following = api.get_friends(user_id, true);
}