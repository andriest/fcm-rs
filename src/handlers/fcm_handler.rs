use crate::models::fcm_payload::FCMPayloadData;
use gauth::serv_account::ServiceAccount;
use isahc::{prelude::*, HttpClient, Request};
use log::{debug, error};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum FcmError {
    #[error("Failed to read service key file: {0}")]
    ServiceKeyRead(String),
    #[error("Failed to parse service key JSON: {0}")]
    ServiceKeyParse(String),
    #[error("Missing project_id in service key")]
    MissingProjectId,
    #[error("Failed to obtain access token: {0}")]
    AccessToken(String),
    #[error("HTTP request failed: {0}")]
    HttpRequest(String),
}

pub struct FCMHandlerV1 {
    key_path: String,
    client: HttpClient,
}

impl Default for FCMHandlerV1 {
    fn default() -> Self {
        Self::new()
    }
}

impl FCMHandlerV1 {
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

    fn read_service_key_file(&self) -> Result<String, FcmError> {
        let content = std::fs::read(&self.key_path)
            .map_err(|e| FcmError::ServiceKeyRead(e.to_string()))?;
        String::from_utf8(content).map_err(|e| FcmError::ServiceKeyRead(e.to_string()))
    }

    fn read_service_key_file_json(&self) -> Result<Value, FcmError> {
        let content = self.read_service_key_file()?;
        serde_json::from_str(&content).map_err(|e| FcmError::ServiceKeyParse(e.to_string()))
    }

    fn get_project_id(&self) -> Result<String, FcmError> {
        let json = self.read_service_key_file_json()?;
        json["project_id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(FcmError::MissingProjectId)
    }

    pub async fn access_token(&self) -> Result<String, FcmError> {
        let scopes = vec!["https://www.googleapis.com/auth/firebase.messaging"];
        let mut service_account = ServiceAccount::from_file(&self.key_path, scopes);
        let token = service_account
            .access_token()
            .await
            .map_err(|e| FcmError::AccessToken(e.to_string()))?;

        // The token is returned as "Bearer <token>"; extract only the token part.
        token
            .split_once(' ')
            .map(|(_, t)| t.to_string())
            .ok_or_else(|| FcmError::AccessToken("unexpected token format".to_string()))
    }

    pub async fn push(&self, payload: &FCMPayloadData, token: &str) -> Result<(), FcmError> {
        debug!("Sending to token: {}", token);

        let message = json!({
            "message": {
                "token": token,
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

        self.send_notification_to(&message.to_string()).await
    }

    pub async fn send_notification_to(&self, message: &str) -> Result<(), FcmError> {
        let project_id = self.get_project_id()?;
        let auth_token = self.access_token().await?;

        // https://firebase.google.com/docs/reference/fcm/rest/v1/projects.messages/send
        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            project_id
        );

        let payload = message.as_bytes().to_vec();

        let request = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("content-length", payload.len().to_string().as_str())
            .timeout(Duration::from_secs(60))
            .header("Authorization", format!("Bearer {}", auth_token).as_str())
            .body(payload)
            .expect("Build isahc http request");

        match self.client.send(request) {
            Ok(mut resp) => {
                let body = resp.text().unwrap_or_default();
                if resp.status().is_success() {
                    debug!("FCM Sent: {}", body);
                } else {
                    error!("FCM Error {}: {}", resp.status(), body);
                }
            }
            Err(err) => return Err(FcmError::HttpRequest(err.to_string())),
        }

        Ok(())
    }
}
