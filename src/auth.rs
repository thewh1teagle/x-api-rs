use super::{TwAPI, BEARER_TOKEN, GUEST_ACTIVE_URL, LOGIN_URL, VERIFY_CREDENTIALS_URL};
use log::debug;
use reqwest::{blocking::{self}, Error};
use serde::Deserialize;
use serde_json::{self, json};
use env_logger;

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub id_str: String,
    pub name: String,
    pub screen_name: String,
}
#[derive(Deserialize, Debug)]
pub struct OpenAccount {
    pub user: Option<User>,
    pub next_link: Option<Link>,
    pub attribution_event: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Subtask {
    pub subtask_id: String,
    pub open_account: Option<OpenAccount>,
}

#[derive(Deserialize, Debug)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct Flow {
    pub errors: Option<Vec<ApiError>>,
    pub flow_token: String,
    pub status: String,
    pub subtasks: Vec<Subtask>,
    pub js_instrumentation: Option<Insrumentation>,
}

#[derive(Deserialize, Debug)]
pub struct Insrumentation {
    pub url: String,
    pub timeout_ms: i64,
    pub next_link: Link,
}

#[derive(Deserialize, Debug)]
pub struct Link {
    pub link_type: String,
    pub link_id: String,
}

#[derive(Deserialize, Debug)]
pub struct GuestToken {
    pub guest_token: String,
}

#[derive(Deserialize, Debug)]
pub struct VerifyCredentials {
    pub errors: Option<Vec<ApiError>>,
}

impl TwAPI {
    pub fn new() -> TwAPI {
        env_logger::init();
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .build()
            .unwrap();
        return TwAPI {
            client,
            csrf_token: String::from(""),
            guest_token: String::from(""),
        };
    }
    fn get_flow(&mut self, body: serde_json::Value) -> Result<Flow, Box<dyn std::error::Error>> {
        if self.guest_token.is_empty() {
            self.get_guest_token()?
            
        }
        let res = self
            .client
            .post(LOGIN_URL)
            .header("Authorization", format!("Bearer {}", BEARER_TOKEN))
            .header("Content-Type", "application/json")
            .header("User-Agent", "TwitterAndroid/99")
            .header("X-Guest-Token", self.guest_token.replace("\"", ""))
            .header("X-Twitter-Auth-Type", "OAuth2Client")
            .header("X-Twitter-Active-User", "yes")
            .header("X-Twitter-Client-Language", "en")
            .json(&body)
            .send()
            ?;

        let cookies = res.cookies();
        for cookie in cookies {
            if cookie.name().eq("ct0") {
                self.csrf_token = cookie.value().to_string()
            }
        }
        let text = res.text()?;
        debug!("text: {text}");
        let result: Flow = serde_json::from_str(text.as_str())?;
        return Ok(result);
    }

    fn get_flow_token(&mut self, data: serde_json::Value) -> Result<Option<Flow>, String> {
        let res = self.get_flow(data);
        match res {
            Ok(info) => {
                
                if info.subtasks.len() > 0 {
                    let subtask_id = info.subtasks[0].subtask_id.as_str();
                    match subtask_id {
                        // "LoginEnterAlternateIdentifierSubtask"
                        "LoginAcid"
                        | "LoginTwoFactorAuthChallenge"
                        | "DenyLoginSubtask" => {
                            return Err(format!("Auth error: {}", subtask_id));
                        }
                        _ => return Ok(Some(info)),
                    }
                }
                return Ok(Some(info));
            }
            Err(e) => Err(format!("Request error: {}", e.to_string())),
        }
    }

    fn get_guest_token(&mut self) -> Result<(), Error> {
        let token = format!("Bearer {}", BEARER_TOKEN);
        let res = self
            .client
            .post(GUEST_ACTIVE_URL)
            .header("Authorization", token)
            .send()
            ;
        match res {
            Ok(r) => {
                let op = r.json::<serde_json::Value>()?;
                let guest_token = op.get("guest_token").unwrap();
                self.guest_token = guest_token.to_string();
                return Ok(());
            }
            Err(e) => Err(e),
        }
    }


    fn read_string(&self) -> String {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("can not read user input");
        input
    }


    fn handle_suspicies(&mut self, token: String, subtask_id: String) -> Result<Flow, Error> {
        debug!("handle suspicies");
        println!("Enter your username (eg. @user): ");
        let username = self.read_string();
        let body = json!({
            "flow_token": token,
            "subtask_inputs": [{"subtask_id": subtask_id, "enter_text": {"text": username,"link":"next_link"}}]
        });


        let res = self
            .client
            .post(LOGIN_URL)
            .header("Authorization", format!("Bearer {}", BEARER_TOKEN))
            .header("Content-Type", "application/json")
            .header("User-Agent", "TwitterAndroid/99")
            .header("X-Guest-Token", self.guest_token.replace("\"", ""))
            .header("X-Twitter-Auth-Type", "OAuth2Client")
            .header("X-Twitter-Active-User", "yes")
            .header("X-Twitter-Client-Language", "en")
            .json(&body)
            .send()
            ?;

        let cookies = res.cookies();
        for cookie in cookies {
            if cookie.name().eq("ct0") {
                self.csrf_token = cookie.value().to_string()
            }
        }
        let result: Flow = res.json()?;
        return Ok(result);

        
    }

    pub fn login(
        &mut self,
        user_name: &str,
        password: &str,
        confirmation: &str,
    ) -> Result<Option<Flow>, String> {
        // flow start
        let data = json!(
            {
                "flow_name": "login",
                "input_flow_data": {
                    "flow_context" : {
                        "debug_overrides": {},
                        "start_location": {
                            "location": "splash_screen"
                        }
                    }
                }
            }
        );
        let flow_token = self.get_flow_token(data)?.unwrap().flow_token;

        // flow instrumentation step
        let data = json!(
            {
                "flow_token": flow_token,
                "subtask_inputs" : [{
                    "subtask_id": "LoginJsInstrumentationSubtask",
                    "js_instrumentation":{
                        "response": "{}",
                        "link": "next_link"
                    }
                }],
            }
        );
        let flow_token = self.get_flow_token(data)?.unwrap().flow_token;

        // flow username step
        let data = json!(
            {
                "flow_token": flow_token,
                "subtask_inputs" : [{
                    "subtask_id": "LoginEnterUserIdentifierSSO",
                    "settings_list": {
                        "setting_responses" : [{
                            "key":           "user_identifier",
                            "response_data": {
                                "text_data" :{
                                    "result": user_name
                                }
                            }
                        }],
                        "link": "next_link"
                    }
                }]
            }
        );

        let mut flow_token = self.get_flow_token(data).unwrap().unwrap();
        let token = flow_token.flow_token.to_owned();
        let subtask_id = flow_token.subtasks[0].subtask_id.clone();

        if subtask_id == "LoginEnterAlternateIdentifierSubtask" {
            println!("Enter your username (eg. @user): ");
            let username = self.read_string();
            let data = json!({
                "flow_token": token,
                "subtask_inputs": [{"subtask_id": subtask_id, "enter_text": {"text": username,"link":"next_link"}}]
            });
            
            // self.handle_suspicies(token.clone(), subtask_id.clone());
            flow_token = self.get_flow_token(data).unwrap().unwrap();
        }

        // flow password step
        let data = json!(
            {
                "flow_token": flow_token.flow_token,
                "subtask_inputs": [{
                    "subtask_id":     "LoginEnterPassword",
                    "enter_password": {
                        "password": password,
                        "link": "next_link"
                    },
                }]
            }
        );

        let flow_token = self.get_flow_token(data)?.unwrap().flow_token;


        // flow duplication check
        let data = json!(
            {
                "flow_token": flow_token,
                "subtask_inputs": [{
                    "subtask_id":              "AccountDuplicationCheck",
                    "check_logged_in_account": {
                        "link": "AccountDuplicationCheck_false"
                    },
                }]
            }
        );
        let flow_token = self.get_flow_token(data);

        match flow_token {
            Err(e) => {
                let mut confirmation_subtask = "";
                for item in vec!["LoginAcid", "LoginTwoFactorAuthChallenge"] {
                    if e.contains(item) {
                        confirmation_subtask = item;
                        break;
                    }
                }
                if !confirmation_subtask.is_empty() {
                    if confirmation.is_empty() {
                        let msg = format!(
                            "confirmation data required for {}",
                            confirmation_subtask.to_owned()
                        );
                        return Err(msg);
                    }
                    let data = json!(
                        {
                            "flow_token": "",
                            "subtask_inputs": {
                                    "subtask_id": confirmation_subtask,
                                    "enter_text": {
                                        "text": confirmation,
                                        "link": "next_link",
                                    },
                            },
                        }
                    );
                    return self.get_flow_token(data);
                }
                Ok(None)
            }
            Ok(_) => return Ok(None),
        }
    }

    pub fn is_logged_in(&mut self) -> bool {
        let req = self
            .client
            .get(VERIFY_CREDENTIALS_URL)
            .header("Authorization", format!("Bearer {}", BEARER_TOKEN))
            .header("X-CSRF-Token", self.csrf_token.to_owned())
            .build()
            .unwrap();
        let res = self.client.execute(req).unwrap();
        let cookies = res.cookies();
        for cookie in cookies {
            if cookie.name().eq("ct0") {
                self.csrf_token = cookie.value().to_string()
            }
        }
        let text = res.text().unwrap();
        let res: VerifyCredentials = serde_json::from_str(&text).unwrap();
        res.errors.is_none()
    }
}
