use super::{TwAPI, BEARER_TOKEN, GUEST_ACTIVE_URL, LOGIN_URL, VERIFY_CREDENTIALS_URL};
use log::debug;
use env_logger;
use serde::Deserialize;
use serde_json::{self, json};
use anyhow::{Result, bail, Context};


#[derive(Clone, Debug, thiserror::Error)]
#[error("Suspicious Login")]
pub struct SuspiciousLoginError(
    pub String,
    pub Flow, // error message, latest flow
);

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub id_str: String,
    pub name: String,
    pub screen_name: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct OpenAccount {
    pub user: Option<User>,
    pub next_link: Option<Link>,
    pub attribution_event: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Subtask {
    pub subtask_id: String,
    pub open_account: Option<OpenAccount>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Flow {
    pub errors: Option<Vec<ApiError>>,
    pub flow_token: String,
    pub status: String,
    pub subtasks: Vec<Subtask>,
    pub js_instrumentation: Option<Insrumentation>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Insrumentation {
    pub url: String,
    pub timeout_ms: i64,
    pub next_link: Link,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Link {
    pub link_type: String,
    pub link_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GuestToken {
    pub guest_token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VerifyCredentials {
    pub errors: Option<Vec<ApiError>>,
}

impl TwAPI {
    pub fn new() -> Result<TwAPI> {
        let _ = env_logger::try_init();
        let client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .build()
            .context("cant build cookie store")?;
        return Ok(TwAPI {
            client,
            csrf_token: String::from(""),
            guest_token: String::from(""),
        });
    }
    async fn get_flow(&mut self, body: serde_json::Value) -> Result<Flow> {
        if self.guest_token.is_empty() {
            self.get_guest_token()
            .await?
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
            .await?;

        let cookies = res.cookies();
        for cookie in cookies {
            if cookie.name().eq("ct0") {
                self.csrf_token = cookie.value().to_string()
            }
        }
        let text = res.text().await?;
        debug!("text: {text}");
        let result: Flow = serde_json::from_str(text.as_str())?;
        return Ok(result);
    }

    pub async fn get_flow_token(
        &mut self,
        data: serde_json::Value,
    ) -> Result<Option<Flow>> {
        let res = self.get_flow(data).await;
        match res {
            Ok(info) => {
                if info.subtasks.len() > 0 {
                    let subtask_id = info.subtasks[0].subtask_id.as_str();
                    match subtask_id {
                        // "LoginEnterAlternateIdentifierSubtask"
                        "LoginAcid" | "LoginTwoFactorAuthChallenge" | "DenyLoginSubtask" => {
                            bail!("Auth error: {}", subtask_id);
                        }
                        _ => return Ok(Some(info)),
                    }
                }
                return Ok(Some(info));
            }
            Err(e) => {
                bail!("Request error: {}", e.to_string())
            },
        }
    }

    async fn get_guest_token(&mut self) -> Result<()> {
        let token = format!("Bearer {}", BEARER_TOKEN);
        let res = self
            .client
            .post(GUEST_ACTIVE_URL)
            .header("Authorization", token)
            .send()
            .await?;
        let op = res.json::<serde_json::Value>().await?;
        let guest_token = op.get("guest_token").context("cant get guest_token")?;
        self.guest_token = guest_token.to_string();
        Ok(())
    }



    pub async fn before_password_steps(&mut self, username: String) -> Result<Flow> {
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
        let flow_token = self.get_flow_token(data).await?.context("cant get folow token")?.flow_token;

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
        let flow_token = self.get_flow_token(data).await?.context("cant get folow token")?.flow_token;

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
                                    "result": username
                                }
                            }
                        }],
                        "link": "next_link"
                    }
                }]
            }
        );

        let flow = self.get_flow_token(data).await?.context("flow is none")?;
        // let token = flow.flow_token.to_owned();
        let subtask_id = flow.subtasks[0].subtask_id.clone();

        // asking for username because of suspicies log in
        if subtask_id == "LoginEnterAlternateIdentifierSubtask" {
            return Err(SuspiciousLoginError("".into(), flow).into());
        }
        Ok(flow)
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
        confirmation: &str,
        latest_flow: Option<Flow>,
    ) -> Result<Option<Flow>> {
        // flow start

        let mut flow: Flow;
        if latest_flow.is_some() {
            debug!("taking latest flow");
            flow = latest_flow.context("latest flow is none")?;
            let subtask_id = flow.subtasks[0].subtask_id.clone();
            let data = json!({
                "flow_token": flow.flow_token,
                "subtask_inputs": [{"subtask_id": subtask_id, "enter_text": {"text": username,"link":"next_link"}}]
            });
            // self.handle_suspicies(token.clone(), subtask_id.clone());
            flow = self.get_flow_token(data).await.context("flow token is none")?.context("inner flow token is none")?;
        } else {
            flow = self.before_password_steps(username.into()).await?;
        }
        // flow password step
        let data = json!(
            {
                "flow_token": flow.flow_token,
                "subtask_inputs": [{
                    "subtask_id":     "LoginEnterPassword",
                    "enter_password": {
                        "password": password,
                        "link": "next_link"
                    },
                }]
            }
        );

        let flow_token = self.get_flow_token(data).await?.context("flow token is none in password step")?.flow_token;

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
        let flow_token = self.get_flow_token(data).await;

        match flow_token {
            Err(e) => {
                let mut confirmation_subtask = "";
                for item in vec!["LoginAcid", "LoginTwoFactorAuthChallenge"] {
                    if e.to_string().contains(item) {
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
                        bail!(msg)
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
                    return self.get_flow_token(data).await;
                }
                Ok(None)
            }
            Ok(_) => return Ok(None),
        }
    }

    pub async fn is_logged_in(&mut self) -> Result<bool> {
        let req = self
            .client
            .get(VERIFY_CREDENTIALS_URL)
            .header("Authorization", format!("Bearer {}", BEARER_TOKEN))
            .header("X-CSRF-Token", self.csrf_token.to_owned())
            .build()
            .context("Cant build http request in logged in")?;
        let res = self.client.execute(req).await.context("failed execute request")?;
        let cookies = res.cookies();
        for cookie in cookies {
            if cookie.name().eq("ct0") {
                self.csrf_token = cookie.value().to_string()
            }
        }
        let text = res.text().await.context("res is not text")?;
        let res: VerifyCredentials = serde_json::from_str(&text).context("res is not diseralizable")?;
        Ok(res.errors.is_none())
    }
}
