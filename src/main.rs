use dotenv::dotenv;
use log::debug;
use std::env;

mod handlers;
mod models;

use handlers::fcm_handler::FCMHandlerV1;
use models::fcm_payload::FCMPayloadData;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    debug!("Starting FCM notification handler...");

    // Initialize FCM Handler
    let fcm_handler = FCMHandlerV1::new();

    // Example payload
    let payload = FCMPayloadData {
        title: "Hello!".to_string(),
        message: "This is a test notification.".to_string(),
        image: "https://dummyimage.com/600x400/000/fff".to_string(),
        click_action: "OPEN_APP".to_string(),
    };

    let token = "token from client";

    // Send a push notification
    match fcm_handler.push(payload, token.to_string()) {
        Ok(_) => debug!("Notification sent successfully!"),
        Err(err) => debug!("Failed to send notification: {:?}", err),
    }

    Ok(())
}
