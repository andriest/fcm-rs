use crate::models::fcm_payload::FCMPayloadData;
use gauth::serv_account::ServiceAccount;
use isahc::{prelude::*, HttpClient, Request};
use log::{debug, error};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

pub struct FCMHandlerV1 {
    key_path: String,
    client: HttpClient,
}

impl FCMHandlerV1 {
    /// Add push notification handler.
    pub fn new() -> FCMHandlerV1 {
        let client = HttpClient::builder()
            .tcp_keepalive(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Build isahc http client");

        FCMHandlerV1 {
            key_path: env::var("GOOGLE_APPLICATION_CREDENTIALS")
                .expect("No GOOGLE_APPLICATION_CREDENTIALS env var"),
            client,
        }
    }

    fn read_service_key_file(&self) -> Result<String, String> {
        let private_key_content = match std::fs::read(&self.key_path) {
            Ok(content) => content,
            Err(err) => return Err(err.to_string()),
        };

        Ok(String::from_utf8(private_key_content).unwrap())
    }

    fn read_service_key_file_json(&self) -> Result<Value, String> {
        let file_content = match self.read_service_key_file() {
            Ok(content) => content,
            Err(err) => return Err(err.to_string()),
        };

        let json_content: Value = match serde_json::from_str(&file_content) {
            Ok(json) => json,
            Err(err) => return Err(err.to_string()),
        };

        Ok(json_content)
    }

    fn get_project_id(&self) -> Result<String, String> {
        let json_content = match self.read_service_key_file_json() {
            Ok(json) => json,
            Err(err) => return Err(err),
        };

        let project_id = match json_content["project_id"].as_str() {
            Some(project_id) => project_id,
            None => return Err("could not get project_id".to_string()),
        };

        Ok(project_id.to_string())
    }

    pub async fn access_token(&self) -> Result<String, String> {
        let scopes = vec!["https://www.googleapis.com/auth/firebase.messaging"];
        let mut service_account = ServiceAccount::from_file(&self.key_path, scopes);
        let access_token = match service_account.access_token().await {
            Ok(access_token) => access_token,
            Err(err) => return Err(err.to_string()),
        };

        let token_no_bearer = access_token.split(" ").collect::<Vec<&str>>()[1];

        Ok(token_no_bearer.to_string())
    }

    async fn get_auth_token(&self) -> Result<String, String> {
        let tkn = match self.access_token().await {
            Ok(tkn) => tkn,
            Err(_) => return Err("could not get access token".to_string()),
        };

        Ok(tkn)
    }

    pub fn push<'a>(&self, payload: FCMPayloadData, token: String) -> Result<(), ()> {
        debug!("Sending to token: {}", &token);

        let message: Value = json!({
            "message": {
                "token": &token,
                "notification": {
                    "title": &payload.title,
                    "body": &payload.message,
                    "image": &payload.image,
                },
                "android": {
                    "notification": {
                        "title": &payload.title,
                        "body": &payload.message,
                        "sound": "default",
                        "click_action": &payload.click_action,
                        "image": &payload.image,
                    }
                }
            }
        });

        let _ = self.send_notification_to(&message.to_string());

        Ok(())
    }

    pub async fn send_notification_to(&self, message: &str) -> Result<(), String> {
        let project_id = match self.get_project_id() {
            Ok(project_id) => project_id,
            Err(err) => return Err(err),
        };

        let auth_token = match self.get_auth_token().await {
            Ok(tkn) => tkn,
            Err(err) => return Err(err),
        };

        // https://firebase.google.com/docs/reference/fcm/rest/v1/projects.messages/send
        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            project_id
        );

        let payload = message.as_bytes().to_vec();

        let request = Request::post(&url)
            .header("Content-Type", "application/json")
            .header(
                "content-length",
                format!("{}", payload.len() as u64).as_str(),
            )
            .timeout(Duration::from_secs(60))
            .header("Authorization", format!("Bearer {}", auth_token).as_str())
            .body(payload)
            .expect("Build isahc http request");

        match self.client.send(request) {
            Ok(mut resp) => {
                let resp_text = resp.text().unwrap_or("".to_string());
                if resp.status().is_success() {
                    debug!("FCM Sent: {}", resp_text);
                } else {
                    error!("FCM Error {}: {}", resp.status(), resp_text);
                }
            }
            Err(err) => {
                error!("FCM request error: {:?}", err);
            }
        }

        Ok(())
    }
}
